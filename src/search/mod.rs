use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy};

use crate::cli::commands::search::SearchArgs;
use crate::core::{NitroError, NitroResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub tap: String,
    pub formula_path: PathBuf,
    pub score: f32,
}

pub struct SearchEngine {
    index: Index,
    reader: IndexReader,
    name_field: Field,
    description_field: Field,
    version_field: Field,
    tap_field: Field,
    path_field: Field,
}

impl SearchEngine {
    pub async fn new() -> Result<Self> {
        let config_dir = directories::ProjectDirs::from("com", "nitro", "nitro")
            .ok_or_else(|| NitroError::Other("Could not determine config directory".into()))?;
        
        let index_dir = config_dir.data_dir().join("search_index");
        std::fs::create_dir_all(&index_dir)?;

        // Create schema
        let mut schema_builder = Schema::builder();
        let name_field = schema_builder.add_text_field("name", TEXT | STORED);
        let description_field = schema_builder.add_text_field("description", TEXT | STORED);
        let version_field = schema_builder.add_text_field("version", STORED);
        let tap_field = schema_builder.add_text_field("tap", STORED);
        let path_field = schema_builder.add_text_field("path", STORED);
        let schema = schema_builder.build();

        // Create or open index
        let index = if index_dir.join("meta.json").exists() {
            Index::open_in_dir(&index_dir)?
        } else {
            Index::create_in_dir(&index_dir, schema.clone())?
        };

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;

        Ok(Self {
            index,
            reader,
            name_field,
            description_field,
            version_field,
            tap_field,
            path_field,
        })
    }

    pub async fn search(&self, query: &str, args: &SearchArgs) -> NitroResult<Vec<SearchResult>> {
        let searcher = self.reader.searcher();
        
        let query_parser = if args.fuzzy {
            // For fuzzy search, we'll use a more permissive approach
            let mut parser = QueryParser::for_index(&self.index, vec![self.name_field, self.description_field]);
            parser.set_field_fuzzy(self.name_field, true, 1, true);
            if args.description {
                parser.set_field_fuzzy(self.description_field, true, 1, true);
            }
            parser
        } else {
            let fields = if args.description {
                vec![self.name_field, self.description_field]
            } else {
                vec![self.name_field]
            };
            QueryParser::for_index(&self.index, fields)
        };

        let query = query_parser.parse_query(query)
            .map_err(|e| NitroError::SearchError(format!("Query parse error: {}", e)))?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(args.limit))?;
        
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;
            
            let name = retrieved_doc
                .get_first(self.name_field)
                .and_then(|v| match v {
                    tantivy::schema::OwnedValue::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            
            let description = retrieved_doc
                .get_first(self.description_field)
                .and_then(|v| match v {
                    tantivy::schema::OwnedValue::Str(s) => Some(s.clone()),
                    _ => None,
                });
            
            let version = retrieved_doc
                .get_first(self.version_field)
                .and_then(|v| match v {
                    tantivy::schema::OwnedValue::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            
            let tap = retrieved_doc
                .get_first(self.tap_field)
                .and_then(|v| match v {
                    tantivy::schema::OwnedValue::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            
            let formula_path = retrieved_doc
                .get_first(self.path_field)
                .and_then(|v| match v {
                    tantivy::schema::OwnedValue::Str(s) => Some(PathBuf::from(s.clone())),
                    _ => None,
                })
                .unwrap_or_default();
            
            results.push(SearchResult {
                name,
                description,
                version,
                tap,
                formula_path,
                score,
            });
        }

        Ok(results)
    }

    pub async fn index_formula(&self, name: &str, description: Option<&str>, version: &str, tap: &str, path: &PathBuf) -> Result<()> {
        let mut index_writer: IndexWriter = self.index.writer(50_000_000)?;
        
        let mut doc = doc!();
        doc.add_text(self.name_field, name);
        if let Some(desc) = description {
            doc.add_text(self.description_field, desc);
        }
        doc.add_text(self.version_field, version);
        doc.add_text(self.tap_field, tap);
        doc.add_text(self.path_field, path.to_string_lossy());
        
        index_writer.add_document(doc)?;
        index_writer.commit()?;
        
        Ok(())
    }

    pub async fn rebuild_index(&self) -> Result<()> {
        use crate::core::tap::TapManager;
        use crate::core::formula::FormulaParser;
        
        // Clear existing index
        let mut index_writer: IndexWriter = self.index.writer(50_000_000)?;
        index_writer.delete_all_documents()?;
        
        let tap_manager = TapManager::new().await?;
        let formula_parser = FormulaParser::new();
        
        // Index all formulae from all taps
        for tap in tap_manager.list_taps().await? {
            let formula_dir = tap.path.join("Formula");
            if !formula_dir.exists() {
                continue;
            }
            
            self.index_formulae_recursive(&mut index_writer, &formula_parser, &formula_dir, &tap.name).await?;
        }
        
        index_writer.commit()?;
        Ok(())
    }
    pub async fn rebuild_index_with_tap_manager(&self, tap_manager: &crate::core::tap::TapManager) -> Result<()> {
        use crate::core::formula::FormulaParser;
        
        // Clear existing index
        let mut index_writer: IndexWriter = self.index.writer(50_000_000)?;
        index_writer.delete_all_documents()?;
        
        let formula_parser = FormulaParser::new();
        
        // Index all formulae from all taps using the provided tap_manager
        for tap in tap_manager.list_taps().await? {
            let formula_dir = tap.path.join("Formula");
            if !formula_dir.exists() {
                continue;
            }
            
            self.index_formulae_recursive(&mut index_writer, &formula_parser, &formula_dir, &tap.name).await?;
        }
        
        index_writer.commit()?;
        Ok(())
    }

    fn index_formulae_recursive<'a>(
        &'a self,
        index_writer: &'a mut IndexWriter,
        formula_parser: &'a crate::core::formula::FormulaParser,
        dir: &'a std::path::Path,
        tap_name: &'a str
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    // Recursively index subdirectories
                    self.index_formulae_recursive(index_writer, formula_parser, &path, tap_name).await?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("rb") {
                    if let Ok(formula) = formula_parser.parse_file(&path).await {
                        let name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(&formula.name);
                        
                        let mut doc = doc!();
                        doc.add_text(self.name_field, name);
                        if let Some(desc) = &formula.description {
                            doc.add_text(self.description_field, desc);
                        }
                        doc.add_text(self.version_field, &formula.version);
                        doc.add_text(self.tap_field, tap_name);
                        doc.add_text(self.path_field, path.to_string_lossy());
                        
                        index_writer.add_document(doc)?;
                    }
                }
            }
            Ok(())
        })
    }
}
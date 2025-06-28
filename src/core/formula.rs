use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::core::{NitroError, NitroResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Formula {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub sources: Vec<Source>,
    pub dependencies: Vec<Dependency>,
    pub build_dependencies: Vec<Dependency>,
    pub optional_dependencies: Vec<Dependency>,
    pub conflicts: Vec<String>,
    pub install_script: Option<String>,
    pub test_script: Option<String>,
    pub caveats: Option<String>,
    pub binary_packages: Vec<BinaryPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub url: String,
    pub sha256: String,
    pub mirror: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub build_only: bool,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryPackage {
    pub platform: String,
    pub arch: String,
    pub url: String,
    pub sha256: String,
}

pub struct FormulaManager {
    cache_dir: PathBuf,
    tap_manager: super::tap::TapManager,
    parser: FormulaParser,
}

impl FormulaManager {
    pub async fn new() -> Result<Self> {
        let config_dir = directories::ProjectDirs::from("com", "nitro", "nitro")
            .ok_or_else(|| NitroError::Other("Could not determine config directory".into()))?;
        
        let cache_dir = config_dir.cache_dir().join("formulae");
        std::fs::create_dir_all(&cache_dir)?;

        let tap_manager = super::tap::TapManager::new().await?;
        let parser = FormulaParser::new();

        Ok(Self {
            cache_dir,
            tap_manager,
            parser,
        })
    }

    pub async fn get_formula(&self, name: &str) -> NitroResult<Formula> {
        // Check cache first
        if let Ok(formula) = self.load_from_cache(name) {
            return Ok(formula);
        }

        // Find formula in taps
        let formula_path = self.tap_manager.find_formula(name).await?;
        let formula = self.parser.parse_file(&formula_path).await?;
        
        // Cache the parsed formula
        self.save_to_cache(&formula)?;
        
        Ok(formula)
    }

    pub async fn update_formulae(&self) -> Result<()> {
        // Clear cache when updating formulae
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
        }
        
        // Update all taps
        self.tap_manager.update_all_taps().await?;
        
        Ok(())
    }

    pub async fn rebuild_search_index(&self) -> Result<()> {
        use crate::search::SearchEngine;
        
        let search_engine = SearchEngine::new().await?;
        search_engine.rebuild_index_with_tap_manager(&self.tap_manager).await?;
        
        Ok(())
    }

    fn load_from_cache(&self, name: &str) -> NitroResult<Formula> {
        let cache_path = self.cache_dir.join(format!("{}.json", name));
        if cache_path.exists() {
            let data = std::fs::read_to_string(&cache_path)?;
            let formula: Formula = serde_json::from_str(&data)?;
            Ok(formula)
        } else {
            Err(NitroError::PackageNotFound(name.to_string()))
        }
    }

    fn save_to_cache(&self, formula: &Formula) -> Result<()> {
        let cache_path = self.cache_dir.join(format!("{}.json", formula.name));
        let data = serde_json::to_string_pretty(formula)?;
        std::fs::write(cache_path, data)?;
        Ok(())
    }
}

pub struct FormulaParser {
    // We'll implement a basic Ruby formula parser
}

impl FormulaParser {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn parse_file(&self, path: &Path) -> NitroResult<Formula> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| NitroError::FormulaParse(format!("Failed to read formula file: {}", e)))?;
        
        self.parse_content(&content)
    }

    pub fn parse_content(&self, content: &str) -> NitroResult<Formula> {
        // This is a simplified parser - in reality, we'd need a proper Ruby parser
        // For now, we'll use regex to extract basic information
        
        let name = self.extract_class_name(content)?;
        let desc = self.extract_desc(content);
        let homepage = self.extract_homepage(content);
        let url = self.extract_url(content)?;
        let sha256 = self.extract_sha256(content)?;
        let version = self.extract_version_from_url(&url);
        let (dependencies, build_dependencies) = self.extract_dependencies(content)?;
        
        Ok(Formula {
            name,
            version,
            description: desc,
            homepage,
            license: None, // TODO: Extract license
            sources: vec![Source {
                url,
                sha256,
                mirror: None,
            }],
            dependencies,
            build_dependencies,
            optional_dependencies: vec![],
            conflicts: vec![],
            install_script: self.extract_install_block(content),
            test_script: self.extract_test_block(content),
            caveats: self.extract_caveats(content),
            binary_packages: vec![], // Will be populated from CDN
        })
    }

    fn extract_class_name(&self, content: &str) -> NitroResult<String> {
        let re = regex::Regex::new(r"class\s+(\w+)\s*<\s*Formula").unwrap();
        if let Some(cap) = re.captures(content) {
            if let Some(name_match) = cap.get(1) {
                Ok(name_match.as_str().to_lowercase())
            } else {
                Err(NitroError::FormulaParse("Could not extract formula class name".into()))
            }
        } else {
            Err(NitroError::FormulaParse("Could not find formula class name".into()))
        }
    }

    fn extract_desc(&self, content: &str) -> Option<String> {
        let re = regex::Regex::new(r#"desc\s+"([^"]+)""#).unwrap();
        re.captures(content).and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
    }

    fn extract_homepage(&self, content: &str) -> Option<String> {
        let re = regex::Regex::new(r#"homepage\s+"([^"]+)""#).unwrap();
        re.captures(content).and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
    }

    fn extract_url(&self, content: &str) -> NitroResult<String> {
        let re = regex::Regex::new(r#"url\s+"([^"]+)""#).unwrap();
        if let Some(cap) = re.captures(content) {
            if let Some(url_match) = cap.get(1) {
                Ok(url_match.as_str().to_string())
            } else {
                Err(NitroError::FormulaParse("Could not extract download URL".into()))
            }
        } else {
            Err(NitroError::FormulaParse("Could not find download URL".into()))
        }
    }

    fn extract_sha256(&self, content: &str) -> NitroResult<String> {
        let re = regex::Regex::new(r#"sha256\s+"([a-fA-F0-9]{64})""#).unwrap();
        if let Some(cap) = re.captures(content) {
            if let Some(sha_match) = cap.get(1) {
                Ok(sha_match.as_str().to_string())
            } else {
                Err(NitroError::FormulaParse("Could not extract SHA256 checksum".into()))
            }
        } else {
            Err(NitroError::FormulaParse("Could not find SHA256 checksum".into()))
        }
    }

    fn extract_version_from_url(&self, url: &str) -> String {
        // Try multiple patterns to extract version
        let patterns = [
            r"/tags/v?(\d+\.\d+(?:\.\d+)*)",  // GitHub tags
            r"download/v?(\d+\.\d+(?:\.\d+)*)", // GitHub releases
            r"[-_/]v?(\d+\.\d+(?:\.\d+)*)",  // Common patterns like -1.2.3 or /v1.2.3
        ];
        
        for pattern in &patterns {
            let re = regex::Regex::new(pattern).unwrap();
            if let Some(cap) = re.captures(url) {
                if let Some(ver_match) = cap.get(1) {
                    return ver_match.as_str().to_string();
                }
            }
        }
        
        "unknown".to_string()
    }

    fn extract_dependencies(&self, content: &str) -> NitroResult<(Vec<Dependency>, Vec<Dependency>)> {
        let mut deps = Vec::new();
        let mut build_deps = Vec::new();
        let re = regex::Regex::new(r#"depends_on\s+"([^"]+)"(?:\s*=>\s*:(\w+))?"#).unwrap();
        
        for cap in re.captures_iter(content) {
            if let Some(name_match) = cap.get(1) {
                let name = name_match.as_str().to_string();
                let build_only = cap.get(2).map(|m| m.as_str() == "build").unwrap_or(false);
                
                let dep = Dependency {
                    name,
                    version: None,
                    build_only,
                    optional: false,
                };
                
                if build_only {
                    build_deps.push(dep);
                } else {
                    deps.push(dep);
                }
            }
        }
        
        Ok((deps, build_deps))
    }

    fn extract_install_block(&self, content: &str) -> Option<String> {
        // Extract the install block (simplified - doesn't handle nested blocks properly)
        let re = regex::Regex::new(r"def install\s*\n((?:.*\n)*?)\s*end").unwrap();
        re.captures(content).and_then(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
    }

    fn extract_test_block(&self, content: &str) -> Option<String> {
        let re = regex::Regex::new(r"test do\s*\n((?:.*\n)*?)\s*end").unwrap();
        re.captures(content).and_then(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
    }

    fn extract_caveats(&self, content: &str) -> Option<String> {
        // Handle heredoc style caveats (most common in modern formulae)
        let heredoc_re = regex::Regex::new(r"def caveats\s*\n\s*<<[-~]EOS\s*\n((?:.*\n)*?)\s*EOS").unwrap();
        if let Some(cap) = heredoc_re.captures(content) {
            if let Some(caveats_match) = cap.get(1) {
                return Some(caveats_match.as_str().trim().to_string());
            }
        }
        
        // Handle string style caveats (less common)
        let string_re = regex::Regex::new(r#"def caveats\s*\n?\s*"([^"]*)""#).unwrap();
        if let Some(cap) = string_re.captures(content) {
            if let Some(caveats_match) = cap.get(1) {
                return Some(caveats_match.as_str().to_string());
            }
        }
        
        None
    }
}
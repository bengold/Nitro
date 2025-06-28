use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct SearchArgs {
    /// Search query
    #[arg(required = true)]
    pub query: String,

    /// Search in descriptions as well
    #[arg(short, long)]
    pub description: bool,

    /// Fuzzy search
    #[arg(short, long)]
    pub fuzzy: bool,

    /// Maximum number of results
    #[arg(short, long, default_value = "20")]
    pub limit: usize,
}

fn find_matching_formulae(dir: &std::path::Path, query: &str) -> Result<Vec<(String, std::path::PathBuf)>> {
    let mut matches = Vec::new();
    let query_lower = query.to_lowercase();
    
    fn search_dir(dir: &std::path::Path, query: &str, matches: &mut Vec<(String, std::path::PathBuf)>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                search_dir(&path, query, matches)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rb") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem.to_lowercase().contains(query) {
                        matches.push((stem.to_string(), path));
                    }
                }
            }
        }
        Ok(())
    }
    
    search_dir(dir, &query_lower, &mut matches)?;
    Ok(matches)
}

pub async fn execute(args: SearchArgs) -> Result<()> {
    use crate::search::SearchEngine;
    use crate::ui::display;

    let search_engine = SearchEngine::new().await?;
    let results = search_engine.search(&args.query, &args).await?;

    if results.is_empty() {
        // Try partial matching as fallback
        use crate::core::tap::TapManager;
        let tap_manager = TapManager::new().await?;
        let mut found_packages = Vec::new();
        
        // Search for formulae containing the query string
        for tap in tap_manager.list_taps().await? {
            let formula_dir = tap.path.join("Formula");
            if formula_dir.exists() {
                if let Ok(entries) = find_matching_formulae(&formula_dir, &args.query) {
                    for (name, path) in entries {
                        found_packages.push((name, tap.name.clone(), path));
                    }
                }
            }
        }
        
        if found_packages.is_empty() {
            // Try common aliases
            let aliased_query = match args.query.as_str() {
                "python" => "python@3.12",
                "python3" => "python@3.12", 
                "ruby" => "ruby@3.3",
                "node" => "node@22",
                "nodejs" => "node@22",
                "postgresql" => "postgresql@17",
                "postgres" => "postgresql@17",
                "mysql" => "mysql@9.1",
                _ => &args.query,
            };
            
            if aliased_query != args.query {
                use crate::core::formula::FormulaManager;
                let formula_manager = FormulaManager::new().await?;
                match formula_manager.get_formula(aliased_query).await {
                    Ok(formula) => {
                        println!("Found package: {} (using common alias)", formula.name);
                        if let Some(desc) = &formula.description {
                            println!("  {}", desc);
                        }
                        println!("  Version: {}", formula.version);
                        if let Some(homepage) = &formula.homepage {
                            println!("  Homepage: {}", homepage);
                        }
                        return Ok(());
                    }
                    Err(_) => {}
                }
            }
            
            println!("No packages found matching '{}'", args.query);
            println!("\nTip: Try searching with more specific names, e.g.:");
            println!("  nitro search python@3.12");
            println!("  nitro search node@22");
        } else {
            // Show found packages
            println!("Found {} packages matching '{}':", found_packages.len(), args.query);
            for (name, tap, _path) in found_packages.iter().take(20) {
                println!("  {} (from {})", name, tap);
            }
            if found_packages.len() > 20 {
                println!("  ... and {} more", found_packages.len() - 20);
            }
            
            println!("\nUse 'nitro info <package>' to see details");
        }
    } else {
        display::show_search_results(&results);
    }

    Ok(())
}
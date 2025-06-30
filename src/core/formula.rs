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
            eprintln!("DEBUG: Loaded formula {} from cache with {} sources", formula.name, formula.sources.len());
            return Ok(formula);
        }
        eprintln!("DEBUG: Formula {} not in cache, will parse", name);

        // Find formula in taps
        let formula_path = self.tap_manager.find_formula(name).await?;
        eprintln!("DEBUG: Found formula at: {}", formula_path.display());
        let formula = self.parser.parse_file(&formula_path).await?;
        eprintln!("DEBUG: Parsed formula {} with {} sources", formula.name, formula.sources.len());
        
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
        eprintln!("DEBUG: Saving formula {} to cache with {} sources", formula.name, formula.sources.len());
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
        eprintln!("DEBUG: Parsing formula file: {}", path.display());
        let content = std::fs::read_to_string(path)
            .map_err(|e| NitroError::FormulaParse(format!("Failed to read formula file: {}", e)))?;
        
        eprintln!("DEBUG: Formula content length: {} chars", content.len());
        self.parse_content(&content)
    }

    pub fn parse_content(&self, content: &str) -> NitroResult<Formula> {
        // This is a simplified parser - in reality, we'd need a proper Ruby parser
        // For now, we'll use regex to extract basic information
        
        let name = self.extract_class_name(content)?;
        eprintln!("DEBUG: Parsing formula: {}", name);
        let desc = self.extract_desc(content);
        let homepage = self.extract_homepage(content);
        let url = self.extract_url(content).ok();
        eprintln!("DEBUG: Extracted URL: {:?}", url);
        let sha256 = self.extract_sha256(content).ok();
        eprintln!("DEBUG: Extracted SHA256: {:?}", sha256);
        let version = if let Some(ref u) = url {
            self.extract_version_from_url(u)
        } else {
            self.extract_version_from_content(content).unwrap_or_else(|| "unknown".to_string())
        };
        let (dependencies, build_dependencies) = self.extract_dependencies(content)?;
        
        let binary_packages = self.extract_bottles(content, &name, &version)?;
        
        Ok(Formula {
            name,
            version,
            description: desc,
            homepage,
            license: None, // TODO: Extract license
            sources: if let Some(url) = url {
                // For git URLs, we don't need SHA256
                if url.ends_with(".git") {
                    vec![Source {
                        url,
                        sha256: String::new(), // Empty SHA256 for git URLs
                        mirror: None,
                    }]
                } else if let Some(sha256) = sha256 {
                    vec![Source {
                        url,
                        sha256,
                        mirror: None,
                    }]
                } else {
                    vec![] // No valid source
                }
            } else {
                vec![] // No sources for formulas that build from git or other methods
            },
            dependencies,
            build_dependencies,
            optional_dependencies: vec![],
            conflicts: vec![],
            install_script: self.extract_install_block(content),
            test_script: self.extract_test_block(content),
            caveats: self.extract_caveats(content),
            binary_packages,
        })
    }

    fn extract_class_name(&self, content: &str) -> NitroResult<String> {
        let re = regex::Regex::new(r"class\s+(\w+)\s*<\s*Formula").unwrap();
        if let Some(cap) = re.captures(content) {
            if let Some(name_match) = cap.get(1) {
                // Convert class name format to package name format
                // e.g., PythonAT312 -> python@3.12
                let class_name = name_match.as_str();
                let name = if let Some(at_pos) = class_name.find("AT") {
                    // Handle versioned formulae like PythonAT312
                    let (base, version_part) = class_name.split_at(at_pos);
                    let version = &version_part[2..]; // Skip "AT"
                    
                    // Insert dots in version number (312 -> 3.12)
                    let formatted_version = if version.len() >= 2 {
                        format!("{}.{}", &version[0..1], &version[1..])
                    } else {
                        version.to_string()
                    };
                    
                    format!("{}@{}", base.to_lowercase(), formatted_version)
                } else {
                    class_name.to_lowercase()
                };
                Ok(name)
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
        // Try standard URL format (with optional trailing comma for multiline entries)
        let re = regex::Regex::new(r#"url\s+"([^"]+)",?"#).unwrap();
        if let Some(cap) = re.captures(content) {
            if let Some(url_match) = cap.get(1) {
                let url = url_match.as_str();
                eprintln!("DEBUG: Extracted URL: {}", url);
                // Check if it's a git URL with additional parameters
                if url.ends_with(".git") {
                    // For git URLs, we need to extract tag/revision info
                    if let Some(tag_match) = regex::Regex::new(r#"tag:\s*"([^"]+)""#).unwrap().captures(content) {
                        if let Some(tag) = tag_match.get(1) {
                            eprintln!("DEBUG: Found git URL with tag: {}", tag.as_str());
                        }
                    }
                }
                return Ok(url.to_string());
            }
        }
        
        Err(NitroError::FormulaParse("Could not find download URL".into()))
    }

    fn extract_sha256(&self, content: &str) -> NitroResult<String> {
        // Try multiple SHA256 patterns
        let patterns = [
            r#"sha256\s+"([a-fA-F0-9]{64})""#,  // Standard format
            r#"sha256\s+["']([a-fA-F0-9]{64})["']"#,  // With single quotes
            r#"sha256\s+:?\s*["']([a-fA-F0-9]{64})["']"#,  // With symbol notation
        ];
        
        for pattern in &patterns {
            let re = regex::Regex::new(pattern).unwrap();
            if let Some(cap) = re.captures(content) {
                if let Some(sha_match) = cap.get(1) {
                    return Ok(sha_match.as_str().to_string());
                }
            }
        }
        
        eprintln!("DEBUG: Could not find SHA256 in formula content");
        Err(NitroError::FormulaParse("Could not find SHA256 checksum".into()))
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

    fn extract_version_from_content(&self, content: &str) -> Option<String> {
        // Try to extract version from version directive
        let re = regex::Regex::new(r#"version\s+"([^"]+)""#).unwrap();
        if let Some(cap) = re.captures(content) {
            if let Some(ver_match) = cap.get(1) {
                return Some(ver_match.as_str().to_string());
            }
        }
        
        // Try to extract from revision or tag
        let re = regex::Regex::new(r#"revision\s+"([^"]+)""#).unwrap();
        if let Some(cap) = re.captures(content) {
            if let Some(ver_match) = cap.get(1) {
                return Some(ver_match.as_str().to_string());
            }
        }
        
        None
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

    fn extract_bottles(&self, content: &str, formula_name: &str, _version: &str) -> NitroResult<Vec<BinaryPackage>> {
        let mut bottles = Vec::new();
        
        // Find the bottle block
        let bottle_re = regex::Regex::new(r"bottle do\s*\n((?:.*\n)*?)\s*end").unwrap();
        if let Some(bottle_cap) = bottle_re.captures(content) {
            if let Some(bottle_block) = bottle_cap.get(1) {
                let bottle_content = bottle_block.as_str();
                eprintln!("DEBUG: Found bottle block with {} chars", bottle_content.len());
                
                // Extract SHA256 entries
                // Pattern: sha256 cellar: :any_skip_relocation, platform: "sha256"
                let sha_re = regex::Regex::new(r#"sha256(?:\s+cellar:\s*:\w+,)?\s+(\w+):\s*"([a-fA-F0-9]{64})""#).unwrap();
                
                for cap in sha_re.captures_iter(bottle_content) {
                    if let (Some(platform_match), Some(sha_match)) = (cap.get(1), cap.get(2)) {
                        let platform_str = platform_match.as_str();
                        let sha256 = sha_match.as_str().to_string();
                        
                        // Map Homebrew platform names to our platform/arch
                        let (platform, arch) = match platform_str {
                            "arm64_sequoia" | "arm64_sonoma" | "arm64_ventura" | "arm64_monterey" => ("darwin", "aarch64"),
                            "sequoia" | "sonoma" | "ventura" | "monterey" | "big_sur" => ("darwin", "x86_64"),
                            "x86_64_linux" => ("linux", "x86_64"),
                            "aarch64_linux" => ("linux", "aarch64"),
                            _ => continue, // Skip unknown platforms
                        };
                        
                        // Construct bottle URL
                        // Homebrew bottles are actually stored at a different location
                        // The ghcr.io URLs need special handling, so we'll use the direct download URL
                        let os_name = match platform_str {
                            "arm64_sequoia" => "arm64_sequoia",
                            "arm64_sonoma" => "arm64_sonoma", 
                            "arm64_ventura" => "arm64_ventura",
                            "arm64_monterey" => "arm64_monterey",
                            "sequoia" => "sequoia",
                            "sonoma" => "sonoma",
                            "ventura" => "ventura", 
                            "monterey" => "monterey",
                            "big_sur" => "big_sur",
                            "x86_64_linux" => "x86_64_linux",
                            _ => platform_str,
                        };
                        
                        // Use the direct GitHub Packages download URL format
                        // Format: https://ghcr.io/v2/homebrew/core/FORMULA/blobs/sha256:SHA256
                        // But we need to use the bottle filename format instead
                        let _bottle_filename = format!("{}-{}.{}.bottle.tar.gz", 
                            formula_name, _version, os_name);
                        
                        // Store the ghcr.io URL - proper authentication will be needed for download
                        let url = format!(
                            "https://ghcr.io/v2/homebrew/core/{}/blobs/sha256:{}",
                            formula_name.replace("@", "/"),
                            sha256
                        );
                        
                        bottles.push(BinaryPackage {
                            platform: platform.to_string(),
                            arch: arch.to_string(),
                            url,
                            sha256,
                        });
                        
                        eprintln!("DEBUG: Found bottle for {}/{}: {}", platform, arch, platform_str);
                    }
                }
            }
        }
        
        eprintln!("DEBUG: Extracted {} bottles for {}", bottles.len(), formula_name);
        Ok(bottles)
    }
}
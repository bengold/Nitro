use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::core::{NitroError, NitroResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tap {
    pub name: String,
    pub url: String,
    pub path: PathBuf,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct TapManager {
    taps_dir: PathBuf,
    db: sled::Db,
}

impl TapManager {
    pub async fn new() -> Result<Self> {
        let config_dir = directories::ProjectDirs::from("com", "nitro", "nitro")
            .ok_or_else(|| NitroError::Other("Could not determine config directory".into()))?;
        
        let taps_dir = config_dir.data_dir().join("taps");
        std::fs::create_dir_all(&taps_dir)?;
        
        let db_path = config_dir.data_dir().join("taps.db");
        let db = sled::Config::new()
            .path(&db_path)
            .mode(sled::Mode::HighThroughput)
            .flush_every_ms(Some(1000))
            .open()?;

        let mut manager = Self { taps_dir, db };
        
        // Add default Homebrew taps if not present
        manager.ensure_default_taps().await?;
        
        Ok(manager)
    }

    pub async fn add_tap(&self, name: &str, custom_url: Option<&str>) -> NitroResult<()> {
        // Check if tap already exists
        if self.db.contains_key(name)? {
            return Err(NitroError::TapError(format!("Tap {} already exists", name)));
        }

        let url = if let Some(url) = custom_url {
            url.to_string()
        } else {
            // Handle Homebrew tap naming convention
            // "homebrew/core" -> "https://github.com/Homebrew/homebrew-core.git"
            let parts: Vec<&str> = name.split('/').collect();
            if parts.len() == 2 {
                let org = parts[0];
                let repo = parts[1];
                
                // Special case for Homebrew organization (capitalize it)
                let org_name = if org.to_lowercase() == "homebrew" {
                    "Homebrew"
                } else {
                    org
                };
                
                // Homebrew taps follow the pattern: org/homebrew-repo
                format!("https://github.com/{}/homebrew-{}.git", org_name, repo)
            } else {
                // Fallback for non-standard tap names
                format!("https://github.com/{}.git", name)
            }
        };

        let tap_path = self.taps_dir.join(name.replace('/', "_"));
        
        // Clone the repository
        self.clone_tap(&url, &tap_path).await?;

        let tap = Tap {
            name: name.to_string(),
            url,
            path: tap_path,
            updated_at: Some(chrono::Utc::now()),
        };

        self.db.insert(name, serde_json::to_vec(&tap)?)?;
        Ok(())
    }

    pub async fn remove_tap(&self, name: &str) -> NitroResult<()> {
        let tap = self.get_tap(name)?;
        
        // Remove tap directory
        if tap.path.exists() {
            std::fs::remove_dir_all(&tap.path)?;
        }
        
        // Remove from database
        self.db.remove(name)?;
        
        Ok(())
    }

    pub async fn list_taps(&self) -> Result<Vec<Tap>> {
        let mut taps = Vec::new();
        
        for entry in self.db.iter() {
            let (_, value) = entry?;
            let tap: Tap = serde_json::from_slice(&value)?;
            taps.push(tap);
        }
        
        taps.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(taps)
    }

    pub async fn update_tap(&self, name: &str) -> NitroResult<()> {
        let mut tap = self.get_tap(name)?;
        
        // Pull latest changes
        self.pull_tap(&tap.path).await?;
        
        // Update timestamp
        tap.updated_at = Some(chrono::Utc::now());
        self.db.insert(name, serde_json::to_vec(&tap)?)?;
        
        Ok(())
    }

    pub async fn update_all_taps(&self) -> Result<()> {
        let taps = self.list_taps().await?;
        
        for tap in taps {
            if let Err(e) = self.update_tap(&tap.name).await {
                eprintln!("Failed to update tap {}: {}", tap.name, e);
            }
        }
        
        Ok(())
    }

    pub async fn find_formula(&self, name: &str) -> NitroResult<PathBuf> {
        // Search for formula in all taps
        for tap in self.list_taps().await? {
            // Check direct path first (legacy layout)
            let formula_path = tap.path.join("Formula").join(format!("{}.rb", name));
            if formula_path.exists() {
                return Ok(formula_path);
            }
            
            // Check alphabetical subdirectories (modern layout)
            let formula_dir = tap.path.join("Formula");
            if formula_dir.exists() {
                if let Ok(formula_path) = self.find_formula_recursive(&formula_dir, name) {
                    return Ok(formula_path);
                }
            }
            
            // Also check HomebrewFormula directory (some taps use this)
            let alt_path = tap.path.join("HomebrewFormula").join(format!("{}.rb", name));
            if alt_path.exists() {
                return Ok(alt_path);
            }
        }
        
        Err(NitroError::PackageNotFound(name.to_string()))
    }

    fn find_formula_recursive(&self, dir: &std::path::Path, name: &str) -> NitroResult<PathBuf> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively search subdirectories
                if let Ok(found) = self.find_formula_recursive(&path, name) {
                    return Ok(found);
                }
            } else if path.file_stem().and_then(|s| s.to_str()) == Some(name) &&
                      path.extension().and_then(|s| s.to_str()) == Some("rb") {
                return Ok(path);
            }
        }
        
        Err(NitroError::PackageNotFound(name.to_string()))
    }

    async fn ensure_default_taps(&mut self) -> Result<()> {
        // First, try to detect existing Homebrew taps
        if let Err(e) = self.import_homebrew_taps().await {
            eprintln!("Warning: Could not import Homebrew taps: {}", e);
        }
        
        // Add homebrew/core if not present
        if !self.db.contains_key("homebrew/core")? {
            if let Err(e) = self.add_tap("homebrew/core", None).await {
                eprintln!("Warning: Could not add homebrew/core tap: {}", e);
            }
        }
        
        Ok(())
    }

    pub async fn import_homebrew_taps(&mut self) -> Result<()> {
        // Detect Homebrew prefix
        let brew_prefix = if let Ok(prefix) = std::env::var("HOMEBREW_PREFIX") {
            PathBuf::from(prefix)
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            PathBuf::from("/opt/homebrew")
        } else {
            PathBuf::from("/usr/local")
        };

        let homebrew_taps_dir = brew_prefix.join("Homebrew/Library/Taps");
        
        if !homebrew_taps_dir.exists() {
            return Ok(());
        }

        // Iterate through Homebrew taps
        for org_entry in std::fs::read_dir(&homebrew_taps_dir)? {
            let org_entry = org_entry?;
            if !org_entry.path().is_dir() {
                continue;
            }

            let org_name = org_entry.file_name().to_string_lossy().to_string();
            
            for tap_entry in std::fs::read_dir(org_entry.path())? {
                let tap_entry = tap_entry?;
                if !tap_entry.path().is_dir() {
                    continue;
                }

                let tap_dir_name = tap_entry.file_name().to_string_lossy().to_string();
                
                // Convert directory name to tap name
                // e.g., "homebrew-core" -> "core"
                let tap_name = if let Some(stripped) = tap_dir_name.strip_prefix("homebrew-") {
                    format!("{}/{}", org_name, stripped)
                } else {
                    format!("{}/{}", org_name, tap_dir_name)
                };

                // Skip if already in our database
                if self.db.contains_key(&tap_name)? {
                    continue;
                }

                // Create a symlink to the existing tap
                let tap = Tap {
                    name: tap_name.clone(),
                    url: format!("file://{}", tap_entry.path().display()),
                    path: tap_entry.path(),
                    updated_at: Some(chrono::Utc::now()),
                };

                self.db.insert(&tap_name, serde_json::to_vec(&tap)?)?;
                println!("Imported existing Homebrew tap: {}", tap_name);
            }
        }

        Ok(())
    }

    async fn clone_tap(&self, url: &str, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(&["clone", "--depth", "1", url, path.to_str().unwrap()])
            .output()
            .await?;

        if !output.status.success() {
            return Err(NitroError::TapError(
                format!("Failed to clone tap: {}", String::from_utf8_lossy(&output.stderr))
            ).into());
        }

        Ok(())
    }

    async fn pull_tap(&self, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(&["pull", "--ff-only"])
            .current_dir(path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(NitroError::TapError(
                format!("Failed to update tap: {}", String::from_utf8_lossy(&output.stderr))
            ).into());
        }

        Ok(())
    }

    fn get_tap(&self, name: &str) -> NitroResult<Tap> {
        if let Some(data) = self.db.get(name)? {
            let tap: Tap = serde_json::from_slice(&data)?;
            Ok(tap)
        } else {
            Err(NitroError::TapError(format!("Tap {} not found", name)))
        }
    }
}

impl Drop for TapManager {
    fn drop(&mut self) {
        // Ensure the database is properly flushed before dropping
        let _ = self.db.flush();
    }
}
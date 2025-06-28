use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::core::NitroError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub path: PathBuf,
    pub size: u64,
    pub created_at: SystemTime,
    pub accessed_at: SystemTime,
    pub ttl: Option<Duration>,
}

pub struct CacheManager {
    cache_dir: PathBuf,
    max_size: u64,
    db: sled::Db,
}

impl CacheManager {
    pub fn new() -> Result<Self> {
        let config_dir = directories::ProjectDirs::from("com", "nitro", "nitro")
            .ok_or_else(|| NitroError::Other("Could not determine config directory".into()))?;
        
        let cache_dir = config_dir.cache_dir().to_path_buf();
        std::fs::create_dir_all(&cache_dir)?;
        
        let db_path = cache_dir.join("cache.db");
        let db = sled::Config::new()
            .path(&db_path)
            .mode(sled::Mode::HighThroughput)
            .flush_every_ms(Some(1000))
            .open()?;

        Ok(Self {
            cache_dir,
            max_size: 10 * 1024 * 1024 * 1024, // 10GB default
            db,
        })
    }

    pub async fn get(&self, key: &str) -> Option<PathBuf> {
        if let Ok(Some(data)) = self.db.get(key) {
            if let Ok(mut entry) = serde_json::from_slice::<CacheEntry>(&data) {
                // Check if entry has expired
                if let Some(ttl) = entry.ttl {
                    if entry.created_at.elapsed().unwrap_or_default() > ttl {
                        // Entry expired, remove it
                        let _ = self.remove(key).await;
                        return None;
                    }
                }
                
                // Update access time
                entry.accessed_at = SystemTime::now();
                if let Ok(updated) = serde_json::to_vec(&entry) {
                    let _ = self.db.insert(key, updated);
                }
                
                if entry.path.exists() {
                    return Some(entry.path);
                }
            }
        }
        None
    }

    pub async fn put(&self, key: &str, source: &Path, ttl: Option<Duration>) -> Result<PathBuf> {
        let dest = self.cache_dir.join("data").join(key);
        
        // Create parent directory
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Copy file to cache
        tokio::fs::copy(source, &dest).await?;
        
        // Get file size
        let metadata = tokio::fs::metadata(&dest).await?;
        let size = metadata.len();

        // Create cache entry
        let entry = CacheEntry {
            key: key.to_string(),
            path: dest.clone(),
            size,
            created_at: SystemTime::now(),
            accessed_at: SystemTime::now(),
            ttl,
        };

        // Store in database
        self.db.insert(key, serde_json::to_vec(&entry)?)?;

        // Check cache size and evict if necessary
        self.evict_if_needed().await?;

        Ok(dest)
    }

    pub async fn remove(&self, key: &str) -> Result<()> {
        if let Ok(Some(data)) = self.db.get(key) {
            if let Ok(entry) = serde_json::from_slice::<CacheEntry>(&data) {
                if entry.path.exists() {
                    tokio::fs::remove_file(&entry.path).await?;
                }
            }
        }
        self.db.remove(key)?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        // Remove all cached files
        let data_dir = self.cache_dir.join("data");
        if data_dir.exists() {
            tokio::fs::remove_dir_all(&data_dir).await?;
            tokio::fs::create_dir_all(&data_dir).await?;
        }
        
        // Clear database
        self.db.clear()?;
        
        Ok(())
    }

    pub async fn size(&self) -> Result<u64> {
        let mut total_size = 0u64;
        
        for entry in self.db.iter() {
            if let Ok((_, value)) = entry {
                if let Ok(cache_entry) = serde_json::from_slice::<CacheEntry>(&value) {
                    total_size += cache_entry.size;
                }
            }
        }
        
        Ok(total_size)
    }

    async fn evict_if_needed(&self) -> Result<()> {
        let current_size = self.size().await?;
        
        if current_size > self.max_size {
            // Collect all entries with access times
            let mut entries: Vec<(String, SystemTime, u64)> = Vec::new();
            
            for entry in self.db.iter() {
                if let Ok((key, value)) = entry {
                    if let Ok(cache_entry) = serde_json::from_slice::<CacheEntry>(&value) {
                        entries.push((
                            String::from_utf8_lossy(&key).to_string(),
                            cache_entry.accessed_at,
                            cache_entry.size,
                        ));
                    }
                }
            }
            
            // Sort by access time (LRU)
            entries.sort_by_key(|(_, accessed, _)| *accessed);
            
            // Remove oldest entries until we're under the limit
            let mut removed_size = 0u64;
            for (key, _, size) in entries {
                if current_size - removed_size <= self.max_size * 9 / 10 {
                    break;
                }
                
                self.remove(&key).await?;
                removed_size += size;
            }
        }
        
        Ok(())
    }
}

pub struct DownloadCache {
    cache_manager: CacheManager,
}

impl DownloadCache {
    pub fn new() -> Result<Self> {
        Ok(Self {
            cache_manager: CacheManager::new()?,
        })
    }

    pub async fn get_or_download<F>(
        &self,
        url: &str,
        downloader: F,
    ) -> Result<PathBuf>
    where
        F: std::future::Future<Output = Result<PathBuf>>,
    {
        let key = self.url_to_key(url);
        
        // Check cache first
        if let Some(path) = self.cache_manager.get(&key).await {
            return Ok(path);
        }
        
        // Download to temporary location
        let temp_path = downloader.await?;
        
        // Add to cache
        let cached_path = self.cache_manager.put(&key, &temp_path, None).await?;
        
        // Remove temporary file
        if temp_path != cached_path {
            let _ = tokio::fs::remove_file(&temp_path).await;
        }
        
        Ok(cached_path)
    }

    fn url_to_key(&self, url: &str) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..16]) // Use first 16 bytes for shorter keys
    }
}

impl Drop for CacheManager {
    fn drop(&mut self) {
        // Ensure the database is properly flushed before dropping
        let _ = self.db.flush();
    }
}
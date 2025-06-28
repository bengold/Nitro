use anyhow::Result;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::core::NitroError;

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Nitro Package Manager")
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        Ok(Self { client })
    }

    pub async fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
        println!("Downloading: {}", url);
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(NitroError::DownloadFailed(
                format!("HTTP {}: {}", response.status(), url)
            ).into());
        }
        
        // Check content type - warn if it's HTML (likely an error page)
        if let Some(content_type) = response.headers().get("content-type") {
            if let Ok(ct) = content_type.to_str() {
                if ct.contains("text/html") {
                    eprintln!("Warning: Server returned HTML content instead of expected archive");
                }
            }
        }

        let total_size = response.content_length().unwrap_or(0);

        let pb = if total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                    .progress_chars("#>-"),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {bytes} downloaded")?
            );
            pb
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = File::create(dest).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            
            downloaded += chunk.len() as u64;
            if total_size > 0 {
                pb.set_position(std::cmp::min(downloaded, total_size));
            } else {
                pb.set_position(downloaded);
            }
        }

        pb.finish_with_message("Download complete");
        Ok(())
    }

    pub async fn download_with_resume(&self, url: &str, dest: &Path) -> Result<()> {
        let mut downloaded = 0;
        
        // Check if file exists and get its size
        if dest.exists() {
            let metadata = tokio::fs::metadata(dest).await?;
            downloaded = metadata.len();
        }

        let client = &self.client;
        let response = if downloaded > 0 {
            // Resume download
            client
                .get(url)
                .header("Range", format!("bytes={}-", downloaded))
                .send()
                .await?
        } else {
            client.get(url).send().await?
        };

        if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(NitroError::DownloadFailed(
                format!("HTTP {}: {}", response.status(), url)
            ).into());
        }

        let total_size = if let Some(content_range) = response.headers().get("content-range") {
            // Extract total size from Content-Range header
            content_range
                .to_str()
                .ok()
                .and_then(|s| s.split('/').last())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0)
        } else {
            response.content_length().unwrap_or(0) + downloaded
        };

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                .progress_chars("#>-"),
        );
        pb.set_position(downloaded);

        // Open file in append mode if resuming
        let mut file = if downloaded > 0 {
            tokio::fs::OpenOptions::new()
                .append(true)
                .open(dest)
                .await?
        } else {
            File::create(dest).await?
        };

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete");
        Ok(())
    }

    pub async fn download_multiple(&self, downloads: Vec<(&str, &Path)>) -> Result<()> {
        use futures::future::join_all;

        let tasks: Vec<_> = downloads
            .into_iter()
            .map(|(url, path)| {
                let downloader = self.clone();
                let url = url.to_string();
                let path = path.to_path_buf();
                tokio::spawn(async move {
                    downloader.download_with_resume(&url, &path).await
                })
            })
            .collect();

        let results = join_all(tasks).await;
        
        for result in results {
            result??;
        }

        Ok(())
    }
}

impl Clone for Downloader {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}
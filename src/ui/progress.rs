use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::NitroError;

pub struct ProgressReporter {
    multi: Arc<Mutex<MultiProgress>>,
    bars: Arc<Mutex<std::collections::HashMap<String, ProgressBar>>>,
}

impl ProgressReporter {
    pub fn new() -> Self {
        Self {
            multi: Arc::new(Mutex::new(MultiProgress::new())),
            bars: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn start_package(&self, package_name: &str) {
        let package_name = package_name.to_string();
        let multi = self.multi.clone();
        let bars = self.bars.clone();
        
        tokio::spawn(async move {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .expect("Failed to set progress style")
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
            );
            pb.set_message(format!("Installing {}", package_name));
            
            let multi_guard = multi.lock().await;
            let pb = multi_guard.add(pb);
            drop(multi_guard);
            
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            
            let mut bars_guard = bars.lock().await;
            bars_guard.insert(package_name, pb);
        });
    }

    pub fn complete_package(&self, package_name: &str) {
        let package_name = package_name.to_string();
        let bars = self.bars.clone();
        
        tokio::spawn(async move {
            let mut bars_guard = bars.lock().await;
            if let Some(pb) = bars_guard.remove(&package_name) {
                pb.finish_with_message(format!("✓ {} installed successfully", package_name));
            }
        });
    }

    pub fn fail_package(&self, package_name: &str, error: &NitroError) {
        let package_name = package_name.to_string();
        let error_msg = error.to_string();
        let bars = self.bars.clone();
        
        tokio::spawn(async move {
            let mut bars_guard = bars.lock().await;
            if let Some(pb) = bars_guard.remove(&package_name) {
                pb.finish_with_message(format!("✗ {} failed: {}", package_name, error_msg));
            }
        });
    }

    pub fn update_package_progress(&self, package_name: &str, message: &str) {
        let package_name = package_name.to_string();
        let message = message.to_string();
        let bars = self.bars.clone();
        
        tokio::spawn(async move {
            let bars_guard = bars.lock().await;
            if let Some(pb) = bars_guard.get(&package_name) {
                pb.set_message(format!("{}: {}", package_name, message));
            }
        });
    }

    pub fn finish(&self) {
        let bars = self.bars.clone();
        
        tokio::spawn(async move {
            let mut bars_guard = bars.lock().await;
            for (_, pb) in bars_guard.drain() {
                pb.finish_and_clear();
            }
        });
    }
}

pub struct DownloadProgress {
    pb: ProgressBar,
}

impl DownloadProgress {
    pub fn new(total_size: u64, url: &str) -> Self {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
                .expect("Failed to set progress style")
                .progress_chars("#>-"),
        );
        pb.set_message(format!("Downloading {}", url));
        
        Self { pb }
    }

    pub fn update(&self, downloaded: u64) {
        self.pb.set_position(downloaded);
    }

    pub fn finish(&self) {
        self.pb.finish_with_message("Download complete");
    }

    pub fn fail(&self, error: &str) {
        self.pb.finish_with_message(format!("Download failed: {}", error));
    }
}

pub struct DependencyProgress {
    pb: ProgressBar,
}

impl DependencyProgress {
    pub fn new(total_deps: usize) -> Self {
        let pb = ProgressBar::new(total_deps as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} dependencies resolved")
                .expect("Failed to set progress style")
                .progress_chars("#>-"),
        );
        
        Self { pb }
    }

    pub fn update(&self, resolved: usize, current_dep: &str) {
        self.pb.set_position(resolved as u64);
        self.pb.set_message(format!("Resolving {}", current_dep));
    }

    pub fn finish(&self) {
        self.pb.finish_with_message("All dependencies resolved");
    }
}
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

use crate::core::{NitroError, NitroResult};
use crate::download::Downloader;
use super::formula::Formula;
use super::package::Package;

pub struct Installer {
    prefix: PathBuf,
    cellar: PathBuf,
    bin_dir: PathBuf,
    downloader: Downloader,
}

impl Installer {
    pub fn new() -> Result<Self> {
        let prefix = Self::get_prefix()?;
        let cellar = prefix.join("Cellar");
        let bin_dir = prefix.join("bin");

        // Create directories if they don't exist
        std::fs::create_dir_all(&cellar)?;
        std::fs::create_dir_all(&bin_dir)?;

        let downloader = Downloader::new()?;

        Ok(Self {
            prefix,
            cellar,
            bin_dir,
            downloader,
        })
    }

    pub async fn install(&self, formula: &Formula, build_from_source: bool) -> NitroResult<()> {
        // Try binary installation first unless building from source
        if !build_from_source && !formula.binary_packages.is_empty() {
            match self.install_binary(formula).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("Binary installation failed: {}. Falling back to source installation.", e);
                }
            }
        }

        // Fall back to source installation
        self.install_from_source(formula).await
    }

    pub async fn uninstall(&self, package: &Package) -> NitroResult<()> {
        let install_path = package.install_path.as_ref()
            .ok_or_else(|| NitroError::Other("Package install path not found".into()))?;

        // Remove symlinks
        self.remove_symlinks(&package.name).await?;

        // Remove installation directory
        if install_path.exists() {
            fs::remove_dir_all(install_path).await?;
        }

        Ok(())
    }

    pub fn get_install_path(&self, name: &str) -> PathBuf {
        self.cellar.join(name)
    }

    async fn install_binary(&self, formula: &Formula) -> NitroResult<()> {
        // Get platform-specific binary package
        let platform = self.get_platform();
        let arch = self.get_arch();
        
        let binary_pkg = formula.binary_packages.iter()
            .find(|pkg| pkg.platform == platform && pkg.arch == arch)
            .ok_or_else(|| NitroError::Other("No binary package available for this platform".into()))?;

        // Download binary package
        let temp_dir = tempfile::tempdir()?;
        let download_path = temp_dir.path().join("package.tar.gz");
        
        self.downloader.download_file(&binary_pkg.url, &download_path).await?;

        // Verify checksum
        self.verify_checksum(&download_path, &binary_pkg.sha256)?;

        // Extract to cellar
        let install_path = self.cellar.join(&formula.name).join(&formula.version);
        std::fs::create_dir_all(&install_path)?;
        
        self.extract_tarball(&download_path, &install_path)?;

        // Create symlinks
        self.create_symlinks(&formula.name, &formula.version).await?;

        Ok(())
    }

    async fn install_from_source(&self, formula: &Formula) -> NitroResult<()> {
        if formula.sources.is_empty() {
            return Err(NitroError::Other("No source URL found".into()));
        }

        let source = &formula.sources[0];
        
        // Download source
        let temp_dir = tempfile::tempdir()?;
        
        // Determine file extension from URL
        let file_name = source.url.split('/').last().unwrap_or("source.tar.gz");
        let download_path = temp_dir.path().join(file_name);
        
        self.downloader.download_file(&source.url, &download_path).await?;

        // Verify checksum
        self.verify_checksum(&download_path, &source.sha256)?;

        // Extract source
        let build_dir = temp_dir.path().join("build");
        std::fs::create_dir_all(&build_dir)?;
        self.extract_tarball(&download_path, &build_dir)?;

        // Find extracted directory
        let extracted_dir = self.find_extracted_dir(&build_dir)?;

        // Run install script
        if let Some(install_script) = &formula.install_script {
            self.run_install_script(&extracted_dir, install_script, formula).await?;
        } else {
            // Default configure, make, make install
            self.run_default_install(&extracted_dir, formula).await?;
        }

        // Create symlinks
        self.create_symlinks(&formula.name, &formula.version).await?;

        Ok(())
    }

    async fn run_install_script(&self, build_dir: &Path, script: &str, formula: &Formula) -> Result<()> {
        let install_path = self.cellar.join(&formula.name).join(&formula.version);
        std::fs::create_dir_all(&install_path)?;

        // Set up environment variables
        std::env::set_var("PREFIX", &install_path);
        std::env::set_var("HOMEBREW_PREFIX", &self.prefix);

        // Parse and execute install script commands
        // This is simplified - in reality we'd need a proper Ruby interpreter
        for line in script.lines() {
            let line = line.trim();
            if line.starts_with("system") {
                // Extract command from system call
                if let Some(cmd) = self.extract_system_command(line) {
                    self.run_command(&cmd, build_dir)?;
                }
            }
        }

        Ok(())
    }

    async fn run_default_install(&self, build_dir: &Path, formula: &Formula) -> Result<()> {
        let install_path = self.cellar.join(&formula.name).join(&formula.version);
        let prefix_arg = format!("--prefix={}", install_path.display());

        // Configure
        if build_dir.join("configure").exists() {
            self.run_command(&format!("./configure {}", prefix_arg), build_dir)?;
        }

        // Make
        self.run_command("make", build_dir)?;

        // Make install
        self.run_command("make install", build_dir)?;

        Ok(())
    }

    async fn create_symlinks(&self, name: &str, version: &str) -> Result<()> {
        let install_path = self.cellar.join(name).join(version);
        let bin_path = install_path.join("bin");

        if bin_path.exists() {
            for entry in std::fs::read_dir(&bin_path)? {
                let entry = entry?;
                let file_name = entry.file_name();
                let src = entry.path();
                let dst = self.bin_dir.join(&file_name);

                // Remove existing symlink if it exists
                if dst.exists() {
                    std::fs::remove_file(&dst)?;
                }

                // Create new symlink
                std::os::unix::fs::symlink(&src, &dst)?;
            }
        }

        Ok(())
    }

    async fn remove_symlinks(&self, name: &str) -> Result<()> {
        // Find and remove all symlinks pointing to this package
        for entry in std::fs::read_dir(&self.bin_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_symlink() {
                if let Ok(target) = std::fs::read_link(&path) {
                    if target.to_string_lossy().contains(&format!("Cellar/{}/", name)) {
                        std::fs::remove_file(&path)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn verify_checksum(&self, file_path: &Path, expected_sha256: &str) -> Result<()> {
        use sha2::{Sha256, Digest};
        use std::io::Read;

        let mut file = std::fs::File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let result = hasher.finalize();
        let calculated = hex::encode(result);

        if calculated != expected_sha256 {
            return Err(NitroError::Other(
                format!("Checksum mismatch: expected {}, got {}", expected_sha256, calculated)
            ).into());
        }

        Ok(())
    }

    fn extract_tarball(&self, tarball: &Path, destination: &Path) -> Result<()> {
        use tar::Archive;
        use flate2::read::GzDecoder;
        use xz2::read::XzDecoder;

        // Check if file exists and has content
        let metadata = std::fs::metadata(tarball)?;
        if metadata.len() == 0 {
            return Err(NitroError::Other("Downloaded file is empty".into()).into());
        }

        // Determine compression type by extension
        let extension = tarball.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        eprintln!("DEBUG: Extracting {} with extension: {}", tarball.display(), extension);

        let file = std::fs::File::open(tarball)?;
        
        match extension {
            "gz" => {
                let decoder = GzDecoder::new(file);
                let mut archive = Archive::new(decoder);
                archive.unpack(destination).map_err(|e| {
                    anyhow::Error::from(NitroError::Other(format!("Failed to extract tar.gz archive: {}", e)))
                })?;
            }
            "xz" => {
                let decoder = XzDecoder::new(file);
                let mut archive = Archive::new(decoder);
                archive.unpack(destination).map_err(|e| {
                    anyhow::Error::from(NitroError::Other(format!("Failed to extract tar.xz archive: {}", e)))
                })?;
            }
            "bz2" => {
                use bzip2::read::BzDecoder;
                let decoder = BzDecoder::new(file);
                let mut archive = Archive::new(decoder);
                archive.unpack(destination).map_err(|e| {
                    anyhow::Error::from(NitroError::Other(format!("Failed to extract tar.bz2 archive: {}", e)))
                })?;
            }
            _ => {
                // Try to detect by reading file header
                let mut file = std::fs::File::open(tarball)?;
                let mut header = [0u8; 6];
                use std::io::Read;
                file.read_exact(&mut header)?;
                
                // Reset file
                let file = std::fs::File::open(tarball)?;
                
                if header[0..2] == [0x1f, 0x8b] {
                    // gzip
                    let decoder = GzDecoder::new(file);
                    let mut archive = Archive::new(decoder);
                    archive.unpack(destination)?;
                } else if header[0..6] == [0xfd, 0x37, 0x7a, 0x58, 0x5a, 0x00] {
                    // xz
                    let decoder = XzDecoder::new(file);
                    let mut archive = Archive::new(decoder);
                    archive.unpack(destination)?;
                } else {
                    return Err(NitroError::Other(
                        "Unknown archive format. Supported formats: .tar.gz, .tar.xz, .tar.bz2".into()
                    ).into());
                }
            }
        }
        
        Ok(())
    }

    fn find_extracted_dir(&self, build_dir: &Path) -> Result<PathBuf> {
        // Find the first directory in the build directory
        for entry in std::fs::read_dir(build_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                return Ok(entry.path());
            }
        }
        Err(NitroError::Other("No extracted directory found".into()).into())
    }

    fn run_command(&self, command: &str, cwd: &Path) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let output = Command::new(parts[0])
            .args(&parts[1..])
            .current_dir(cwd)
            .output()?;

        if !output.status.success() {
            return Err(NitroError::Other(
                format!("Command failed: {}", String::from_utf8_lossy(&output.stderr))
            ).into());
        }

        Ok(())
    }

    fn extract_system_command(&self, line: &str) -> Option<String> {
        // Extract command from Ruby system call
        // system "command", "arg1", "arg2"
        // or system "./configure", "--prefix=#{prefix}"
        
        // This is a simplified extraction
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start+1..].find('"') {
                return Some(line[start+1..start+1+end].to_string());
            }
        }
        None
    }

    fn get_platform(&self) -> String {
        if cfg!(target_os = "macos") {
            "macos".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn get_arch(&self) -> String {
        if cfg!(target_arch = "x86_64") {
            "x64".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "arm64".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn get_prefix() -> Result<PathBuf> {
        // Check for HOMEBREW_PREFIX environment variable first
        if let Ok(prefix) = std::env::var("HOMEBREW_PREFIX") {
            return Ok(PathBuf::from(prefix));
        }

        // Detect Homebrew installation
        // Apple Silicon Macs use /opt/homebrew
        // Intel Macs and Linux use /usr/local
        let apple_silicon_path = PathBuf::from("/opt/homebrew");
        let intel_path = PathBuf::from("/usr/local");
        
        // Check if running on Apple Silicon
        if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            if apple_silicon_path.join("bin/brew").exists() {
                return Ok(apple_silicon_path);
            }
        }
        
        // Check standard Homebrew location
        if intel_path.join("bin/brew").exists() {
            return Ok(intel_path);
        }
        
        // Check Apple Silicon location even on Intel (user might have it there)
        if apple_silicon_path.join("bin/brew").exists() {
            return Ok(apple_silicon_path);
        }
        
        // Default to standard location
        Ok(intel_path)
    }
}
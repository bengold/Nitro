use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct HomebrewArgs {
    #[command(subcommand)]
    pub command: HomebrewCommands,
}

#[derive(Subcommand)]
pub enum HomebrewCommands {
    /// Import existing Homebrew taps and formulae
    Import,
    /// Show Homebrew compatibility status
    Status,
}

pub async fn execute(args: HomebrewArgs) -> Result<()> {
    match args.command {
        HomebrewCommands::Import => import_homebrew().await,
        HomebrewCommands::Status => show_status().await,
    }
}

async fn import_homebrew() -> Result<()> {
    use crate::core::tap::TapManager;
    
    println!("🔍 Detecting Homebrew installation...");
    
    // Detect Homebrew prefix
    let brew_prefix = if let Ok(prefix) = std::env::var("HOMEBREW_PREFIX") {
        std::path::PathBuf::from(prefix)
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        std::path::PathBuf::from("/opt/homebrew")
    } else {
        std::path::PathBuf::from("/usr/local")
    };
    
    if !brew_prefix.join("bin/brew").exists() {
        println!("❌ Homebrew not found at {}", brew_prefix.display());
        println!("   Install Homebrew from https://brew.sh");
        return Ok(());
    }
    
    println!("✅ Found Homebrew at: {}", brew_prefix.display());
    
    // Import taps
    println!("\n📦 Importing Homebrew taps...");
    let mut tap_manager = TapManager::new().await?;
    tap_manager.import_homebrew_taps().await?;
    
    // List imported taps
    let taps = tap_manager.list_taps().await?;
    println!("\n✅ Imported {} tap(s):", taps.len());
    for tap in &taps {
        println!("   • {}", tap.name);
    }
    
    // Skip search index building for now as it's too slow with 7000+ formulae
    println!("\n⚠️  Search index building skipped due to large number of formulae.");
    println!("   Run 'nitro update --formulae' to build the search index later.");
    
    println!("\n✨ Homebrew import complete!");
    println!("\nYou can now:");
    println!("  • Search packages: nitro search <name>");
    println!("  • Install packages: nitro install <name>");
    println!("  • List packages: nitro list");
    
    Ok(())
}

async fn show_status() -> Result<()> {
    println!("🍺 Homebrew Compatibility Status");
    println!("================================");
    
    // Check Homebrew installation
    let brew_exists = std::process::Command::new("brew")
        .arg("--version")
        .output()
        .is_ok();
    
    if brew_exists {
        let output = std::process::Command::new("brew")
            .arg("--prefix")
            .output()?;
        let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("✅ Homebrew installed at: {}", prefix);
        
        // Show Homebrew version
        let version_output = std::process::Command::new("brew")
            .arg("--version")
            .output()?;
        let version = String::from_utf8_lossy(&version_output.stdout).trim().to_string();
        println!("   Version: {}", version);
    } else {
        println!("❌ Homebrew not installed");
    }
    
    // Check Nitro configuration
    println!("\n📦 Nitro Configuration:");
    println!("   Data directory: {}", 
        directories::ProjectDirs::from("com", "nitro", "nitro")
            .map(|d| d.data_dir().display().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    );
    
    // Check taps
    use crate::core::tap::TapManager;
    let tap_manager = TapManager::new().await?;
    let taps = tap_manager.list_taps().await?;
    println!("   Configured taps: {}", taps.len());
    
    // Check if we can use system Cellar
    let cellar_path = if brew_exists {
        std::path::PathBuf::from(
            String::from_utf8_lossy(
                &std::process::Command::new("brew")
                    .arg("--cellar")
                    .output()?
                    .stdout
            ).trim()
        )
    } else {
        std::path::PathBuf::from("/usr/local/Cellar")
    };
    
    if cellar_path.exists() {
        let writable = std::fs::metadata(&cellar_path)
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false);
        
        if writable {
            println!("✅ Cellar directory writable: {}", cellar_path.display());
        } else {
            println!("⚠️  Cellar directory read-only: {}", cellar_path.display());
            println!("   You may need to use sudo for installations");
        }
    } else {
        println!("❌ Cellar directory not found");
    }
    
    Ok(())
}
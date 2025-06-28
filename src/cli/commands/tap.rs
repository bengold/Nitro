use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct TapArgs {
    #[command(subcommand)]
    pub command: TapCommands,
}

#[derive(Subcommand)]
pub enum TapCommands {
    /// Add a new tap
    Add {
        /// Tap name (e.g., homebrew/core)
        name: String,
        /// Custom URL (optional)
        #[arg(long)]
        url: Option<String>,
    },
    /// Remove a tap
    Remove {
        /// Tap name to remove
        name: String,
    },
    /// List all taps
    List,
    /// Update taps
    Update {
        /// Specific tap to update (updates all if not specified)
        name: Option<String>,
    },
}

pub async fn execute(args: TapArgs) -> Result<()> {
    use crate::core::tap::TapManager;
    use crate::ui::display;

    let tap_manager = TapManager::new().await?;

    match args.command {
        TapCommands::Add { name, url } => {
            println!("Adding tap {}...", name);
            tap_manager.add_tap(&name, url.as_deref()).await?;
            println!("Successfully added tap {}", name);
        }
        TapCommands::Remove { name } => {
            println!("Removing tap {}...", name);
            tap_manager.remove_tap(&name).await?;
            println!("Successfully removed tap {}", name);
        }
        TapCommands::List => {
            let taps = tap_manager.list_taps().await?;
            if taps.is_empty() {
                println!("No taps configured");
            } else {
                display::show_tap_list(&taps);
            }
        }
        TapCommands::Update { name } => {
            if let Some(tap_name) = name {
                println!("Updating tap {}...", tap_name);
                tap_manager.update_tap(&tap_name).await?;
                println!("Successfully updated tap {}", tap_name);
            } else {
                println!("Updating all taps...");
                tap_manager.update_all_taps().await?;
                println!("Successfully updated all taps");
            }
        }
    }

    Ok(())
}
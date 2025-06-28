use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod cli;
mod core;
mod download;
mod cache;
mod search;
mod ui;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle commands
    match cli.command {
        Commands::Install(args) => {
            cli::commands::install::execute(args).await?;
        }
        Commands::Uninstall(args) => {
            cli::commands::uninstall::execute(args).await?;
        }
        Commands::Search(args) => {
            cli::commands::search::execute(args).await?;
        }
        Commands::List(args) => {
            cli::commands::list::execute(args).await?;
        }
        Commands::Update(args) => {
            cli::commands::update::execute(args).await?;
        }
        Commands::Info(args) => {
            cli::commands::info::execute(args).await?;
        }
        Commands::Tap(args) => {
            cli::commands::tap::execute(args).await?;
        }
    }

    Ok(())
}
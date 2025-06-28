pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nitro")]
#[command(about = "A high-performance package manager leveraging Homebrew formulae", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Increase logging verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all output except errors
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a package
    Install(commands::install::InstallArgs),

    /// Uninstall a package
    Uninstall(commands::uninstall::UninstallArgs),

    /// Search for packages
    Search(commands::search::SearchArgs),

    /// List installed packages
    List(commands::list::ListArgs),

    /// Update packages or formulae
    Update(commands::update::UpdateArgs),

    /// Show information about a package
    Info(commands::info::InfoArgs),

    /// Manage taps (formula repositories)
    Tap(commands::tap::TapArgs),

    /// Homebrew compatibility commands
    Homebrew(commands::homebrew::HomebrewArgs),
}
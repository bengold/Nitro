use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct ListArgs {
    /// Show all versions
    #[arg(long)]
    pub versions: bool,

    /// Show only explicitly installed packages
    #[arg(long)]
    pub installed: bool,

    /// Show package sizes
    #[arg(long)]
    pub size: bool,

    /// Filter by prefix
    #[arg(short, long)]
    pub prefix: Option<String>,
}

pub async fn execute(args: ListArgs) -> Result<()> {
    use crate::core::package::PackageManager;
    use crate::ui::display;

    let package_manager = PackageManager::new().await?;
    let packages = package_manager.list_installed(&args).await?;

    if packages.is_empty() {
        println!("No packages installed");
    } else {
        display::show_package_list(&packages, &args);
    }

    Ok(())
}
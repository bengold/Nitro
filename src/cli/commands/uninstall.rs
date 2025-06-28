use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct UninstallArgs {
    /// Package name(s) to uninstall
    #[arg(required = true)]
    pub packages: Vec<String>,

    /// Force uninstallation
    #[arg(short, long)]
    pub force: bool,

    /// Remove all versions
    #[arg(long)]
    pub all_versions: bool,
}

pub async fn execute(args: UninstallArgs) -> Result<()> {
    use crate::core::package::PackageManager;
    use crate::ui::progress::ProgressReporter;

    let progress = ProgressReporter::new();
    let package_manager = PackageManager::new().await?;

    for package_name in &args.packages {
        progress.start_package(package_name);
        
        match package_manager.uninstall(package_name, &args).await {
            Ok(_) => progress.complete_package(package_name),
            Err(e) => {
                progress.fail_package(package_name, &e);
                if !args.force {
                    return Err(e);
                }
            }
        }
    }

    progress.finish();
    Ok(())
}
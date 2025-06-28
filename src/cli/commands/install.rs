use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct InstallArgs {
    /// Package name(s) to install
    #[arg(required = true)]
    pub packages: Vec<String>,

    /// Force installation (overwrite existing)
    #[arg(short, long)]
    pub force: bool,

    /// Skip binary packages and build from source
    #[arg(long)]
    pub build_from_source: bool,

    /// Don't install dependencies
    #[arg(long)]
    pub only_deps: bool,

    /// Install only dependencies
    #[arg(long)]
    pub skip_deps: bool,

    /// Use specific version
    #[arg(short, long)]
    pub version: Option<String>,

    /// Run installation in verbose mode
    #[arg(long)]
    pub debug: bool,
}

pub async fn execute(args: InstallArgs) -> Result<()> {
    use crate::core::package::PackageManager;
    use crate::ui::progress::ProgressReporter;

    let progress = ProgressReporter::new();
    let package_manager = PackageManager::new().await?;

    for package_name in &args.packages {
        progress.start_package(package_name);
        
        match package_manager.install(package_name, &args).await {
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
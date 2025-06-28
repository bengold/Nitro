use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct InfoArgs {
    /// Package name
    #[arg(required = true)]
    pub package: String,

    /// Show JSON output
    #[arg(long)]
    pub json: bool,

    /// Show all versions
    #[arg(long)]
    pub all_versions: bool,
}

pub async fn execute(args: InfoArgs) -> Result<()> {
    use crate::core::formula::FormulaManager;
    use crate::ui::display;

    let formula_manager = FormulaManager::new().await?;
    
    // Try common aliases first
    let package_name = match args.package.as_str() {
        "python" => "python@3.12",
        "python3" => "python@3.12",
        "ruby" => "ruby@3.3",
        "node" => "node@22",
        "nodejs" => "node@22",
        "postgresql" => "postgresql@17",
        "postgres" => "postgresql@17",
        "mysql" => "mysql@9.1",
        _ => &args.package,
    };
    
    let formula = match formula_manager.get_formula(package_name).await {
        Ok(f) => f,
        Err(e) if package_name != args.package => {
            // If alias failed, try original name
            formula_manager.get_formula(&args.package).await?
        }
        Err(e) => return Err(e.into()),
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&formula)?);
    } else {
        display::show_formula_info(&formula, &args);
    }

    Ok(())
}
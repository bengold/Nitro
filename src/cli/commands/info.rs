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
    let formula = formula_manager.get_formula(&args.package).await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&formula)?);
    } else {
        display::show_formula_info(&formula, &args);
    }

    Ok(())
}
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct SearchArgs {
    /// Search query
    #[arg(required = true)]
    pub query: String,

    /// Search in descriptions as well
    #[arg(short, long)]
    pub description: bool,

    /// Fuzzy search
    #[arg(short, long)]
    pub fuzzy: bool,

    /// Maximum number of results
    #[arg(short, long, default_value = "20")]
    pub limit: usize,
}

pub async fn execute(args: SearchArgs) -> Result<()> {
    use crate::search::SearchEngine;
    use crate::ui::display;

    let search_engine = SearchEngine::new().await?;
    let results = search_engine.search(&args.query, &args).await?;

    if results.is_empty() {
        println!("No packages found matching '{}'", args.query);
    } else {
        display::show_search_results(&results);
    }

    Ok(())
}
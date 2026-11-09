use clap::Parser;
use financial_presentation::{cli, persistence};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    let store = persistence::JsonStore::new("transactions.json");
    cli::run(cli, store)?;
    Ok(())
}

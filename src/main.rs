use clap::{CommandFactory, Parser};
use financial_infrastructure::JsonStore;
use financial_presentation::cli;
use std::io::{self, IsTerminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    if cli.command.is_none() && (!io::stdin().is_terminal() || !io::stdout().is_terminal()) {
        cli::Cli::command().print_help()?;
        return Ok(());
    }

    let store = JsonStore::new("transactions.json");
    cli::run(cli, store)?;
    Ok(())
}

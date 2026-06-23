mod cli;

use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Timeline => herdr_insight_tui::timeline::run()?,
    }
    Ok(())
}

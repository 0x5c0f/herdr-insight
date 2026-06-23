use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "herdr-insight", about = "Agent insight tools for herdr")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the agent status timeline TUI
    Timeline,
}

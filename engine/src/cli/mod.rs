pub mod commands;
pub mod parser;

use clap::Parser;
use commands::Command;
use parser::{Cli, Commands};

pub fn parse() -> Command {
    let cli = Cli::parse();
    match cli.command {
        Commands::Version => Command::Version,
        Commands::Doctor { preflight } => Command::Doctor { preflight },
    }
}

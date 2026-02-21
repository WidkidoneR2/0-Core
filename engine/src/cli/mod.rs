pub mod commands;
pub mod parser;

use clap::Parser;
use commands::{Command, LinkCommand};
use parser::{Cli, Commands, LinkCommands};

pub fn parse() -> Command {
    let cli = Cli::parse();
    match cli.command {
        Commands::Version => Command::Version,
        Commands::Doctor { preflight } => Command::Doctor { preflight },
        Commands::Link { command } => Command::Link(match command {
            LinkCommands::Status { json } => LinkCommand::Status { json },
            LinkCommands::List => LinkCommand::List,
            LinkCommands::Audit => LinkCommand::Audit,
        }),
        Commands::Zone {
            icon,
            label,
            json,
            health,
        } => Command::Zone {
            icon,
            label,
            json,
            health,
        },
    }
}

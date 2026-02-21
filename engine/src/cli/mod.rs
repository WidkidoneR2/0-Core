pub mod commands;
pub mod parser;

use clap::Parser;
use commands::{Command, IntentCommand, LinkCommand, ProfileCommand};
use parser::{Cli, Commands, IntentCommands, LinkCommands, ProfileCommands};

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
        Commands::Intent { command } => Command::Intent(match command {
            IntentCommands::List {
                planned,
                active,
                complete,
            } => IntentCommand::List {
                planned,
                active,
                complete,
            },
            IntentCommands::Show { id } => IntentCommand::Show { id },
            IntentCommands::Search { term } => IntentCommand::Search { term },
            IntentCommands::Stats => IntentCommand::Stats,
            IntentCommands::Validate => IntentCommand::Validate,
        }),
        Commands::Profile { command } => Command::Profile(match command {
            ProfileCommands::List => ProfileCommand::List,
            ProfileCommands::Status => ProfileCommand::Status,
            ProfileCommands::Switch { name } => ProfileCommand::Switch { name },
            ProfileCommands::History => ProfileCommand::History,
            ProfileCommands::Health => ProfileCommand::Health,
        }),
    }
}

pub mod commands;
pub mod parser;

use clap::Parser;
use commands::{
    Command, IntentCommand, LinkCommand, ProfileCommand, SandboxCommand, SecurityCommand,
};
use parser::{
    Cli, Commands, IntentCommands, LinkCommands, ProfileCommands, SandboxCommands, SecurityCommands,
};

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
        Commands::Security { command } => Command::Security(match command {
            SecurityCommands::Scan => SecurityCommand::Scan,
            SecurityCommands::Report { all } => SecurityCommand::Report { all },
            SecurityCommands::Show { id } => SecurityCommand::Show { id },
            SecurityCommands::History => SecurityCommand::History,
        }),
        Commands::Sandbox { command } => Command::Sandbox(match command {
            SandboxCommands::Run { args } => SandboxCommand::Run { args },
            SandboxCommands::Diff => SandboxCommand::Diff,
            SandboxCommands::Status => SandboxCommand::Status,
            SandboxCommands::Clear => SandboxCommand::Clear,
            SandboxCommands::Snapshot { target, name } => SandboxCommand::Snapshot { target, name },
            SandboxCommands::Restore { name } => SandboxCommand::Restore { name },
            SandboxCommands::Snapshots => SandboxCommand::Snapshots,
        }),
        Commands::Fetch { health_check } => Command::Fetch { health_check },
    }
}

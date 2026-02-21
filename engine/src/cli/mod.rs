pub mod commands;
pub mod parser;

use clap::Parser;
use commands::{
    Command, GitCommand, IntentCommand, LinkCommand, NotifyCommand, ProfileCommand, ReleaseCommand,
    SandboxCommand, SecurityCommand, WorkspaceCommand,
};
use parser::{
    Cli, Commands, GitCommands, IntentCommands, LinkCommands, NotifyCommands, ProfileCommands,
    ReleaseCommands, SandboxCommands, SecurityCommands, WorkspaceCommands,
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
        Commands::Git { command } => Command::Git(match command {
            GitCommands::Status => GitCommand::Status,
            GitCommands::Risk => GitCommand::Risk,
            GitCommands::Log { n } => GitCommand::Log { n },
            GitCommands::Verify => GitCommand::Verify,
            GitCommands::Commit { args } => GitCommand::Delegate {
                subcmd: "commit".to_string(),
                args,
            },
            GitCommands::Sync { args } => GitCommand::Delegate {
                subcmd: "sync".to_string(),
                args,
            },
            GitCommands::Quick { args } => GitCommand::Delegate {
                subcmd: "quick".to_string(),
                args,
            },
            GitCommands::Branch { args } => GitCommand::Delegate {
                subcmd: "branch".to_string(),
                args,
            },
            GitCommands::InstallHooks => GitCommand::Delegate {
                subcmd: "install-hooks".to_string(),
                args: vec![],
            },
            GitCommands::RemoveHooks => GitCommand::Delegate {
                subcmd: "remove-hooks".to_string(),
                args: vec![],
            },
        }),
        Commands::Workspace { command } => Command::Workspace(match command {
            WorkspaceCommands::View {
                active,
                summary,
                json,
            } => WorkspaceCommand::View {
                active,
                summary,
                json,
            },
            WorkspaceCommands::Recent {
                range,
                limit,
                full_paths,
            } => WorkspaceCommand::Recent {
                range,
                limit,
                full_paths,
            },
            WorkspaceCommands::Fm { args } => WorkspaceCommand::Fm { args },
        }),
        Commands::Release { command } => Command::Release(match command {
            ReleaseCommands::Get { package } => ReleaseCommand::Get { package },
            ReleaseCommands::BumpTool { args } => ReleaseCommand::BumpTool { args },
            ReleaseCommands::BumpSystem { dry_run } => ReleaseCommand::BumpSystem { dry_run },
        }),
        Commands::Notify { command } => Command::Notify(match command {
            NotifyCommands::Send {
                summary,
                body,
                urgency,
            } => NotifyCommand::Send {
                summary,
                body,
                urgency,
            },
            NotifyCommands::Status => NotifyCommand::Status,
        }),
    }
}

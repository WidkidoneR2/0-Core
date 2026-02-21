use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "core")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "0-Core v2 — single orchestrator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Version,
    Doctor {
        #[arg(long)]
        preflight: bool,
    },
    Link {
        #[command(subcommand)]
        command: LinkCommands,
    },
    Zone {
        #[arg(long)]
        icon: bool,
        #[arg(long)]
        label: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        health: bool,
    },
    Intent {
        #[command(subcommand)]
        command: IntentCommands,
    },
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    Security {
        #[command(subcommand)]
        command: SecurityCommands,
    },
    /// Controlled experimentation environment
    Sandbox {
        #[command(subcommand)]
        command: SandboxCommands,
    },
}

#[derive(Subcommand)]
pub enum LinkCommands {
    Status {
        #[arg(long)]
        json: bool,
    },
    List,
    Audit,
}

#[derive(Subcommand)]
pub enum IntentCommands {
    List {
        #[arg(long)]
        planned: bool,
        #[arg(long)]
        active: bool,
        #[arg(long)]
        complete: bool,
    },
    Show {
        id: String,
    },
    Search {
        term: String,
    },
    Stats,
    Validate,
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    List,
    Status,
    Switch { name: String },
    History,
    Health,
}

#[derive(Subcommand)]
pub enum SecurityCommands {
    Scan,
    Report {
        #[arg(long)]
        all: bool,
    },
    Show {
        id: String,
    },
    History,
}

#[derive(Subcommand)]
pub enum SandboxCommands {
    /// Run a command in the sandbox
    Run { args: Vec<String> },
    /// Show what changed in last session
    Diff,
    /// Show current sandbox status
    Status,
    /// Clear sandbox session state
    Clear,
    /// Create a reflink snapshot
    Snapshot {
        #[arg(long)]
        target: String,
        #[arg(long)]
        name: String,
    },
    /// Restore from a snapshot
    Restore { name: String },
    /// List available snapshots
    Snapshots,
}

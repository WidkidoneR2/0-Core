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
    /// Show version information
    Version,
    /// Run health checks
    Doctor {
        #[arg(long)]
        preflight: bool,
    },
    /// Manage stow symlinks
    Link {
        #[command(subcommand)]
        command: LinkCommands,
    },
    /// Detect current filesystem zone
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
    /// Manage the intent ledger
    Intent {
        #[command(subcommand)]
        command: IntentCommands,
    },
    /// Manage system profiles
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
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
    /// List available profiles
    List,
    /// Show current profile status
    Status,
    /// Switch to a profile
    Switch { name: String },
    /// Show profile switch history
    History,
    /// Run profile health check
    Health,
}

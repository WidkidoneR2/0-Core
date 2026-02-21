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
}

#[derive(Subcommand)]
pub enum LinkCommands {
    /// Show status of all stow packages
    Status {
        #[arg(long)]
        json: bool,
    },
    /// List all stow packages
    List,
    /// Audit link health
    Audit,
}

#[derive(Subcommand)]
pub enum IntentCommands {
    /// List intents
    List {
        #[arg(long)]
        planned: bool,
        #[arg(long)]
        active: bool,
        #[arg(long)]
        complete: bool,
    },
    /// Show a specific intent
    Show { id: String },
    /// Search intents
    Search { term: String },
    /// Show intent statistics
    Stats,
    /// Validate intent ledger integrity
    Validate,
}

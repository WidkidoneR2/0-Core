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

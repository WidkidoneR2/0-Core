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
    Sandbox {
        #[command(subcommand)]
        command: SandboxCommands,
    },
    Fetch {
        #[arg(long)]
        health_check: bool,
    },
    Git {
        #[command(subcommand)]
        command: GitCommands,
    },
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
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
    Run {
        args: Vec<String>,
    },
    Diff,
    Status,
    Clear,
    Snapshot {
        #[arg(long)]
        target: String,
        #[arg(long)]
        name: String,
    },
    Restore {
        name: String,
    },
    Snapshots,
}

#[derive(Subcommand)]
pub enum GitCommands {
    Status,
    Risk,
    Log {
        #[arg(short, long, default_value = "10")]
        n: u32,
    },
    Verify,
    Commit {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    Sync {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    Quick {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    Branch {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    InstallHooks,
    RemoveHooks,
}

#[derive(Subcommand)]
pub enum WorkspaceCommands {
    /// Show Sway workspace info
    View {
        #[arg(long)]
        active: bool,
        #[arg(long)]
        summary: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show recently modified files
    Recent {
        #[arg(default_value = "today")]
        range: String,
        #[arg(short, long, default_value = "10")]
        limit: u32,
        #[arg(long)]
        full_paths: bool,
    },
    /// Open file manager
    Fm {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

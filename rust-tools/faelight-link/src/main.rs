use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod conflict;
mod intelligence;
mod link;
mod package;
mod paths;

#[derive(Parser)]
#[command(name = "faelight-link")]
#[command(version = "3.0.0")]
#[command(about = "Zone-aware symlink manager for Faelight Forest", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Preview changes without applying (dry-run mode)
    #[arg(long, global = true)]
    dry_run: bool,

    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Stow a package (create symlinks)
    Stow {
        /// Package name
        package: String,

        /// Skip verification prompts
        #[arg(long)]
        force: bool,
    },

    /// Unstow a package (remove symlinks)
    Unstow {
        /// Package name
        package: String,
    },

    /// List all packages
    List,

    /// Show status of links
    Status,

    /// Audit link health (check for broken/orphaned links)
    Audit,

    /// Clean up broken and orphaned links
    Clean {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Forest-aware status with intent tracing (v3)
    StatusV3,
    /// Intent traceability audit (v3)
    AuditV3,
    /// Show which package owns a file (v3)
    Why { file: String },
    /// Deep validation of all symlinks (v3)
    Verify,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{}", "🔗 faelight-link v3.0.0".bright_blue().bold());

    match cli.command {
        Commands::Stow { package, force } => {
            println!("📦 Stowing package: {}", package.bright_green());
            package::stow(&package, force, cli.dry_run)?;
            // INT-214: emit signal for coordination
            if !cli.dry_run {
                let _ = emit_signal("package_linked", &package);
            }
        }
        Commands::Unstow { package } => {
            println!("📦 Unstowing package: {}", package.bright_yellow());
            package::unstow(&package, cli.dry_run)?;
            // INT-214: emit signal for coordination
            if !cli.dry_run {
                let _ = emit_signal("package_unlinked", &package);
            }
        }
        Commands::List => {
            println!("📋 Available packages:");
            package::list()?;
        }
        Commands::Status => {
            println!("📊 Link status:");
            link::status()?;
        }
        Commands::Audit => {
            println!("📊 Auditing link health:");
            link::audit()?;
        }
        Commands::Clean { force } => {
            println!("🧹 Cleaning up broken links:");
            link::clean(force)?;
        }
        Commands::StatusV3 => {
            intelligence::status_v3()?;
        }
        Commands::AuditV3 => {
            intelligence::audit_v3()?;
        }
        Commands::Why { file } => {
            intelligence::why(&file)?;
        }
        Commands::Verify => {
            intelligence::verify()?;
        }
    }

    Ok(())
}

fn emit_signal(signal_type: &str, package: &str) -> anyhow::Result<()> {
    let db_path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join("0-core/runtime/state.db");
    if !db_path.exists() {
        return Ok(());
    }
    let conn = rusqlite::Connection::open(&db_path)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let payload = format!("{{\"package\":\"{}\"}}", package);
    let _ = conn.execute(
        "INSERT INTO engine_signals (source, signal_type, payload, weight, created_at)
         VALUES ('faelight-link', ?1, ?2, 1.0, ?3)",
        rusqlite::params![signal_type, payload, now],
    );
    Ok(())
}

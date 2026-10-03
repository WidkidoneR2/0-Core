//! zero-git v2.1 - Git Governance Layer
//! Git becomes a policy boundary

use clap::{Parser, Subcommand};
use colored::*;
use std::process::{exit, Command};

// Import our library modules
use zero_core::paths;
use zero_git::commands;

#[derive(Parser)]
#[command(name = "zero-git")]
#[command(about = "Git Governance for Project 0")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show risk-aware repository status
    Status,

    /// Show detailed risk assessment
    Risk,

    /// Pre-commit verification with intent checking
    Commit {
        #[arg(long)]
        intent: Option<String>,

        #[arg(long)]
        no_intent: bool,
    },

    /// Interactive git workflow (pull → stage → commit → push)
    Sync,

    /// Quick commit and push with message
    Quick {
        /// Commit message
        message: String,
    },
    /// Stage all, commit with active intent prefix, push -- no prompts
    Done {
        /// Optional extra context appended to commit message
        message: Option<String>,
    },

    /// Branch management (switch, create, delete)
    Branch,

    /// View commit history
    Log {
        /// Number of commits to show (default: 10)
        #[arg(short = 'n', long, default_value = "10")]
        count: Option<usize>,
    },

    /// Verify commit/push readiness
    Verify,

    /// Rollback to a previous commit (interactive or by commit hash)
    Rollback {
        /// Specific commit hash to rollback to (optional)
        hash: Option<String>,
        /// Show what would change without executing
        #[arg(long)]
        dry_run: bool,
        /// Show last N commits with risk scores
        #[arg(long)]
        list: bool,
    },
}

fn main() {
    // INT-256: FIRST statement, before any output. `tool | head -3` must not print a panic.
    zero_core::restore_sigpipe();
    let cli = Cli::parse();

    let exit_code = match cli.command {
        // v2.x commands using git2-rs
        Commands::Status => match commands::status::run() {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{} {}", "Error:".red(), e);
                1
            }
        },

        Commands::Risk => match commands::risk::run() {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{} {}", "Error:".red(), e);
                1
            }
        },

        Commands::Commit { intent, no_intent } => match commands::commit::run(intent, no_intent) {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{} {}", "Error:".red(), e);
                1
            }
        },

        Commands::Sync => match commands::sync::run() {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{} {}", "Error:".red(), e);
                1
            }
        },

        Commands::Done { message } => match commands::done::run(message.as_deref()) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{} {}", "  ✗".red(), e);
                1
            }
        },
        Commands::Quick { message } => match commands::quick::run(&message) {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{} {}", "Error:".red(), e);
                1
            }
        },

        Commands::Branch => match commands::branch::run() {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{} {}", "Error:".red(), e);
                1
            }
        },

        Commands::Log { count } => match commands::log::run(count) {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{} {}", "Error:".red(), e);
                1
            }
        },

        // v0.1 commands (preserved)
        Commands::Verify => verify(),
        Commands::Rollback {
            hash,
            dry_run,
            list,
        } => match commands::rollback::run(hash.as_deref(), dry_run, list) {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{} {}", "Error:".red(), e);
                1
            }
        },
    };

    exit(exit_code);
}

// ═══════════════════════════════════════════════════════════
// 🔍 VERIFY
// ═══════════════════════════════════════════════════════════

fn verify() -> i32 {
    println!("{}", "🔍 Verifying git state...".cyan());
    println!();

    let mut issues = 0;

    println!("  {} Core is unlocked", "✅".green());

    // Check for uncommitted changes
    let status = Command::new("git")
        .args([
            "-C",
            paths::core_dir().to_str().unwrap(),
            "status",
            "--porcelain",
        ])
        .output();

    if let Ok(output) = status {
        if output.stdout.is_empty() {
            println!("  {} Working tree clean", "✅".green());
        } else {
            let lines = String::from_utf8_lossy(&output.stdout).lines().count();
            println!("  {} {} uncommitted changes", "⚠️".yellow(), lines);
            issues += 1;
        }
    }

    // Check for unpushed commits
    let unpushed = Command::new("git")
        .args([
            "-C",
            paths::core_dir().to_str().unwrap(),
            "log",
            "@{u}..",
            "--oneline",
        ])
        .output();

    if let Ok(output) = unpushed {
        if output.stdout.is_empty() {
            println!("  {} All commits pushed", "✅".green());
        } else {
            let count = String::from_utf8_lossy(&output.stdout).lines().count();
            println!("  {} {} unpushed commits", "⚠️".yellow(), count);
        }
    }

    println!();
    if issues > 0 {
        println!("{}", "Some issues found. Fix before committing.".red());
        1
    } else {
        println!("{}", "Ready to commit!".green());
        0
    }
}

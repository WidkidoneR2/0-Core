use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;

mod checks;
mod install;

#[derive(Parser)]
#[command(name = "faelight-hooks")]
#[command(about = "🎣 Git hooks management - Faelight Forest", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install git hooks
    Install {
        /// Specific hook to install (pre-commit, pre-push, commit-msg)
        hook: Option<String>,
    },
    /// Run hook checks manually
    Check {
        /// Skip specific checks (comma-separated: secrets,conflicts,syntax)
        #[arg(long)]
        skip: Option<String>,
        
        /// Run pre-push checks
        #[arg(long)]
        pre_push: bool,
        
        /// Validate commit message from file
        #[arg(long)]
        commit_msg: Option<String>,
    },
    /// Configure hook settings
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { hook } => {
            install::install_hooks(hook)?;
        }
        Commands::Check { skip, pre_push, commit_msg } => {
            if let Some(msg_file) = commit_msg {
                // Commit message validation
                run_commit_msg_check(&msg_file)?;
            } else if pre_push {
                // Pre-push checks
                run_pre_push_checks()?;
            } else {
                // Pre-commit checks
                println!("{}", "🔍 Running hook checks...".cyan().bold());
                run_checks(skip)?;
            }
        }
        Commands::Config { show } => {
            if show {
                println!("{}", "⚙️  Hook Configuration".cyan().bold());
                show_config()?;
            }
        }
    }

    Ok(())
}

fn run_checks(skip: Option<String>) -> Result<()> {
    let skip_list: Vec<String> = skip
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let mut all_passed = true;

    // Secret scanning
    if !skip_list.contains(&"secrets".to_string()) {
        if !checks::secrets::check_secrets()? {
            all_passed = false;
        }
    } else {
        println!("{}", "⏭️  Skipping secret scanning".yellow());
    }

    // Conflict detection
    if !skip_list.contains(&"conflicts".to_string()) {
        if !checks::conflicts::check_conflicts()? {
            all_passed = false;
        }
    } else {
        println!("{}", "⏭️  Skipping conflict detection".yellow());
    }

    println!();
    if all_passed {
        println!("{}", "✅ All checks passed! 🌲".green().bold());
        Ok(())
    } else {
        println!("{}", "❌ Some checks failed!".red().bold());
        std::process::exit(1);
    }
}

fn run_pre_push_checks() -> Result<()> {
    println!("{}", "🎣 Running pre-push checks...".cyan().bold());
    
    let mut all_passed = true;

    // Check for uncommitted changes
    if !checks::prepush::check_unpushed_changes()? {
        all_passed = false;
    }

    // Check push target (main branch warning)
    if !checks::prepush::check_push_to_main()? {
        all_passed = false;
    }

    println!();
    if all_passed {
        println!("{}", "✅ Pre-push checks passed! 🌲".green().bold());
        Ok(())
    } else {
        println!("{}", "❌ Pre-push checks failed!".red().bold());
        std::process::exit(1);
    }
}

fn run_commit_msg_check(msg_file: &str) -> Result<()> {
    // Read commit message from file
    let msg = fs::read_to_string(msg_file)?;
    
    if !checks::commitmsg::validate_commit_msg(&msg)? {
        println!("{}", "❌ Commit message validation failed!".red().bold());
        std::process::exit(1);
    }

    Ok(())
}

fn show_config() -> Result<()> {
    println!("Current configuration:");
    println!();
    println!("{}", "Pre-commit checks:".bold());
    println!("  - Secret scanning: {}", "enabled".green());
    println!("  - Conflict detection: {}", "enabled".green());
    println!();
    println!("{}", "Pre-push checks:".bold());
    println!("  - Branch warnings: {}", "enabled".green());
    println!("  - Uncommitted changes: {}", "enabled".green());
    println!();
    println!("{}", "Commit-msg checks:".bold());
    println!("  - Conventional commits: {}", "validation (non-blocking)".yellow());
    println!("  - Length checks: {}", "enabled".green());
    Ok(())
}

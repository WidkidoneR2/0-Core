use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::time::Instant;

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
        /// Skip specific checks (comma-separated: secrets,conflicts,filesize,branch,rustfmt,clippy)
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
                println!();
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
    
    let start_time = Instant::now();
    let mut all_passed = true;
    let mut check_times: Vec<(&str, u128)> = Vec::new();
    
    // Branch name validation (non-blocking)
    if !skip_list.contains(&"branch".to_string()) {
        let check_start = Instant::now();
        let _ = checks::branch::validate_branch_name()?;
        check_times.push(("Branch", check_start.elapsed().as_millis()));
        println!();
    }
    
    // File size check (non-blocking warning)
    if !skip_list.contains(&"filesize".to_string()) {
        let check_start = Instant::now();
        let _ = checks::filesize::check_file_sizes()?;
        check_times.push(("FileSize", check_start.elapsed().as_millis()));
        println!();
    }
    
    // Rustfmt check (BLOCKING)
    if !skip_list.contains(&"rustfmt".to_string()) {
        let check_start = Instant::now();
        if !checks::rustfmt::check_rustfmt()? {
            all_passed = false;
        }
        check_times.push(("Rustfmt", check_start.elapsed().as_millis()));
        println!();
    }
    
    // Clippy check (BLOCKING)
    if !skip_list.contains(&"clippy".to_string()) {
        let check_start = Instant::now();
        if !checks::clippy::check_clippy()? {
            all_passed = false;
        }
        check_times.push(("Clippy", check_start.elapsed().as_millis()));
        println!();
    }
    
    // Secret scanning (BLOCKING)
    if !skip_list.contains(&"secrets".to_string()) {
        let check_start = Instant::now();
        if !checks::secrets::check_secrets()? {
            all_passed = false;
        }
        check_times.push(("Secrets", check_start.elapsed().as_millis()));
        println!();
    } else {
        println!("{}", "⏭️  Skipping secret scanning".yellow());
        println!();
    }
    
    // Conflict detection (BLOCKING)
    if !skip_list.contains(&"conflicts".to_string()) {
        let check_start = Instant::now();
        if !checks::conflicts::check_conflicts()? {
            all_passed = false;
        }
        check_times.push(("Conflicts", check_start.elapsed().as_millis()));
        println!();
    } else {
        println!("{}", "⏭️  Skipping conflict detection".yellow());
        println!();
    }
    
    if all_passed {
        let total_time = start_time.elapsed();
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!("{}", "📊 Check Statistics".cyan().bold());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        for (name, time) in check_times {
            println!("   {:<12} {}ms", name, time);
        }
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
        println!("   {:<12} {}ms", "Total".bold(), total_time.as_millis());
        println!();
        println!("{}", "✅ All checks passed! 🌲".green().bold());
        Ok(())
    } else {
        println!("{}", "❌ Some checks failed!".red().bold());
        println!("   💡 Check the errors above for details");
        std::process::exit(1);
    }
}

fn run_pre_push_checks() -> Result<()> {
    println!("{}", "🎣 Running pre-push checks...".cyan().bold());
    println!();
    
    let mut all_passed = true;
    
    // Check for uncommitted changes (warning)
    let _ = checks::prepush::check_unpushed_changes()?;
    println!();
    
    // Check push target (main branch warning)
    if !checks::prepush::check_push_to_main()? {
        all_passed = false;
    }
    
    if all_passed {
        println!("{}", "✅ Pre-push checks passed! 🌲".green().bold());
        Ok(())
    } else {
        println!("{}", "❌ Pre-push checks failed!".red().bold());
        println!("   💡 Fix the issues above or use --no-verify to skip");
        std::process::exit(1);
    }
}

fn run_commit_msg_check(msg_file: &str) -> Result<()> {
    // Read commit message from file
    let msg = fs::read_to_string(msg_file)?;
    
    if !checks::commitmsg::validate_commit_msg(&msg)? {
        println!("{}", "❌ Commit message validation failed!".red().bold());
        println!("   💡 Use format: type: description");
        println!("   💡 Example: feat: Add user authentication");
        println!("   💡 Types: feat, fix, docs, style, refactor, test, chore");
        std::process::exit(1);
    }
    
    Ok(())
}

fn show_config() -> Result<()> {
    println!("Current configuration:");
    println!();
    println!("{}", "Pre-commit checks:".bold());
    println!("  - Branch name validation: {}", "enabled (non-blocking)".yellow());
    println!("  - File size check: {}", "enabled (50MB warning)".yellow());
    println!("  - Secret scanning: {}", "enabled (BLOCKING)".red());
    println!("  - Conflict detection: {}", "enabled (BLOCKING)".red());
    println!();
    println!("{}", "Pre-push checks:".bold());
    println!("  - Branch warnings: {}", "enabled".green());
    println!("  - Uncommitted changes: {}", "enabled (warning)".yellow());
    println!();
    println!("{}", "Commit-msg checks:".bold());
    println!("  - Conventional commits: {}", "validation (non-blocking)".yellow());
    println!("  - Length checks: {}", "enabled".green());
    println!();
    println!("{}", "Skip checks with:".dimmed());
    println!("  {}", "faelight-hooks check --skip secrets,filesize".dimmed());
    
    Ok(())
}

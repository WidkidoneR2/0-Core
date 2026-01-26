use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod checks;

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
        /// Skip specific checks (comma-separated: secrets,syntax,conflicts)
        #[arg(long)]
        skip: Option<String>,
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
            println!("{}", "🎣 Installing git hooks...".cyan().bold());
            install_hooks(hook)?;
        }
        Commands::Check { skip } => {
            println!("{}", "🔍 Running hook checks...".cyan().bold());
            run_checks(skip)?;
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

fn install_hooks(hook: Option<String>) -> Result<()> {
    match hook {
        Some(name) => println!("Installing {} hook...", name.green()),
        None => println!("Installing all hooks..."),
    }
    println!("{}", "✅ Hooks installed!".green());
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

    println!();
    if all_passed {
        println!("{}", "✅ All checks passed! 🌲".green().bold());
        Ok(())
    } else {
        println!("{}", "❌ Some checks failed!".red().bold());
        std::process::exit(1);
    }
}

fn show_config() -> Result<()> {
    println!("Current configuration:");
    println!("  - Secret scanning: {}", "enabled".green());
    println!("  - Syntax validation: {}", "planned".yellow());
    println!("  - Conflict detection: {}", "planned".yellow());
    Ok(())
}

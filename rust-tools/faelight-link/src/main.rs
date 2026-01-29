use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod package;
mod link;
mod conflict;

#[derive(Parser)]
#[command(name = "faelight-link")]
#[command(version = "1.0.0")]
#[command(about = "Zone-aware symlink manager for Faelight Forest", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    println!("{}", "🔗 faelight-link v0.1.0".bright_blue().bold());
    
    match cli.command {
        Commands::Stow { package, force } => {
            println!("📦 Stowing package: {}", package.bright_green());
            package::stow(&package, force)?;
        }
        Commands::Unstow { package } => {
            println!("📦 Unstowing package: {}", package.bright_yellow());
            package::unstow(&package)?;
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
    }
    
    Ok(())
}

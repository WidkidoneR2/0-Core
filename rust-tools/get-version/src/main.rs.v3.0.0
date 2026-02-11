//! get-version v3.0.0
//! Simple utility to read system version from VERSION file
use clap::{Parser, Subcommand};
use faelight_core::paths;
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "get-version")]
#[command(version = "3.0.0")]
#[command(about = "Get 0-Core system version", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show system version
    System,
    /// Show package version from .dotmeta
    Package { name: String },
    /// Run health check
    Health,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::System) | None => {
            if let Some(version) = get_system_version() {
                println!("{}", version);
            } else {
                eprintln!("❌ Could not read system version");
                process::exit(1);
            }
        }
        Some(Commands::Package { name }) => {
            if let Some(version) = get_package_version(&name) {
                println!("{}", version);
            } else {
                eprintln!("❌ Could not find version for package: {}", name);
                process::exit(1);
            }
        }
        Some(Commands::Health) => {
            health_check();
        }
    }
}

fn get_system_version() -> Option<String> {
    let version_file = paths::version_file();
    fs::read_to_string(version_file)
        .ok()?
        .trim()
        .to_string()
        .into()
}

fn get_package_version(package: &str) -> Option<String> {
    // Try stow path first (current structure)
    let stow_path = paths::stow_dir().join(package).join(".dotmeta");

    if let Some(version) = read_version_from_dotmeta(&stow_path) {
        return Some(version);
    }

    // Fallback to old path (for backwards compatibility)
    let old_path = paths::core_dir().join(package).join(".dotmeta");

    read_version_from_dotmeta(&old_path)
}

fn read_version_from_dotmeta(path: &PathBuf) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;

    for line in content.lines() {
        if line.starts_with("version=") {
            return Some(line.trim_start_matches("version=").to_string());
        }
    }

    None
}

fn health_check() {
    println!("🔍 get-version v3.0.0 - Health Check");

    let stow_dir = paths::stow_dir();

    if !stow_dir.exists() {
        eprintln!("❌ Stow directory not found: {}", stow_dir.display());
        process::exit(1);
    }

    // Test with a known package
    if let Some(version) = get_package_version("shell-zsh") {
        println!("✅ Can read package versions (shell-zsh: {})", version);
    } else {
        eprintln!("⚠️  Could not read test package version");
    }

    if let Some(version) = get_system_version() {
        println!("✅ System version: {}", version);
    } else {
        eprintln!("❌ Could not read system version");
        process::exit(1);
    }

    println!("✅ All checks passed");
}

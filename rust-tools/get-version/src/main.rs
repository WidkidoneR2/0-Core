//! 🔍 get-version - Extract version from package .dotmeta files
//! 
//! Reads version information from stow package metadata.

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "get-version")]
#[command(about = "Get version from package .dotmeta file", long_about = None)]
#[command(version = "2.0.0")]
struct Args {
    /// Package name to query
    package: Option<String>,
    
    /// Run health check and exit
    #[arg(long)]
    health_check: bool,
}

fn main() {
    let args = Args::parse();
    
    if args.health_check {
        health_check();
        return;
    }
    
    let package = match args.package {
        Some(p) => p,
        None => {
            eprintln!("❌ Error: Package name required");
            eprintln!("Usage: get-version <package-name>");
            process::exit(1);
        }
    };
    
    match get_package_version(&package) {
        Some(version) => println!("{}", version),
        None => println!("unknown"),
    }
}

fn get_package_version(package: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    
    // Try stow path first (current structure)
    let stow_path = PathBuf::from(&home)
        .join("0-core/stow")
        .join(package)
        .join(".dotmeta");
    
    if let Some(version) = read_version_from_dotmeta(&stow_path) {
        return Some(version);
    }
    
    // Fallback to old path (for backwards compatibility)
    let old_path = PathBuf::from(&home)
        .join("0-core")
        .join(package)
        .join(".dotmeta");
    
    read_version_from_dotmeta(&old_path)
}

fn read_version_from_dotmeta(path: &PathBuf) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    
    for line in content.lines() {
        if line.starts_with("version = ") {
            // Extract value between quotes
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if start < end {
                        return Some(line[start + 1..end].to_string());
                    }
                }
            }
        }
    }
    
    None
}

fn health_check() {
    println!("🔍 get-version v2.0.0 - Health Check");
    
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => {
            eprintln!("❌ HOME environment variable not set");
            process::exit(1);
        }
    };
    
    let stow_dir = PathBuf::from(&home).join("0-core/stow");
    
    if !stow_dir.exists() {
        eprintln!("❌ Stow directory not found: {}", stow_dir.display());
        process::exit(1);
    }
    
    // Test with a known package
    if let Some(version) = get_package_version("shell-zsh") {
        println!("✅ Successfully read shell-zsh version: {}", version);
        println!("✅ Health check passed");
        process::exit(0);
    } else {
        eprintln!("⚠️  Could not read shell-zsh version (package may not exist)");
        println!("✅ Health check passed (tool functional)");
        process::exit(0);
    }
}

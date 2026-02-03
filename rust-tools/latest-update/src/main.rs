//! latest-update v3.0.0
//! Show the most recently updated packages from .dotmeta files
use chrono::{DateTime, Local};
use clap::Parser;
use std::fs;
use std::process;
use faelight_core::paths;

#[derive(Parser)]
#[command(name = "latest-update")]
#[command(version = "3.0.0")]
#[command(about = "Show latest package updates", long_about = None)]
struct Cli {
    /// Number of recent packages to show
    #[arg(short = 'n', long, default_value = "10")]
    count: usize,
    
    /// Show all packages (no limit)
    #[arg(short, long)]
    all: bool,
}

#[derive(Debug)]
struct PackageInfo {
    name: String,
    version: String,
    updated: DateTime<Local>,
}

fn main() {
    let cli = Cli::parse();
    
    let mut packages = scan_packages();
    
    if packages.is_empty() {
        println!("No packages found with .dotmeta files");
        return;
    }
    
    // Sort by update time (most recent first)
    packages.sort_by(|a, b| b.updated.cmp(&a.updated));
    
    let count = if cli.all {
        packages.len()
    } else {
        cli.count.min(packages.len())
    };
    
    println!("📦 Latest {} package updates:", count);
    println!();
    
    for pkg in packages.iter().take(count) {
        let time_str = pkg.updated.format("%Y-%m-%d %H:%M:%S");
        println!("  {} {} ({})", pkg.name, pkg.version, time_str);
    }
}

fn scan_packages() -> Vec<PackageInfo> {
    let stow_dir = paths::stow_dir();
    
    if !stow_dir.exists() {
        eprintln!("❌ Stow directory not found: {}", stow_dir.display());
        process::exit(1);
    }
    
    let mut packages = Vec::new();
    
    let entries = match fs::read_dir(&stow_dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("❌ Failed to read stow directory: {}", err);
            process::exit(1);
        }
    };
    
    for entry in entries.flatten() {
        let path = entry.path();
        
        if !path.is_dir() {
            continue;
        }
        
        let dotmeta = path.join(".dotmeta");
        
        if !dotmeta.exists() {
            continue;
        }
        
        // Read .dotmeta file
        let content = match fs::read_to_string(&dotmeta) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        // Get modification time
        let metadata = match fs::metadata(&dotmeta) {
            Ok(m) => m,
            Err(_) => continue,
        };
        
        let modified = match metadata.modified() {
            Ok(t) => DateTime::<Local>::from(t),
            Err(_) => continue,
        };
        
        // Parse version from .dotmeta
        let mut version = String::from("unknown");
        for line in content.lines() {
            if line.starts_with("version=") {
                version = line.trim_start_matches("version=").to_string();
                break;
            }
        }
        
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        packages.push(PackageInfo {
            name,
            version,
            updated: modified,
        });
    }
    
    packages
}

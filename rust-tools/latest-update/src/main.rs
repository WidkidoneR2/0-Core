//! 📅 latest-update - Find most recently updated package
//! 
//! Scans stow packages to find which was updated most recently.

use clap::Parser;
use chrono::{DateTime, NaiveDateTime, Local};
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "latest-update")]
#[command(about = "Find most recently updated package", long_about = None)]
#[command(version = "2.0.0")]
struct Args {
    /// Show all packages (not just latest)
    #[arg(short, long)]
    all: bool,
    
    /// Run health check and exit
    #[arg(long)]
    health_check: bool,
}

struct PackageInfo {
    name: String,
    version: String,
    last_updated: String,
}

fn main() {
    let args = Args::parse();
    
    if args.health_check {
        health_check();
        return;
    }
    
    let packages = scan_packages();
    
    if packages.is_empty() {
        eprintln!("⚠️  No packages found with update information");
        process::exit(1);
    }
    
    if args.all {
        println!("📦 All packages (newest first):");
        for pkg in &packages {
            print_package(pkg);
        }
    } else {
        // Show only the latest
        if let Some(latest) = packages.first() {
            print_package(latest);
        }
    }
}

fn scan_packages() -> Vec<PackageInfo> {
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
    
    let mut packages = Vec::new();
    
    let entries = match fs::read_dir(&stow_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("❌ Failed to read stow directory: {}", e);
            process::exit(1);
        }
    };
    
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        
        let dotmeta_path = path.join(".dotmeta");
        if !dotmeta_path.exists() {
            continue;
        }
        
        let content = match fs::read_to_string(&dotmeta_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        let pkg_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let mut updated = String::new();
        let mut version = String::new();
        
        for line in content.lines() {
            if line.starts_with("last_updated = ") {
                updated = extract_quoted_value(line);
            } else if line.starts_with("version = ") && version.is_empty() {
                version = extract_quoted_value(line);
            }
        }
        
        if !updated.is_empty() {
            packages.push(PackageInfo {
                name: pkg_name,
                version,
                last_updated: updated,
            });
        }
    }
    
    // Sort by date (newest first)
    packages.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
    
    packages
}

fn extract_quoted_value(line: &str) -> String {
    if let Some(start) = line.find('"') {
        if let Some(end) = line.rfind('"') {
            if start < end {
                return line[start + 1..end].to_string();
            }
        }
    }
    String::new()
}

fn print_package(pkg: &PackageInfo) {
    let time_ago = format_time_ago(&pkg.last_updated);
    println!("📦 {} v{} ({})", pkg.name, pkg.version, time_ago);
}

fn format_time_ago(date_str: &str) -> String {
    // Try to parse the date and show relative time
    // Format: "2026-01-19"
    if let Ok(date) = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S") {
        let date_time = DateTime::<Local>::from_naive_utc_and_offset(date, *Local::now().offset());
        let now = Local::now();
        let duration = now.signed_duration_since(date_time);
        
        let days = duration.num_days();
        if days == 0 {
            return "today".to_string();
        } else if days == 1 {
            return "yesterday".to_string();
        } else if days < 7 {
            return format!("{} days ago", days);
        } else if days < 30 {
            return format!("{} weeks ago", days / 7);
        } else {
            return format!("{} months ago", days / 30);
        }
    }
    
    // Fallback to showing the date
    date_str.to_string()
}

fn health_check() {
    println!("📅 latest-update v2.0.0 - Health Check");
    
    let packages = scan_packages();
    
    if packages.is_empty() {
        eprintln!("⚠️  No packages found");
        println!("✅ Health check passed (tool functional)");
        process::exit(0);
    }
    
    println!("✅ Found {} packages with update info", packages.len());
    
    if let Some(latest) = packages.first() {
        println!("✅ Latest: {} v{} ({})", 
                 latest.name, latest.version, latest.last_updated);
    }
    
    println!("✅ Health check passed");
    process::exit(0);
}

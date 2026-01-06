use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};

// ANSI colors
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const CYAN: &str = "\x1b[0;36m";
const BLUE: &str = "\x1b[0;34m";
const NC: &str = "\x1b[0m";

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    
    match command {
        "status" => cmd_status(),
        "bump" => cmd_bump(&args[2..]),
        "history" => cmd_history(&args[2..]),
        "health" => cmd_health(),
        "help" | "--help" | "-h" => cmd_help(),
        _ => {
            eprintln!("Unknown command: {}", command);
            cmd_help();
            process::exit(1);
        }
    }
}

fn get_core_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("0-core")
}

fn cmd_status() {
    let core_dir = get_core_dir();
    
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!("{}📊 0-Core System Status{}", CYAN, NC);
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!();
    
    // System version
    let version_file = core_dir.join("VERSION");
    if let Ok(version) = fs::read_to_string(&version_file) {
        println!("{}System Version:{} v{}", GREEN, NC, version.trim());
    }
    
    println!();
    println!("{}Package Versions:{}", BLUE, NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // List all packages
    if let Ok(entries) = fs::read_dir(&core_dir) {
        let mut packages: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter(|e| e.path().join(".dotmeta").exists())
            .collect();
        
        packages.sort_by_key(|e| e.file_name());
        
        for entry in packages {
            let dotmeta_path = entry.path().join(".dotmeta");
            if let Ok(content) = fs::read_to_string(&dotmeta_path) {
                let pkg_name = entry.file_name().to_string_lossy().to_string();
                let version = extract_field(&content, "version").unwrap_or("?".to_string());
                let category = extract_field(&content, "category").unwrap_or("misc".to_string());
                let blast = extract_field(&content, "blast_radius").unwrap_or("low".to_string());
                
                let icon = match blast.as_str() {
                    "critical" => format!("{}🔴{}", RED, NC),
                    "high" => format!("{}🟠{}", YELLOW, NC),
                    "medium" => format!("{}🔵{}", BLUE, NC),
                    _ => format!("{}🟢{}", GREEN, NC),
                };
                
                println!("  {} {:<25} v{:<8} ({})", icon, pkg_name, version, category);
            }
        }
    }
    
    println!();
    println!("{}Latest Update:{}", BLUE, NC);
    let latest_cmd = core_dir.join("scripts/latest-update");
    if let Ok(output) = Command::new(&latest_cmd).output() {
        let latest = String::from_utf8_lossy(&output.stdout);
        println!("  {}", latest.trim());
    }
    
    println!();
    println!("{}Health Status:{}", BLUE, NC);
    if let Ok(output) = Command::new("dot-doctor").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().find(|l| l.contains("Health:")) {
            if line.contains("100%") {
                println!("  {}✅ 100%{}", GREEN, NC);
            } else {
                println!("  {}⚠️  {}{}", YELLOW, line.split_whitespace().last().unwrap_or("?"), NC);
            }
        }
    } else {
        println!("  Unknown (dot-doctor not found)");
    }
    
    println!();
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
}

fn cmd_bump(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: dotctl bump <package> <version> [change-note]");
        eprintln!();
        eprintln!("Example: dotctl bump shell-zsh 3.3.1 'Added new features'");
        process::exit(1);
    }
    
    let core_dir = get_core_dir();
    let bump_cmd = core_dir.join("scripts/bump-version");
    
    let status = Command::new(&bump_cmd)
        .args(args)
        .status()
        .expect("Failed to run bump-version");
    
    process::exit(status.code().unwrap_or(1));
}

fn cmd_history(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: dotctl history <package>");
        process::exit(1);
    }
    
    let package = &args[0];
    let core_dir = get_core_dir();
    let dotmeta_path = core_dir.join(package).join(".dotmeta");
    
    if !dotmeta_path.exists() {
        eprintln!("❌ No .dotmeta found for package: {}", package);
        process::exit(1);
    }
    
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!("{}📜 Changelog: {}{}", CYAN, package, NC);
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!();
    
    if let Ok(content) = fs::read_to_string(&dotmeta_path) {
        for line in content.lines() {
            if line.starts_with('"') && line.contains("\" = \"") {
                // Parse "version" = "change"
                let parts: Vec<&str> = line.split("\" = \"").collect();
                if parts.len() == 2 {
                    let version = parts[0].trim_start_matches('"');
                    let change = parts[1].trim_end_matches('"');
                    println!("  {}v{}{}: {}", GREEN, version, NC, change);
                }
            }
        }
    }
    
    println!();
}

fn cmd_health() {
    let status = Command::new("dot-doctor")
        .status()
        .expect("Failed to run dot-doctor");
    
    process::exit(status.code().unwrap_or(1));
}

fn cmd_help() {
    println!("═══════════════════════════════════════════════════════════");
    println!("🎮 dotctl - 0-Core Control Utility");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("COMMANDS:");
    println!("  status              Show system and package versions");
    println!("  bump <pkg> <ver>    Bump package version");
    println!("  history <pkg>       Show package changelog");
    println!("  health              Run system health check");
    println!("  help                Show this help");
    println!();
    println!("EXAMPLES:");
    println!("  dotctl status");
    println!("  dotctl bump shell-zsh 3.3.1 \"Added aliases\"");
    println!("  dotctl history wm-hypr");
    println!("  dotctl health");
    println!();
    println!("═══════════════════════════════════════════════════════════");
}

fn extract_field(content: &str, field: &str) -> Option<String> {
    let prefix = format!("{} = ", field);
    for line in content.lines() {
        if line.starts_with(&prefix) {
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

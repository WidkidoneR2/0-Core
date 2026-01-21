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

const VERSION: &str = "2.0.0";


fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    
    match command {
        "status" => cmd_status(),
        "bump" => cmd_bump(&args[2..]),
        "history" => cmd_history(&args[2..]),
        "health" => cmd_health(),
        "--version" | "-v" | "version" => cmd_version(),
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

fn get_stow_dir() -> PathBuf {
    get_core_dir().join("stow")
}

fn cmd_version() {
    println!("dotctl v{}", VERSION);
}

fn parse_dotmeta(content: &str) -> (String, String, String, String) {
    let mut version = "?".to_string();
    let mut category = "misc".to_string();
    let mut blast = "low".to_string();
    let mut description = "".to_string();
    
    // Try TOML format first
    if content.contains("[package]") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("version = ") {
                version = line.split('=').nth(1)
                    .unwrap_or("?")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            } else if line.starts_with("category = ") {
                category = line.split('=').nth(1)
                    .unwrap_or("misc")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            } else if line.starts_with("blast_radius = ") {
                blast = line.split('=').nth(1)
                    .unwrap_or("low")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            } else if line.starts_with("description = ") {
                description = line.split('=').nth(1)
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            }
        }
    } else {
        // Simple format
        for line in content.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "version" => version = value.to_string(),
                    "category" => category = value.to_string(),
                    "blast_radius" => blast = value.to_string(),
                    "description" => description = value.to_string(),
                    _ => {}
                }
            }
        }
    }
    
    (version, category, blast, description)
}

fn cmd_status() {
    let core_dir = get_core_dir();
    let stow_dir = get_stow_dir();
    
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
    
    // List all stow packages
    if let Ok(entries) = fs::read_dir(&stow_dir) {
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
                let (version, category, blast, _) = parse_dotmeta(&content);
                
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
                println!("  {}⚠️  {}{}", YELLOW, line.split("Health:").nth(1).unwrap_or("?").trim(), NC);
            }
        }
    }
    
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
}

fn cmd_bump(args: &[String]) {
    if args.len() < 2 {
        eprintln!("{}Usage:{} dotctl bump <package> <version> [message]", YELLOW, NC);
        process::exit(1);
    }
    
    let pkg_name = &args[0];
    let new_version = &args[1];
    let message = args.get(2).map(|s| s.as_str()).unwrap_or("Version bump");
    
    let stow_dir = get_stow_dir();
    let pkg_dir = stow_dir.join(pkg_name);
    let dotmeta_path = pkg_dir.join(".dotmeta");
    
    if !dotmeta_path.exists() {
        eprintln!("{}❌ Package not found:{} {}", RED, NC, pkg_name);
        process::exit(1);
    }
    
    let content = fs::read_to_string(&dotmeta_path).expect("Failed to read .dotmeta");
    
    // Update version in .dotmeta
    let updated_content = if content.contains("[package]") {
        // TOML format
        content.lines().map(|line| {
            if line.trim().starts_with("version = ") {
                format!("version = \"{}\"", new_version)
            } else {
                line.to_string()
            }
        }).collect::<Vec<_>>().join("\n")
    } else {
        // Simple format
        content.lines().map(|line| {
            if line.trim().starts_with("version:") {
                format!("version: {}", new_version)
            } else {
                line.to_string()
            }
        }).collect::<Vec<_>>().join("\n")
    };
    
    fs::write(&dotmeta_path, updated_content).expect("Failed to write .dotmeta");
    
    println!("{}✅ Bumped {} to v{}{}", GREEN, pkg_name, new_version, NC);
    println!("   {}", message);
}

fn cmd_history(args: &[String]) {
    if args.is_empty() {
        eprintln!("{}Usage:{} dotctl history <package>", YELLOW, NC);
        process::exit(1);
    }
    
    let pkg_name = &args[0];
    let stow_dir = get_stow_dir();
    let dotmeta_path = stow_dir.join(pkg_name).join(".dotmeta");
    
    if !dotmeta_path.exists() {
        eprintln!("{}❌ No .dotmeta found for package:{} {}", RED, NC, pkg_name);
        process::exit(1);
    }
    
    let content = fs::read_to_string(&dotmeta_path).expect("Failed to read .dotmeta");
    
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!("{}📜 Change History: {}{}", CYAN, pkg_name, NC);
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!();
    
    // Parse changelog section (TOML format only)
    let mut in_changelog = false;
    for line in content.lines() {
        if line.trim() == "[changelog]" {
            in_changelog = true;
            continue;
        }
        if in_changelog {
            if line.trim().starts_with('[') && line.trim() != "[changelog]" {
                break;
            }
            if let Some((ver, msg)) = line.split_once('=') {
                let ver = ver.trim().trim_matches('"');
                let msg = msg.trim().trim_matches('"');
                println!("  {}v{}{} - {}", GREEN, ver, NC, msg);
            }
        }
    }
    
    println!();
}

fn cmd_health() {
    // Just run dot-doctor
    let status = Command::new("dot-doctor")
        .status()
        .expect("Failed to run dot-doctor");
    
    process::exit(status.code().unwrap_or(1));
}

fn cmd_help() {
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!("{}🎮 dotctl - 0-Core Control Utility{}", CYAN, NC);
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
    println!();
    println!("{}COMMANDS:{}", GREEN, NC);
    println!("  status              Show system and package versions");
    println!("  bump <pkg> <ver>    Bump package version");
    println!("  history <pkg>       Show package changelog");
    println!("  health              Run system health check");
    println!("  version, -v         Show dotctl version");
    println!("  help                Show this help");
    println!();
    println!("{}EXAMPLES:{}", GREEN, NC);
    println!("  dotctl status");
    println!("  dotctl bump shell-zsh 3.3.1 \"Added aliases\"");
    println!("  dotctl history wm-sway");
    println!("  dotctl health");
    println!("{}═══════════════════════════════════════════════════════════{}", CYAN, NC);
}

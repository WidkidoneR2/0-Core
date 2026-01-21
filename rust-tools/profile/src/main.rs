use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::{self, Command};

const VERSION: &str = "1.0.0";

// ANSI colors
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const CYAN: &str = "\x1b[0;36m";
const BLUE: &str = "\x1b[0;34m";
const NC: &str = "\x1b[0m";

fn main() {
    let home = env::var("HOME").expect("HOME not set");
    let state_dir = PathBuf::from(&home).join(".local/state/0-core");
    fs::create_dir_all(&state_dir).ok();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        cmd_status();
        return;
    }
    
    match args[1].as_str() {
        "list" | "ls" => cmd_list(),
        "status" => cmd_status(),
        "history" => cmd_history(),
        "health" => cmd_health(),
        "--version" | "-v" => {
            println!("profile v{}", VERSION);
        }
        "edit" => {
            if args.len() < 3 {
                error("Usage: profile edit <name>");
            }
            cmd_edit(&args[2]);
        }
        "export" => {
            if args.len() < 3 {
                error("Usage: profile export <name>");
            }
            cmd_export(&args[2]);
        }
        "import" => {
            if args.len() < 3 {
                error("Usage: profile import <file>");
            }
            cmd_import(&args[2]);
        }
        "help" | "-h" | "--help" => cmd_help(),
        name => cmd_switch(name),
    }
}

fn get_profile_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("0-core/profiles")
}

fn get_state_file() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".local/state/0-core/current-profile")
}

fn get_log_file() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".local/state/0-core/profile.log")
}

fn error(msg: &str) -> ! {
    eprintln!("{}❌ Error: {}{}", RED, msg, NC);
    process::exit(1);
}

fn success(msg: &str) {
    println!("{}✅ {}{}", GREEN, msg, NC);
}

fn info(msg: &str) {
    println!("{}ℹ️  {}{}", BLUE, msg, NC);
}

fn warn(msg: &str) {
    println!("{}⚠️  {}{}", YELLOW, msg, NC);
}

fn get_profile_icons() -> HashMap<&'static str, &'static str> {
    let mut icons = HashMap::new();
    icons.insert("default", "🏠");
    icons.insert("gaming", "🎮");
    icons.insert("work", "💼");
    icons.insert("low-power", "🔋");
    icons
}

fn get_profile_icon(name: &str) -> &'static str {
    get_profile_icons().get(name).copied().unwrap_or("📦")
}

fn get_current_profile() -> String {
    fs::read_to_string(get_state_file())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "default".to_string())
}

fn set_current_profile(name: &str) {
    fs::write(get_state_file(), name).ok();
}

fn log_switch(from: &str, to: &str) {
    let timestamp = get_timestamp();
    let entry = format!("{} | {} -> {}\n", timestamp, from, to);
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(get_log_file())
        .and_then(|mut f| f.write_all(entry.as_bytes()))
        .ok();
}

fn get_timestamp() -> String {
    let output = Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output()
        .expect("Failed to get date");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn get_profile_section(content: &str, section: &str) -> Vec<String> {
    let mut in_section = false;
    let section_header = format!("[{}]", section);
    let mut commands = Vec::new();
    
    for line in content.lines() {
        if line.trim() == section_header {
            in_section = true;
            continue;
        }
        if line.starts_with('[') {
            in_section = false;
            continue;
        }
        if in_section {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                commands.push(trimmed.to_string());
            }
        }
    }
    commands
}

fn run_section(profile_path: &PathBuf, section: &str) {
    if let Ok(content) = fs::read_to_string(profile_path) {
        let commands = get_profile_section(&content, section);
        for cmd in commands {
            info(&format!("Running: {}", cmd));
            let status = Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .status();
            if status.is_err() || !status.unwrap().success() {
                warn(&format!("Command failed (non-fatal): {}", cmd));
            }
        }
    }
}

fn cmd_health() {
    println!();
    println!("{}🏥 profile v{} - Health Check{}", CYAN, VERSION, NC);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut healthy = true;
    
    // Check profile directory
    print!("  Checking profile directory... ");
    let profile_dir = get_profile_dir();
    if profile_dir.exists() {
        println!("{}✅{}", GREEN, NC);
    } else {
        println!("{}❌ not found{}", RED, NC);
        healthy = false;
    }
    
    // Check state file
    print!("  Checking state tracking... ");
    let state_file = get_state_file();
    if state_file.exists() {
        println!("{}✅{}", GREEN, NC);
    } else {
        println!("{}⚠️  no state file (will create on first switch){}", YELLOW, NC);
    }
    
    // Count profiles
    print!("  Checking profiles... ");
    if let Ok(entries) = fs::read_dir(&profile_dir) {
        let count = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "profile").unwrap_or(false))
            .count();
        println!("{}✅ {} profiles found{}", GREEN, count, NC);
    } else {
        println!("{}❌ cannot read directory{}", RED, NC);
        healthy = false;
    }
    
    // Check current profile
    print!("  Checking current profile... ");
    let current = get_current_profile();
    let current_file = profile_dir.join(format!("{}.profile", current));
    if current_file.exists() {
        println!("{}✅ {} (valid){}", GREEN, current, NC);
    } else {
        println!("{}⚠️  {} (file not found){}", YELLOW, current, NC);
    }
    
    // Check system tools
    print!("  Checking powerprofilesctl... ");
    if Command::new("which").arg("powerprofilesctl").output().is_ok() {
        println!("{}✅{}", GREEN, NC);
    } else {
        println!("{}⚠️  not found (power management unavailable){}", YELLOW, NC);
    }
    
    print!("  Checking mullvad... ");
    if Command::new("which").arg("mullvad").output().is_ok() {
        println!("{}✅{}", GREEN, NC);
    } else {
        println!("{}⚠️  not found (VPN control unavailable){}", YELLOW, NC);
    }
    
    print!("  Checking makoctl... ");
    if Command::new("which").arg("makoctl").output().is_ok() {
        println!("{}✅{}", GREEN, NC);
    } else {
        println!("{}⚠️  not found (notification control unavailable){}", YELLOW, NC);
    }
    
    println!();
    if healthy {
        println!("{}✅ All systems operational{}", GREEN, NC);
        process::exit(0);
    } else {
        println!("{}⚠️  Some issues detected{}", YELLOW, NC);
        process::exit(1);
    }
}

fn cmd_list() {
    let profile_dir = get_profile_dir();
    let current = get_current_profile();
    
    println!("{}📋 Available Profiles{}", CYAN, NC);
    println!();
    
    if let Ok(entries) = fs::read_dir(&profile_dir) {
        let mut profiles: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "profile").unwrap_or(false))
            .collect();
        
        profiles.sort_by_key(|e| e.file_name());
        
        for entry in profiles {
            let name = entry.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            let icon = get_profile_icon(&name);
            
            // Get description from file
            let desc = fs::read_to_string(entry.path())
                .ok()
                .and_then(|content| {
                    content.lines()
                        .find(|l| l.starts_with("# Description:"))
                        .map(|l| l.replace("# Description:", "").trim().to_string())
                });
            
            if name == current {
                println!("  {}▶ {} {}{} (active)", GREEN, icon, name, NC);
            } else {
                println!("  {} {}", icon, name);
            }
            
            if let Some(d) = desc {
                println!("    {}{}{}", BLUE, d, NC);
            }
        }
    }
    println!();
}

fn cmd_status() {
    let current = get_current_profile();
    let icon = get_profile_icon(&current);
    
    println!("{}📊 Profile Status{}", CYAN, NC);
    println!();
    println!("  Current: {} {}{}{}", icon, GREEN, current, NC);
    println!();
    println!("  {}System State:{}", BLUE, NC);
    
    // Power profile
    let power = Command::new("powerprofilesctl")
        .arg("get")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("    Power: {}", power);
    
    // VPN status
    let vpn = Command::new("mullvad")
        .arg("status")
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("unknown")
                .to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string());
    println!("    VPN: {}", vpn);
    
    // Notifications
    let notif = Command::new("makoctl")
        .arg("mode")
        .output()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            if out.contains("dnd") {
                "Do Not Disturb".to_string()
            } else {
                "Normal".to_string()
            }
        })
        .unwrap_or_else(|_| "unknown".to_string());
    println!("    Notifications: {}", notif);
    
    println!();
}

fn cmd_switch(target: &str) {
    let profile_dir = get_profile_dir();
    let profile_file = profile_dir.join(format!("{}.profile", target));
    
    if !profile_file.exists() {
        error(&format!("Profile '{}' not found. Run 'profile list' to see available profiles.", target));
    }
    
    let current = get_current_profile();
    let current_file = profile_dir.join(format!("{}.profile", current));
    
    if target == current {
        info(&format!("Already on profile '{}'", target));
        return;
    }
    
    let target_icon = get_profile_icon(target);
    let current_icon = get_profile_icon(&current);
    
    println!("{}🔄 Switching Profile{}", CYAN, NC);
    println!("  From: {} {}", current_icon, current);
    println!("  To:   {} {}", target_icon, target);
    println!();
    
    // Deactivate current
    if current_file.exists() {
        info(&format!("Deactivating {}...", current));
        run_section(&current_file, "deactivate");
    }
    
    // Activate new
    info(&format!("Activating {}...", target));
    run_section(&profile_file, "activate");
    
    // Update state
    log_switch(&current, target);
    set_current_profile(target);
    
    println!();
    success(&format!("Switched to {} {}", target_icon, target));
}

fn cmd_history() {
    println!("{}📜 Profile History{}", CYAN, NC);
    println!();
    
    if let Ok(file) = fs::File::open(get_log_file()) {
        let lines: Vec<String> = io::BufReader::new(file)
            .lines()
            .filter_map(|l| l.ok())
            .collect();
        
        for line in lines.iter().rev().take(10).rev() {
            println!("  {}", line);
        }
    } else {
        info("No history yet");
    }
    println!();
}

fn cmd_edit(name: &str) {
    let profile_file = get_profile_dir().join(format!("{}.profile", name));
    
    if !profile_file.exists() {
        error(&format!("Profile '{}' not found", name));
    }
    
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
    Command::new(&editor)
        .arg(&profile_file)
        .status()
        .ok();
}

fn cmd_export(name: &str) {
    let profile_dir = get_profile_dir();
    let profile_file = profile_dir.join(format!("{}.profile", name));
    
    if !profile_file.exists() {
        error(&format!("Profile '{}' not found", name));
    }
    
    let export_dir = profile_dir.join("exports");
    fs::create_dir_all(&export_dir).ok();
    
    let export_file = export_dir.join(format!("{}.profile", name));
    
    let timestamp = get_timestamp();
    let home = env::var("HOME").expect("HOME not set");
    let version = fs::read_to_string(PathBuf::from(&home).join("0-core/VERSION"))
        .unwrap_or_else(|_| "unknown".to_string());
    let system = fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|c| {
            c.lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .map(|l| l.replace("PRETTY_NAME=", "").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());
    
    let original = fs::read_to_string(&profile_file).unwrap_or_default();
    
    let header = format!(
        "# ═══════════════════════════════════════════════════════════\n\
         # 0-Core Profile Export\n\
         # Name: {}\n\
         # Exported: {}\n\
         # System: {}\n\
         # Version: {}\n\
         # ═══════════════════════════════════════════════════════════\n\n",
        name, timestamp, system, version.trim()
    );
    
    fs::write(&export_file, format!("{}{}", header, original)).ok();
    
    success(&format!("Exported to: {}", export_file.display()));
    info("Share this file with others!");
}

fn cmd_import(import_path: &str) {
    let import_file = PathBuf::from(import_path);
    
    if !import_file.exists() {
        error(&format!("File not found: {}", import_path));
    }
    
    let name = import_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported")
        .to_string();
    
    let dest = get_profile_dir().join(format!("{}.profile", name));
    
    if dest.exists() {
        warn(&format!("Profile '{}' already exists", name));
        print!("Overwrite? [y/N] ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        if !input.trim().eq_ignore_ascii_case("y") {
            info("Import cancelled");
            return;
        }
    }
    
    let content = fs::read_to_string(&import_file).unwrap_or_default();
    
    // Strip export header if present
    let final_content = if content.contains("# 0-Core Profile Export") {
        content.lines().skip(8).collect::<Vec<&str>>().join("\n")
    } else {
        content
    };
    
    fs::write(&dest, final_content).ok();
    
    success(&format!("Imported profile: {}", name));
    info(&format!("Try it: profile {}", name));
}

fn cmd_help() {
    println!("{}profile v{}{} - System Profile Manager", CYAN, VERSION, NC);
    println!();
    println!("{}Usage:{}", YELLOW, NC);
    println!("  profile <name>           Switch to profile");
    println!("  profile list             List available profiles");
    println!("  profile status           Show current profile and system state");
    println!("  profile history          Show recent profile switches");
    println!("  profile health           Run health check");
    println!("  profile edit <name>      Edit a profile");
    println!("  profile export <name>    Export profile for sharing");
    println!("  profile import <file>    Import community profile");
    println!("  profile --version        Show version");
    println!("  profile help             Show this help");
    println!();
    println!("{}Profiles:{}", YELLOW, NC);
    println!("  default     🏠  Balanced daily driver");
    println!("  gaming      🎮  GPU performance, notifications off");
    println!("  work        💼  VPN on, focus mode");
    println!("  low-power   🔋  Battery optimization");
    println!();
    println!("{}The Omega Legacy - You control your system.{} 🎮", BLUE, NC);
}

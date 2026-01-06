use std::env;
use std::fs;
use std::io::{self, Write};
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
    let home = env::var("HOME").expect("HOME not set");
    let core_dir = PathBuf::from(&home).join("0-core");
    
    if args.len() < 2 {
        show_help();
        return;
    }
    
    match args[1].as_str() {
        "lock" => cmd_lock(&core_dir),
        "unlock" => cmd_unlock(&core_dir),
        "status" => cmd_status(&core_dir),
        "edit" => {
            if args.len() < 3 {
                eprintln!("Usage: core-protect edit <package-name>");
                eprintln!("Example: core-protect edit shell-zsh");
                process::exit(1);
            }
            cmd_edit(&core_dir, &args[2]);
        }
        _ => show_help(),
    }
}

fn cmd_lock(core_dir: &PathBuf) {
    println!("🔒 Locking 0-core (immutable protection)...");
    
    // Lock all items in core_dir
    if let Ok(entries) = fs::read_dir(core_dir) {
        for entry in entries.flatten() {
            Command::new("sudo")
                .args(["chattr", "+i"])
                .arg(entry.path())
                .status()
                .ok();
        }
    }
    
    // Lock the directory itself
    Command::new("sudo")
        .args(["chattr", "+i"])
        .arg(core_dir)
        .status()
        .ok();
    
    println!("{}✅ Core protected! Cannot modify without unlocking.{}", GREEN, NC);
}

fn cmd_unlock(core_dir: &PathBuf) {
    println!("🔓 Unlocking 0-core for editing...");
    
    // Unlock all items first
    if let Ok(entries) = fs::read_dir(core_dir) {
        for entry in entries.flatten() {
            Command::new("sudo")
                .args(["chattr", "-i"])
                .arg(entry.path())
                .status()
                .ok();
        }
    }
    
    // Unlock the directory
    Command::new("sudo")
        .args(["chattr", "-i"])
        .arg(core_dir)
        .status()
        .ok();
    
    println!("{}✅ Core unlocked! You can now edit.{}", GREEN, NC);
}

fn cmd_status(core_dir: &PathBuf) {
    println!("📊 Checking 0-core protection status...");
    
    let output = Command::new("lsattr")
        .arg("-d")
        .arg(core_dir)
        .output();
    
    if let Ok(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        if stdout.contains('i') {
            println!("🔒 Core is LOCKED (immutable)");
        } else {
            println!("🔓 Core is UNLOCKED (editable)");
        }
    } else {
        println!("❓ Could not determine status");
    }
}

fn cmd_edit(core_dir: &PathBuf, package: &str) {
    let pkg_dir = core_dir.join(package);
    
    if !pkg_dir.exists() {
        eprintln!("{}❌ Package not found: {}{}", RED, package, NC);
        process::exit(1);
    }
    
    let blast_radius = get_blast_radius(core_dir, package);
    
    if !show_blast_warning(core_dir, package, &blast_radius) {
        return;
    }
    
    create_backup(core_dir, package, &blast_radius);
    
    println!("🔓 Temporarily unlocking for edit...");
    cmd_unlock(core_dir);
    
    println!("📝 Opening editor...");
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
    Command::new(&editor)
        .arg(".")
        .current_dir(&pkg_dir)
        .status()
        .ok();
    
    println!("🔒 Re-locking core...");
    cmd_lock(core_dir);
    
    println!("{}✅ Edits complete, core re-locked!{}", GREEN, NC);
}

fn get_blast_radius(core_dir: &PathBuf, package: &str) -> String {
    let dotmeta = core_dir.join(package).join(".dotmeta");
    
    if let Ok(content) = fs::read_to_string(&dotmeta) {
        for line in content.lines() {
            if line.contains("blast_radius") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line.rfind('"') {
                        if start < end {
                            return line[start + 1..end].to_string();
                        }
                    }
                }
            }
        }
    }
    "unknown".to_string()
}

fn get_failure_modes(core_dir: &PathBuf, package: &str) -> Vec<String> {
    let dotmeta = core_dir.join(package).join(".dotmeta");
    let mut modes = vec![];
    
    if let Ok(content) = fs::read_to_string(&dotmeta) {
        let mut in_section = false;
        for line in content.lines() {
            if line.contains("[blast_impact]") {
                in_section = true;
                continue;
            }
            if line.starts_with('[') && !line.contains("[blast_impact]") {
                in_section = false;
            }
            if in_section && line.contains("failure_modes") {
                continue;
            }
            if in_section && line.trim().starts_with('"') {
                let clean = line.trim()
                    .trim_matches(|c| c == '"' || c == ',' || c == '[' || c == ']')
                    .trim()
                    .to_string();
                if !clean.is_empty() {
                    modes.push(clean);
                }
            }
        }
    }
    modes
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().to_string()
}

fn show_blast_warning(core_dir: &PathBuf, package: &str, blast_radius: &str) -> bool {
    let failure_modes = get_failure_modes(core_dir, package);
    
    match blast_radius {
        "critical" => {
            println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", RED, NC);
            println!("{}⚠️  CRITICAL BLAST RADIUS COMPONENT{}", RED, NC);
            println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", RED, NC);
            println!();
            println!("Package: {}{}{}", CYAN, package, NC);
            println!("Risk: {}🔴 Critical{} (system unusable if broken)", RED, NC);
            println!();
            println!("Failure may cause:");
            for mode in &failure_modes {
                println!("  {}•{} {}", RED, NC, mode);
            }
            println!();
            println!("{}⚠️  Auto-backup will be created before editing{}", YELLOW, NC);
            println!();
            
            let confirm = prompt("Type 'CRITICAL' to proceed: ");
            if confirm != "CRITICAL" {
                println!("❌ Edit cancelled");
                return false;
            }
        }
        "high" => {
            println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", YELLOW, NC);
            println!("{}⚠️  HIGH BLAST RADIUS COMPONENT{}", YELLOW, NC);
            println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", YELLOW, NC);
            println!();
            println!("Package: {}{}{}", CYAN, package, NC);
            println!("Risk: {}🟠 High{} (major functionality affected)", YELLOW, NC);
            println!();
            println!("Failure may cause:");
            for mode in &failure_modes {
                println!("  {}•{} {}", YELLOW, NC, mode);
            }
            println!();
            println!("{}⚠️  Auto-backup will be created before editing{}", YELLOW, NC);
            println!();
            
            let confirm = prompt("Type 'yes' to proceed: ");
            if confirm != "yes" {
                println!("❌ Edit cancelled");
                return false;
            }
        }
        "medium" => {
            println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", BLUE, NC);
            println!("{}ℹ️  MEDIUM BLAST RADIUS COMPONENT{}", BLUE, NC);
            println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", BLUE, NC);
            println!();
            println!("Package: {}{}{}", CYAN, package, NC);
            println!("Risk: {}🔵 Medium{} (important but not essential)", BLUE, NC);
            println!();
            
            let confirm = prompt("Continue? (y/N): ");
            if confirm != "y" {
                println!("❌ Edit cancelled");
                return false;
            }
        }
        _ => {}
    }
    
    true
}

fn create_backup(core_dir: &PathBuf, package: &str, blast_radius: &str) {
    if blast_radius == "critical" || blast_radius == "high" {
        println!("💾 Creating backup...");
        
        Command::new("git")
            .args(["-C", &core_dir.to_string_lossy(), "add", "-A"])
            .status()
            .ok();
        
        let timestamp = Command::new("date")
            .arg("+%Y-%m-%d-%H%M")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        
        let msg = format!("Pre-edit backup: {} {}", package, timestamp);
        Command::new("git")
            .args(["-C", &core_dir.to_string_lossy(), "stash", "push", "-m", &msg])
            .status()
            .ok();
        
        println!("{}✅ Backup created (git stash){}", GREEN, NC);
        println!();
    }
}

fn show_help() {
    println!("🛡️  Core Protection - Immutable 0-core Management");
    println!();
    println!("Usage:");
    println!("  core-protect lock          Lock 0-core (prevent changes)");
    println!("  core-protect unlock        Unlock 0-core (allow changes)");
    println!("  core-protect status        Check protection status");
    println!("  core-protect edit <pkg>    Unlock, edit, re-lock (with blast radius check)");
    println!();
    println!("Examples:");
    println!("  core-protect lock");
    println!("  core-protect edit shell-zsh");
    println!("  core-protect status");
}

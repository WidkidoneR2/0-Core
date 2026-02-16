use clap::{Parser, Subcommand};
use colored::*;
use faelight_core::paths;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};

const VERSION: &str = "2.1.0";

#[derive(Parser)]
#[command(name = "core-protect")]
#[command(about = "🛡️  System Guardian - Immutable protection for 0-core", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lock 0-core (prevent changes)
    Lock,
    /// Unlock 0-core (allow changes)
    Unlock,
    /// Check protection status
    Status,
    /// Health check
    Health,
    /// Edit a package safely
    Edit { package: String },
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let core_dir = paths::core_dir();

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
        "--version" | "-v" => {
            println!("core-protect v{}", VERSION);
        }
        "--health" => cmd_health(&core_dir),
        "--help" | "-h" => show_help(),
        _ => show_help(),
    }
}

fn cmd_health(core_dir: &PathBuf) {
    println!();
    println!(
        "{}",
        format!("🏥 core-protect v{} - Health Check", VERSION).cyan()
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut healthy = true;

    // Check chattr available
    print!("  Checking chattr command... ");
    match Command::new("which").arg("chattr").output() {
        Ok(output) if output.status.success() => println!("{}", "✅".green()),
        _ => {
            println!("{}", "❌ chattr not found".red());
            healthy = false;
        }
    }

    // Check lsattr available
    print!("  Checking lsattr command... ");
    match Command::new("which").arg("lsattr").output() {
        Ok(output) if output.status.success() => println!("{}", "✅".green()),
        _ => {
            println!("{}", "❌ lsattr not found".red());
            healthy = false;
        }
    }

    // Check 0-core exists
    print!("  Checking 0-core directory... ");
    if core_dir.exists() {
        println!("{}", format!("✅ {}", core_dir.display()).green());
    } else {
        println!(
            "{}",
            format!("❌ not found at {}", core_dir.display()).red()
        );
        healthy = false;
    }

    // Check sudo access
    print!("  Checking sudo access... ");
    match Command::new("sudo").args(["-n", "true"]).status() {
        Ok(status) if status.success() => println!("{}", "✅".green()),
        _ => {
            println!("{}", "⚠️  sudo may require password".yellow());
        }
    }

    // Check current protection status
    print!("  Checking protection status... ");
    let output = Command::new("lsattr").arg("-d").arg(core_dir).output();

    if let Ok(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        if stdout.contains('i') {
            println!("{}", "🔒 LOCKED".green());
        } else {
            println!("{}", "🔓 UNLOCKED".yellow());
        }
    } else {
        println!("{}", "❓ Unknown".yellow());
    }

    println!();
    if healthy {
        println!("{}", "✅ All systems operational".green());
        process::exit(0);
    } else {
        println!("{}", "❌ System unhealthy".red());
        process::exit(1);
    }
}

fn cmd_lock(core_dir: &PathBuf) {
    println!("🔒 Locking 0-core (immutable protection)...");

    // Lock all items in core_dir (silently skip unsupported files)
    if let Ok(entries) = fs::read_dir(core_dir) {
        for entry in entries.flatten() {
            Command::new("sudo")
                .args(["chattr", "+i"])
                .arg(entry.path())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .ok();
        }
    }

    // Lock the directory itself
    Command::new("sudo")
        .args(["chattr", "+i"])
        .arg(core_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok();

    println!(
        "{}",
        "✅ Core protected! Cannot modify without unlocking.".green()
    );
}

fn cmd_unlock(core_dir: &PathBuf) {
    println!("🔓 Unlocking 0-core for editing...");

    // Unlock all items first (silently skip unsupported files)
    if let Ok(entries) = fs::read_dir(core_dir) {
        for entry in entries.flatten() {
            Command::new("sudo")
                .args(["chattr", "-i"])
                .arg(entry.path())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .ok();
        }
    }

    // Unlock the directory
    Command::new("sudo")
        .args(["chattr", "-i"])
        .arg(core_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok();

    println!("{}", "✅ Core unlocked! You can now edit.".green());
}

fn cmd_status(core_dir: &PathBuf) {
    println!("📊 Checking 0-core protection status...");

    let output = Command::new("lsattr").arg("-d").arg(core_dir).output();

    if let Ok(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        // Only check the flags section (first 20 chars), not the path
        let flags = stdout.chars().take(20).collect::<String>();
        if flags.contains('i') {
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
        eprintln!("{}", format!("❌ Package not found: {}", package).red());
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

    println!("{}", "✅ Edits complete, core re-locked!".green());
}

fn get_blast_radius(core_dir: &Path, package: &str) -> String {
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

fn get_failure_modes(core_dir: &Path, package: &str) -> Vec<String> {
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
                let clean = line
                    .trim()
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

fn show_blast_warning(core_dir: &Path, package: &str, blast_radius: &str) -> bool {
    let failure_modes = get_failure_modes(core_dir, package);

    match blast_radius {
        "critical" => {
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red());
            println!("{}", "⚠️  CRITICAL BLAST RADIUS COMPONENT".red());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red());
            println!();
            println!("{}", format!("Package: {}", package).cyan());
            println!("{}", "Risk: 🔴 Critical (system unusable if broken)".red());
            println!();
            println!("Failure may cause:");
            for mode in &failure_modes {
                println!("  {} {}", "•".red(), mode);
            }
            println!();
            println!(
                "{}",
                "⚠️  Auto-backup will be created before editing".yellow()
            );
            println!();

            let confirm = prompt("Type 'CRITICAL' to proceed: ");
            if confirm != "CRITICAL" {
                println!("❌ Edit cancelled");
                return false;
            }
        }
        "high" => {
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".yellow());
            println!("{}", "⚠️  HIGH BLAST RADIUS COMPONENT".yellow());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".yellow());
            println!();
            println!("{}", format!("Package: {}", package).cyan());
            println!(
                "{}",
                "Risk: 🟠 High (major functionality affected)".yellow()
            );
            println!();
            println!("Failure may cause:");
            for mode in &failure_modes {
                println!("  {} {}", "•".yellow(), mode);
            }
            println!();
            println!(
                "{}",
                "⚠️  Auto-backup will be created before editing".yellow()
            );
            println!();

            let confirm = prompt("Type 'yes' to proceed: ");
            if confirm != "yes" {
                println!("❌ Edit cancelled");
                return false;
            }
        }
        "medium" => {
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".blue());
            println!("{}", "ℹ️  MEDIUM BLAST RADIUS COMPONENT".blue());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".blue());
            println!();
            println!("{}", format!("Package: {}", package).cyan());
            println!("{}", "Risk: 🔵 Medium (important but not essential)".blue());
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

fn create_backup(core_dir: &Path, package: &str, blast_radius: &str) {
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
            .args([
                "-C",
                &core_dir.to_string_lossy(),
                "stash",
                "push",
                "-m",
                &msg,
            ])
            .status()
            .ok();

        println!("{}", "✅ Backup created (git stash)".green());
        println!();
    }
}

fn show_help() {
    println!(
        "🛡️  core-protect v{} - Immutable 0-core Management",
        VERSION
    );
    println!();
    println!("USAGE:");
    println!("  core-protect <command>");
    println!();
    println!("COMMANDS:");
    println!("  lock              Lock 0-core (prevent changes)");
    println!("  unlock            Unlock 0-core (allow changes)");
    println!("  status            Check protection status");
    println!("  edit <package>    Unlock, edit, re-lock (with blast radius check)");
    println!("  --health          Run health check");
    println!("  --version, -v     Show version");
    println!("  --help, -h        Show this help");
    println!();
    println!("EXAMPLES:");
    println!("  core-protect lock");
    println!("  core-protect edit shell-zsh");
    println!("  core-protect status");
    println!("  core-protect --health");
}

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use faelight_core::paths;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};

mod audit;

const VERSION: &str = "2.3.0";

#[derive(Parser)]
#[command(name = "core-protect")]
#[command(version = VERSION)]
#[command(about = "🛡️  System Guardian - Immutable protection for 0-core")]
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
    /// Verify all files have +i immutable flag
    Verify,
    /// Run health check
    Health,
    /// Show audit log of lock/unlock events
    Audit,

    /// Edit a package safely (unlock, edit, relock)
    Edit {
        /// Package name to edit
        package: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let core_dir = paths::core_dir();

    let result = match cli.command {
        Commands::Lock => {
            cmd_lock(&core_dir);
            Ok(())
        }
        Commands::Unlock => {
            cmd_unlock(&core_dir);
            Ok(())
        }
        Commands::Status => {
            cmd_status(&core_dir);
            Ok(())
        }
        Commands::Verify => cmd_verify(&core_dir),
        Commands::Health => {
            cmd_health(&core_dir);
            Ok(())
        }
        Commands::Audit => {
            cmd_audit();
            Ok(())
        }
        Commands::Edit { package } => {
            cmd_edit(&core_dir, &package);
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{}", format!("❌ Error: {}", e).red());
        process::exit(1);
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

    print!("  Checking chattr command... ");
    match Command::new("which").arg("chattr").output() {
        Ok(output) if output.status.success() => println!("{}", "✅".green()),
        _ => {
            println!("{}", "❌ chattr not found".red());
            healthy = false;
        }
    }

    print!("  Checking lsattr command... ");
    match Command::new("which").arg("lsattr").output() {
        Ok(output) if output.status.success() => println!("{}", "✅".green()),
        _ => {
            println!("{}", "❌ lsattr not found".red());
            healthy = false;
        }
    }

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

    print!("  Checking sudo access... ");
    match Command::new("sudo").args(["-n", "true"]).status() {
        Ok(status) if status.success() => println!("{}", "✅".green()),
        _ => println!("{}", "⚠️  sudo may require password".yellow()),
    }

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

fn cmd_lock(core_dir: &Path) {
    println!("🔒 Locking 0-core (immutable protection)...");
    audit::log_event("LOCK");

    lock_recursive(core_dir, "+i");

    println!(
        "{}",
        "✅ Core protected! Cannot modify without unlocking.".green()
    );
}

fn cmd_unlock(core_dir: &Path) {
    println!("🔓 Unlocking 0-core for editing...");
    audit::log_event("UNLOCK");

    lock_recursive(core_dir, "-i");

    println!("{}", "✅ Core unlocked! You can now edit.".green());
}

fn lock_recursive(dir: &Path, flag: &str) {
    // INT-251: lock only source dirs. SKIP runtime/, bin/, scripts/, target/, BACKUPS/, .git/
    // because these are working directories that need continuous write access from daemons.
    // Locking runtime/state.db caused readonly warnings every time lock-core was used.
    let lockable = [
        "00-meta", "01-registry", "02-rules", "03-interfaces", "04-schema",
        "docs", "engine", "intents", "rust-tools", "status-blocks",
        "Cargo.lock", "Cargo.toml", "README.md", "TOOLS.md", "VERSION",
    ];
    for entry in &lockable {
        let path = dir.join(entry);
        if path.exists() {
            Command::new("sudo")
                .args(["chattr", "-R", flag])
                .arg(&path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .ok();
        }
    }
}

fn cmd_verify(core_dir: &Path) -> Result<()> {
    println!("{}", "🔍 Verifying immutable flags on 0-core...".cyan());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut total = 0;
    let mut protected = 0;
    let mut unprotected: Vec<PathBuf> = Vec::new();

    verify_recursive(core_dir, &mut total, &mut protected, &mut unprotected);

    println!();
    println!("  Total checked:   {}", total);
    println!("  Protected (+i):  {}", format!("{}", protected).green());
    println!(
        "  Unprotected:     {}",
        if unprotected.is_empty() {
            "0".green()
        } else {
            format!("{}", unprotected.len()).red()
        }
    );

    if !unprotected.is_empty() {
        println!();
        println!("{}", "⚠️  Files missing immutable flag:".yellow());
        for path in &unprotected {
            println!("  {}", path.display().to_string().yellow());
        }
        println!();
        println!("{}", "💡 Run: core-protect lock  to re-protect".cyan());
    } else {
        println!();
        println!("{}", "✅ All files properly protected!".green());
    }

    Ok(())
}

fn verify_recursive(
    dir: &Path,
    total: &mut usize,
    protected: &mut usize,
    unprotected: &mut Vec<PathBuf>,
) {
    let output = Command::new("lsattr").arg("-d").arg(dir).output();

    if let Ok(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        *total += 1;
        if stdout.contains('i') {
            *protected += 1;
        } else {
            unprotected.push(dir.to_path_buf());
        }
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            let output = Command::new("lsattr").arg("-d").arg(&path).output();

            if let Ok(o) = output {
                let stdout = String::from_utf8_lossy(&o.stdout);
                *total += 1;
                if stdout.contains('i') {
                    *protected += 1;
                } else {
                    unprotected.push(path.clone());
                }
            }

            if path.is_dir() {
                verify_recursive(&path, total, protected, unprotected);
            }
        }
    }
}

fn cmd_audit() {
    use colored::Colorize;
    println!("{}", "📋 core-protect Audit Log".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!(
        "  Log: {}",
        audit::audit_log_path().display().to_string().dimmed()
    );
    println!();
    audit::show_log();
}

fn cmd_status(core_dir: &PathBuf) {
    println!("📊 Checking 0-core protection status...");

    let output = Command::new("lsattr").arg("-d").arg(core_dir).output();

    if let Ok(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        let flags = stdout.chars().take(20).collect::<String>();
        if flags.contains('i') {
            println!("{}", "🔒 Core is LOCKED (immutable)".green());
        } else {
            println!("{}", "🔓 Core is UNLOCKED (editable)".yellow());
        }
    } else {
        println!("{}", "❓ Could not determine status".yellow());
    }
}

fn cmd_edit(core_dir: &Path, package: &str) {
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

    audit::log_event(&format!("EDIT package={}", package));
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
            if !failure_modes.is_empty() {
                println!("Failure may cause:");
                for mode in &failure_modes {
                    println!("  {} {}", "•".red(), mode);
                }
                println!();
            }
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
            if !failure_modes.is_empty() {
                println!("Failure may cause:");
                for mode in &failure_modes {
                    println!("  {} {}", "•".yellow(), mode);
                }
                println!();
            }
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

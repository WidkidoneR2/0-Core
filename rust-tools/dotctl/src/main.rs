use clap::{Parser, Subcommand};
use faelight_core::paths;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};

// ANSI colors (keeping for now - colored ready for future)
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const CYAN: &str = "\x1b[0;36m";
const BLUE: &str = "\x1b[0;34m";
const NC: &str = "\x1b[0m";

// ANSI colors

const VERSION: &str = "3.1.0";

#[derive(Parser)]
#[command(name = "dotctl")]
#[command(about = "🎮 Dotfile Control Center - Manage stow packages", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show status of all stow packages
    Status,
    /// Bump package version
    Bump { args: Vec<String> },
    /// Show version history
    History { args: Vec<String> },
    /// Health check
    Health,
    /// Show version
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => cmd_status(),
        Commands::Bump { args } => cmd_bump(&args),
        Commands::History { args } => cmd_history(&args),
        Commands::Health => cmd_health(),
        Commands::Version => cmd_version(),
    }
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
                version = line
                    .split('=')
                    .nth(1)
                    .unwrap_or("?")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            } else if line.starts_with("category = ") {
                category = line
                    .split('=')
                    .nth(1)
                    .unwrap_or("misc")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            } else if line.starts_with("blast_radius = ") {
                blast = line
                    .split('=')
                    .nth(1)
                    .unwrap_or("low")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            } else if line.starts_with("description = ") {
                description = line
                    .split('=')
                    .nth(1)
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
    let core_dir = paths::core_dir();
    let stow_dir = paths::stow_dir();
    let home = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/home".to_string()));

    // Header with box
    println!("╭─────────────────────────────────────────────────╮");
    println!("│ 🎮 Dotfile Control Center                      │");

    // System version
    let version_file = core_dir.join("VERSION");
    if let Ok(version) = fs::read_to_string(&version_file) {
        println!("│ System: v{:<38} │", version.trim());
    }
    println!("╰─────────────────────────────────────────────────╯");
    println!();

    // Current zone
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let (zone_enum, _) = faelight_zone::current_zone(&cwd, &home);
    println!(
        "  Current Zone: {} {}",
        zone_enum.icon(),
        zone_enum.short_label()
    );
    println!();

    // Packages
    println!("{}📦 Packages:{}", BLUE, NC);
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

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

                // Detect package zone
                let pkg_path = entry.path();
                let (pkg_zone, _) = faelight_zone::current_zone(&pkg_path, &home);
                let zone_icon = pkg_zone.icon();

                let blast_icon = match blast.as_str() {
                    "critical" => format!("{}🔴{}", RED, NC),
                    "high" => format!("{}🟡{}", YELLOW, NC),
                    "medium" => format!("{}🔵{}", BLUE, NC),
                    _ => format!("{}🟢{}", GREEN, NC),
                };

                println!(
                    "  {} {} {:<22} v{:<8} {}",
                    zone_icon, blast_icon, pkg_name, version, category
                );
            }
        }
    }

    println!();

    // Health with better display
    println!("{}🏥 System Health:{}", BLUE, NC);
    if let Ok(output) = Command::new("dot-doctor").arg("--quiet").output() {
        let exit_code = output.status.code().unwrap_or(1);
        if exit_code == 0 {
            println!("  {}✅ All checks passed{}", GREEN, NC);
        } else {
            // Run with output to get percentage
            if let Ok(full) = Command::new("dot-doctor").output() {
                let stdout = String::from_utf8_lossy(&full.stdout);
                if let Some(line) = stdout.lines().find(|l| l.contains("Health:")) {
                    let health = line.split("Health:").nth(1).unwrap_or("?").trim();
                    println!("  {}⚠️  {}{}", YELLOW, health, NC);
                }
            }
        }
    }

    println!();
}
fn cmd_bump(args: &[String]) {
    if args.len() < 2 {
        eprintln!(
            "{}Usage:{} dotctl bump <package> <version> [message]",
            YELLOW, NC
        );
        process::exit(1);
    }

    let pkg_name = &args[0];
    let new_version = &args[1];
    let message = args.get(2).map(|s| s.as_str()).unwrap_or("Version bump");

    let stow_dir = paths::stow_dir();
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
        content
            .lines()
            .map(|line| {
                if line.trim().starts_with("version = ") {
                    format!("version = \"{}\"", new_version)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        // Simple format
        content
            .lines()
            .map(|line| {
                if line.trim().starts_with("version:") {
                    format!("version: {}", new_version)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
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
    let stow_dir = paths::stow_dir();
    let dotmeta_path = stow_dir.join(pkg_name).join(".dotmeta");

    if !dotmeta_path.exists() {
        eprintln!(
            "{}❌ No .dotmeta found for package:{} {}",
            RED, NC, pkg_name
        );
        process::exit(1);
    }

    let content = fs::read_to_string(&dotmeta_path).expect("Failed to read .dotmeta");

    println!(
        "{}═══════════════════════════════════════════════════════════{}",
        CYAN, NC
    );
    println!("{}📜 Change History: {}{}", CYAN, pkg_name, NC);
    println!(
        "{}═══════════════════════════════════════════════════════════{}",
        CYAN, NC
    );
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

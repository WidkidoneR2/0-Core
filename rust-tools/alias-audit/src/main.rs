use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use faelight_core::paths;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "alias-audit")]
#[command(about = "Audit zsh aliases for duplicates, conflicts, and coverage", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Output format for doctor integration
    #[arg(long)]
    doctor: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Check for duplicate aliases
    Duplicates,
    /// Check for missing tool aliases
    Missing,
    /// Check for conflicts
    Conflicts,
    /// Show all tools with their aliases
    Tools,
}

const EXPECTED_TOOLS: [&str; 38] = [
    // Core Infrastructure (11)
    "dot-doctor",
    "faelight-update",
    "faelight-core",
    "core-protect",
    "safe-update",
    "core-diff",
    "dotctl",
    "entropy-check",
    "intent-guard",
    "faelight-stow",
    "faelight-snapshot",
    // Desktop Environment (9)
    "faelight-fetch",
    "faelight-bar",
    "faelight-launcher",
    "faelight-dmenu",
    "faelight-menu",
    "faelight-notify",
    "faelight-lock",
    "faelight-dashboard",
    "faelight-term",
    // Development (14)
    "intent",
    "archaeology-0-core",
    "workspace-view",
    "faelight-git",
    "faelight-hooks",
    "recent-files",
    "profile",
    "teach",
    "faelight",
    "keyscan",
    "faelight-zone",
    "faelight-fm",
    "faelight-link",
    "faelight-daemon",
    // Version Management (4)
    "bump-system-version",
    "faelight-bootstrap",
    "get-version",
    "latest-update",
];

fn main() -> Result<()> {
    let cli = Cli::parse();
    let aliases_path = paths::aliases_file();

    let aliases = parse_aliases(&aliases_path)?;

    if cli.doctor {
        output_doctor_format(&aliases)?;
        return Ok(());
    }

    match cli.command {
        Some(Commands::Duplicates) => check_duplicates(&aliases),
        Some(Commands::Missing) => check_missing(&aliases),
        Some(Commands::Conflicts) => check_conflicts(&aliases),
        Some(Commands::Tools) => show_tools(&aliases),
        None => run_full_audit(&aliases),
    }
}

fn parse_aliases(path: &PathBuf) -> Result<HashMap<String, String>> {
    let content = fs::read_to_string(path)?;
    let mut aliases = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("alias ") {
            if let Some(rest) = trimmed.strip_prefix("alias ") {
                if let Some((name, target)) = rest.split_once('=') {
                    let target = target.trim_matches('\'').trim_matches('"');
                    aliases.insert(name.to_string(), target.to_string());
                }
            }
        }
    }

    Ok(aliases)
}

fn check_duplicates(aliases: &HashMap<String, String>) -> Result<()> {
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();

    for name in aliases.keys() {
        if !seen.insert(name) {
            duplicates.push(name);
        }
    }

    if duplicates.is_empty() {
        println!("{}", "✅ No duplicate aliases found!".green().bold());
    } else {
        println!("{}", "❌ DUPLICATES FOUND:".red().bold());
        for dup in duplicates {
            println!("  {}", dup.yellow());
        }
    }

    Ok(())
}

fn check_missing(aliases: &HashMap<String, String>) -> Result<()> {
    println!("{}", "🔍 Checking tool coverage...".cyan().bold());
    println!();

    let mut missing = Vec::new();

    for tool in EXPECTED_TOOLS {
        // Skip daemon (background service)
        if tool == "faelight-daemon" {
            continue;
        }

        let has_alias = aliases.values().any(|v| v.contains(tool));

        // Skip faelight-core (library, not a binary)
        if tool == "faelight-core" {
            continue;
        }
        if !has_alias {
            missing.push(tool);
        }
    }

    if missing.is_empty() {
        println!(
            "{}",
            "✅ All 37 tools have aliases! (daemon excluded)"
                .green()
                .bold()
        );
    } else {
        println!(
            "{}",
            format!("❌ {} tools missing aliases:", missing.len())
                .red()
                .bold()
        );
        for tool in missing {
            println!("  {}", tool.yellow());
        }
    }

    Ok(())
}

fn check_conflicts(aliases: &HashMap<String, String>) -> Result<()> {
    let mut tool_aliases: HashMap<String, Vec<String>> = HashMap::new();

    for (alias, target) in aliases {
        tool_aliases
            .entry(target.clone())
            .or_default()
            .push(alias.clone());
    }

    println!("{}", "🔍 Checking for conflicts...".cyan().bold());
    println!();

    let mut has_conflicts = false;
    for (tool, alias_list) in tool_aliases {
        if alias_list.len() > 5 {
            println!(
                "{} {}",
                "⚠️ ".yellow(),
                format!("{} has {} aliases", tool, alias_list.len()).yellow()
            );
            has_conflicts = true;
        }
    }

    if !has_conflicts {
        println!("{}", "✅ No excessive aliasing detected!".green().bold());
    }

    Ok(())
}

fn show_tools(aliases: &HashMap<String, String>) -> Result<()> {
    println!("{}", "🌲 FAELIGHT TOOLS ALIAS COVERAGE".cyan().bold());
    println!("{}", "═".repeat(60));
    println!();

    for tool in EXPECTED_TOOLS {
        let tool_aliases: Vec<&String> = aliases
            .iter()
            .filter(|(_, v)| v.contains(tool))
            .map(|(k, _)| k)
            .collect();

        if tool == "faelight-daemon" {
            println!("{} {}", "N/A".dimmed(), tool.dimmed());
        } else if tool_aliases.is_empty() {
            println!("{} {}", "❌".red(), tool);
        } else {
            println!(
                "{} {} → {}",
                "✅".green(),
                tool,
                tool_aliases
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .cyan()
            );
        }
    }

    Ok(())
}

fn run_full_audit(aliases: &HashMap<String, String>) -> Result<()> {
    // Get current zone
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let home = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/home".to_string()));
    let (zone_enum, _) = faelight_zone::current_zone(&cwd, &home);

    // Header with box
    println!("╭─────────────────────────────────────────────────╮");
    println!("│ 🔍 Alias Audit - Full Check                    │");
    println!("╰─────────────────────────────────────────────────╯");
    println!();

    // Current zone
    println!(
        "  Current Zone: {} {}",
        zone_enum.icon(),
        zone_enum.short_label()
    );
    println!();

    // Check duplicates
    println!("{}", "📋 Checking for duplicates...".bold());
    check_duplicates(aliases)?;
    println!();

    // Check coverage
    println!("{}", "📦 Checking tool coverage...".bold());
    check_missing(aliases)?;
    println!();

    // Summary with box
    println!("╭─────────────────────────────────────────────────╮");
    println!("│ 📊 Total aliases: {:<30} │", aliases.len());
    println!("│ {} Audit complete!{:<29} │", "✅".green().bold(), "");
    println!("╰─────────────────────────────────────────────────╯");

    Ok(())
}
fn output_doctor_format(aliases: &HashMap<String, String>) -> Result<()> {
    // Check for issues
    let mut missing = Vec::new();
    for tool in EXPECTED_TOOLS {
        if tool == "faelight-daemon" {
            continue;
        }
        let has_alias = aliases.values().any(|v| v.contains(tool));
        // Skip faelight-core (library, not a binary)
        if tool == "faelight-core" {
            continue;
        }
        if !has_alias {
            missing.push(tool);
        }
    }

    if missing.is_empty() {
        println!(
            "✅ Alias Coverage: All 37 tools have aliases ({} total)",
            aliases.len()
        );
    } else {
        println!(
            "⚠️  Alias Coverage: {} tools missing aliases",
            missing.len()
        );
    }

    Ok(())
}

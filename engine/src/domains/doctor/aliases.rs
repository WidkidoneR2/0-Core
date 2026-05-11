//! Alias audit — absorbed from rust-tools/alias-audit
use crate::errors::CoreResult;
use colored::*;
use faelight_core::paths;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const EXPECTED_TOOLS: &[&str] = &[
    // Core Infrastructure
    "dot-doctor",
    "faelight-update",
    "faelight-core",
    "core-protect",
    "safe-update",
    "core-diff",
    "dotctl",
    "entropy-check",
    "intent-guard",
    "faelight-snapshot",
    // Desktop Environment
    "faelight-fetch",
    "faelight-bar",
    "faelight-menu",
    "faelight-notify",
    "faelight-lock",
    "faelight-dashboard",
    "faelight-term",
    // Development
    "intent",
    "workspace-view",
    "faelight-git",
    "faelight-hooks",
    "profile",
    "teach",
    "faelight",
    "keyscan",
    "faelight-zone",
    "faelight-fm",
    "faelight-link",
    "faelight-daemon",
    "faelight-palette",
    // Version Management
    "bump-system-version",
    "faelight-bootstrap",
    "get-version",
    "latest-update",
];

pub fn parse_aliases(path: &PathBuf) -> CoreResult<HashMap<String, String>> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut aliases = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("alias ") {
            if let Some((name, target)) = rest.split_once('=') {
                let target = target.trim_matches('\'').trim_matches('"');
                aliases.insert(name.to_string(), target.to_string());
            }
        }
    }
    Ok(aliases)
}

pub fn run_full_audit(subcmd: Option<&str>) -> CoreResult<()> {
    let aliases_path = paths::aliases_file();
    let aliases = parse_aliases(&aliases_path)?;

    match subcmd {
        Some("duplicates") => check_duplicates(&aliases),
        Some("missing") => check_missing(&aliases),
        Some("conflicts") => check_conflicts(&aliases),
        Some("tools") => show_tools(&aliases),
        Some("--doctor") => output_doctor_format(&aliases),
        _ => run_default(&aliases),
    }
}

fn check_duplicates(aliases: &HashMap<String, String>) -> CoreResult<()> {
    let mut seen = std::collections::HashSet::new();
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

fn check_missing(aliases: &HashMap<String, String>) -> CoreResult<()> {
    println!("{}", "🔍 Checking tool coverage...".cyan().bold());
    let mut missing = Vec::new();
    for tool in EXPECTED_TOOLS {
        if *tool == "faelight-daemon" || *tool == "faelight-core" {
            continue;
        }
        if !aliases.values().any(|v| v.contains(tool)) {
            missing.push(tool);
        }
    }
    if missing.is_empty() {
        println!("{}", "✅ All tools have aliases!".green().bold());
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

fn check_conflicts(aliases: &HashMap<String, String>) -> CoreResult<()> {
    let mut tool_aliases: HashMap<String, Vec<String>> = HashMap::new();
    for (alias, target) in aliases {
        tool_aliases
            .entry(target.clone())
            .or_default()
            .push(alias.clone());
    }
    let mut has_conflicts = false;
    for (tool, alias_list) in &tool_aliases {
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

fn show_tools(aliases: &HashMap<String, String>) -> CoreResult<()> {
    println!("{}", "🌲 FAELIGHT TOOLS ALIAS COVERAGE".cyan().bold());
    println!("{}", "═".repeat(60));
    for tool in EXPECTED_TOOLS {
        let tool_aliases: Vec<&String> = aliases
            .iter()
            .filter(|(_, v)| v.contains(tool))
            .map(|(k, _)| k)
            .collect();
        if *tool == "faelight-daemon" {
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

fn run_default(aliases: &HashMap<String, String>) -> CoreResult<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let home = PathBuf::from(std::env::var("HOME").unwrap_or_default());
    let (zone_enum, _) = faelight_zone::current_zone(&cwd, &home);
    println!("╭─────────────────────────────────────────────────╮");
    println!("│ 🔍 Alias Audit - Full Check                    │");
    println!("╰─────────────────────────────────────────────────╯");
    println!(
        "  Current Zone: {} {}",
        zone_enum.icon(),
        zone_enum.short_label()
    );
    println!();
    println!("{}", "📋 Checking for duplicates...".bold());
    check_duplicates(aliases)?;
    println!();
    println!("{}", "📦 Checking tool coverage...".bold());
    check_missing(aliases)?;
    println!();
    println!("╭─────────────────────────────────────────────────╮");
    println!("│ 📊 Total aliases: {:<30} │", aliases.len());
    println!("│ {} Audit complete!{:<29} │", "✅".green().bold(), "");
    println!("╰─────────────────────────────────────────────────╯");
    Ok(())
}

pub fn output_doctor_format(aliases: &HashMap<String, String>) -> CoreResult<()> {
    let mut missing = Vec::new();
    for tool in EXPECTED_TOOLS {
        if *tool == "faelight-daemon" || *tool == "faelight-core" {
            continue;
        }
        if !aliases.values().any(|v| v.contains(tool)) {
            missing.push(tool);
        }
    }
    if missing.is_empty() {
        println!(
            "✅ Alias Coverage: All {} tools have aliases ({} total)",
            EXPECTED_TOOLS.len(),
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

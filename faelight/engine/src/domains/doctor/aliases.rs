//! Alias audit — absorbed from rust-tools/alias-audit
use crate::errors::CoreResult;
use colored::*;
use faelight_core::paths;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Tools that should have an alias, READ FROM THE REGISTRY rather than named here.
///
/// WARNING: this was a hardcoded list of eight. By 2026-08-27 it named a tool deleted
/// the same day (faelight-lock), a subcommand that is not a binary (intent, which is
/// core intent), and a package that is not a cargo target so is never installed
/// (faelight-logout). It reported All 8 tools have aliases while asking only whether
/// an alias EXISTED, never whether the tool did.
///
/// Two consumers also skipped faelight-daemon and faelight-core by name -- exclusions
/// guarding entries the list did not contain.
///
/// Same failure as check_scripts and the faelight-launcher registry entry: a hardcoded
/// census goes stale silently and the check keeps passing. tools.toml knows what
/// exists and carries deployable and retired flags, so removing a tool now updates
/// this check for free.
// INT-192: an unreadable registry is UNKNOWN, not an empty expectation. Returning
// Vec::new() here meant nothing was expected, so nothing could be missing, so the
// check reported clean -- the silent half of this check. (The loud half is
// parse_aliases below, whose unwrap_or_default makes every tool read as missing.)
// The doc comment above was written to kill a census that went stale silently; the
// same disease was left sitting in its own error arm.
pub fn expected_tools() -> faelight_core::check::Checked<Vec<String>> {
    let path = faelight_core::paths::tools_registry();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return Err(faelight_core::check::Skipped::new(
                format!("tools registry at {}", path.display()),
                e,
            ))
        }
    };
    let mut out: Vec<String> = Vec::new();
    let mut name = String::new();
    let mut deployable = false;
    let mut retired = false;
    let mut usage = String::new();
    for line in content.lines() {
        let line = line.trim();
        if line == "[[tool]]" {
            if !name.is_empty() && deployable && !retired && (usage == "high" || usage == "medium")
            {
                out.push(name.clone());
            }
            name.clear();
            deployable = false;
            retired = false;
            usage.clear();
        } else if let Some(v) = line.strip_prefix("name = \"") {
            name = v.trim_end_matches('"').to_string();
        } else if let Some(v) = line.strip_prefix("expected_usage = \"") {
            usage = v.trim_end_matches('"').to_string();
        } else if line == "deployable = true" {
            deployable = true;
        } else if line == "retired = true" {
            retired = true;
        }
    }
    if !name.is_empty() && deployable && !retired && (usage == "high" || usage == "medium") {
        out.push(name);
    }
    Ok(out)
}

// INT-222: this returned CoreResult and then threw the error away. unwrap_or_default turned an
// unreadable config into an EMPTY alias map, so check_alias_coverage found every expected tool
// missing -- and the Status::Fail arm in that check, the one that says "Could not read aliases
// file", was UNREACHABLE because this function could not return Err. A failure branch that
// cannot fire is this intent's thesis in the layer below the check.
// CoreError::Io is #[from] std::io::Error, so `?` carries the real reason up.
pub fn parse_aliases(path: &PathBuf) -> CoreResult<HashMap<String, String>> {
    let content = fs::read_to_string(path)?;
    let mut aliases = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("alias ") {
            if let Some((name, target)) = rest.split_once('=') {
                let name = name.trim().to_string();
                let target = target
                    .trim()
                    .trim_matches('\'')
                    .trim_matches('"')
                    .to_string();
                if !name.is_empty() {
                    aliases.insert(name, target);
                }
            }
        }
    }
    Ok(aliases)
}

pub fn run_full_audit(subcmd: Option<&str>) -> CoreResult<()> {
    let aliases_path = paths::shell_config();
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
    // INT-192: could-not-read must not render as full coverage.
    let expected = match expected_tools() {
        Ok(e) => e,
        Err(skip) => {
            println!("  [??] {}", skip);
            return Ok(());
        }
    };
    for tool in &expected {
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
    // INT-192: could-not-read must not render as an empty tool list.
    let expected = match expected_tools() {
        Ok(e) => e,
        Err(skip) => {
            println!("  [??] {}", skip);
            return Ok(());
        }
    };
    for tool in &expected {
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
    // INT-192: could-not-read must not render as full coverage.
    let expected = match expected_tools() {
        Ok(e) => e,
        Err(skip) => {
            println!("  [??] Alias Coverage: {}", skip);
            return Ok(());
        }
    };
    for tool in &expected {
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
            expected.len(),
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

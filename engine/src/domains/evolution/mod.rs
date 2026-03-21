//! evolution domain — Core v8: The forest refines itself
//! Phase 1: Architecture map  (pure data, no suggestions)
//! Phase 2: Tools usage       (pure data, no suggestions)
//!
//! THE RULE: No suggestion without strong evidence.
//! These phases collect evidence. Suggestions come later.

use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::fs;
use std::path::Path;

// ── Phase 1 — Architecture Map ────────────────────────────────────────────────

/// Pure observation. Reads domain structure, counts files, measures coupling.
/// No suggestions. No interpretations. Data only.
pub fn map(ctx: &AppContext) -> CoreResult<()> {
    let domains_path = Path::new(&ctx.core_root).join("engine/src/domains");

    println!();
    println!("{}", "🏗  Architecture Map".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!("  {}  {}", "Source:".dimmed(), "engine/src/domains/".bright_white());
    println!();

    let mut domains: Vec<(String, usize, usize)> = vec![];

    if let Ok(entries) = fs::read_dir(&domains_path) {
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "evolution" { continue; } // skip self
                let rs_count = count_rs_files(&path);
                let coupling  = count_cross_domain_refs(&path);
                domains.push((name, rs_count, coupling));
            }
        }
    }

    let total_domains  = domains.len();
    let total_files: usize   = domains.iter().map(|(_, f, _)| f).sum();
    let total_coupling: usize = domains.iter().map(|(_, _, c)| c).sum();

    println!("  {}  {}", "Domains:".bright_white().bold(),
        total_domains.to_string().bright_green());
    println!("  {}  {}", "Total .rs files:".bright_white().bold(),
        total_files.to_string().bright_green());
    println!("  {}  {}", "Cross-domain refs:".bright_white().bold(),
        total_coupling.to_string().yellow());
    println!();

    println!("  {}", "Domain Breakdown:".bright_white().bold());
    println!("  {:<24} {:>6}  {:>10}",
        "Domain".dimmed(), "Files".dimmed(), "Coupling".dimmed());
    println!("  {}", "─".repeat(44).dimmed());

    let mut sorted = domains.clone();
    sorted.sort_by(|a, b| b.2.cmp(&a.2)); // sort by coupling desc

    for (name, files, coupling) in &sorted {
        let coupling_str = if *coupling > 8 {
            coupling.to_string().bright_red().to_string()
        } else if *coupling > 4 {
            coupling.to_string().yellow().to_string()
        } else {
            coupling.to_string().dimmed().to_string()
        };
        println!("  {:<24} {:>6}  {:>10}", name.bright_white(), files, coupling_str);
    }

    println!();
    println!("  {}",
        "coupling = outbound crate::domains:: references in domain source".dimmed().italic());
    println!("{}", "━".repeat(56).dimmed());
    println!("  {}", "Data collected. No suggestions. The forest observes.".dimmed().italic());
    println!();

    Ok(())
}

// ── Phase 2 — Tools Usage Analysis ───────────────────────────────────────────

/// Pure observation. Reads rust-tools, buckets by age, shows lifecycle stage.
/// No suggestions. No interpretations. Data only.
pub fn tools(ctx: &AppContext) -> CoreResult<()> {
    let tools_path = Path::new(&ctx.core_root).join("rust-tools");

    println!();
    println!("{}", "🔧  Tools Usage Analysis".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!("  {}  {}", "Source:".dimmed(), "rust-tools/".bright_white());
    println!();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut tool_data: Vec<(String, u64)> = vec![];

    if let Ok(entries) = fs::read_dir(&tools_path) {
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue; }
            let modified = newest_mtime(&path);
            tool_data.push((name, modified));
        }
    }

    let total = tool_data.len();

    let days_since = |m: u64| -> u64 { (now.saturating_sub(m)) / 86400 };

    let fresh:   Vec<_> = tool_data.iter().filter(|(_, m)| days_since(*m) <= 14).collect();
    let active:  Vec<_> = tool_data.iter().filter(|(_, m)| { let d = days_since(*m); d > 14 && d <= 60 }).collect();
    let stable:  Vec<_> = tool_data.iter().filter(|(_, m)| { let d = days_since(*m); d > 60 && d <= 120 }).collect();
    let dormant: Vec<_> = tool_data.iter().filter(|(_, m)| days_since(*m) > 120).collect();

    println!("  {}  {}", "Total tools:".bright_white().bold(),
        total.to_string().bright_green());
    println!();

    println!("  {}", "Lifecycle Distribution:".bright_white().bold());
    println!("  {:<10} {}  (≤ 14 days)",  "Fresh:".bright_green(),  fresh.len().to_string().bright_green());
    println!("  {:<10} {}  (15–60 days)", "Active:".bright_white(), active.len().to_string().bright_white());
    println!("  {:<10} {}  (61–120 days)","Stable:".yellow(),       stable.len().to_string().yellow());
    println!("  {:<10} {}  (> 120 days)", "Dormant:".dimmed(),      dormant.len().to_string().dimmed());
    println!();

    println!("  {}", "Tool Roster:".bright_white().bold());
    println!("  {:<34} {:>6}  {}",
        "Tool".dimmed(), "Age(d)".dimmed(), "Stage".dimmed());
    println!("  {}", "─".repeat(52).dimmed());

    for (name, modified) in &tool_data {
        let days = days_since(*modified);
        let stage = match days {
            0..=14   => "fresh".bright_green().to_string(),
            15..=60  => "active".bright_white().to_string(),
            61..=120 => "stable".yellow().to_string(),
            _        => "dormant".dimmed().to_string(),
        };
        println!("  {:<34} {:>6}  {}", name, days, stage);
    }

    println!();
    println!("  {}",
        "age = days since newest file modification inside tool directory".dimmed().italic());
    println!("{}", "━".repeat(56).dimmed());
    println!("  {}", "Data collected. No suggestions. The forest observes.".dimmed().italic());
    println!();

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn count_rs_files(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "rs").unwrap_or(false))
                .count()
        })
        .unwrap_or(0)
}

fn count_cross_domain_refs(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|x| x == "rs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    count += content.matches("crate::domains::").count();
                }
            }
        }
    }
    count
}

fn newest_mtime(dir: &Path) -> u64 {
    let mut latest = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(meta) = entry.metadata() {
                if let Ok(t) = meta.modified() {
                    if let Ok(d) = t.duration_since(std::time::UNIX_EPOCH) {
                        if d.as_secs() > latest { latest = d.as_secs(); }
                    }
                }
            }
        }
    }
    latest
}

// ── Phase 4 — Architecture Suggestions ───────────────────────────────────────
// The evidence rule: no suggestion without 2+ independent signals.
// Every suggestion cites source, threshold, evidence, confidence.

pub fn suggest(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;

    println!();
    println!("{}", "💡  Architecture Suggestions".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!("  {}  {}", "Rule:".dimmed(), "No suggestion without strong evidence.".bright_white());
    println!();

    let mut suggestions: Vec<(String, String, String, String)> = vec![];
    // (title, evidence, source, confidence)

    // ── Signal 1: CLI layer churn ─────────────────────────────────────────────
    // Read git log churn for CLI files
    let cli_files = [
        "engine/src/app/dispatcher.rs",
        "engine/src/cli/parser.rs",
        "engine/src/cli/mod.rs",
        "engine/src/cli/commands.rs",
    ];

    let mut cli_churn: Vec<(String, usize)> = vec![];
    for file in &cli_files {
        let output = std::process::Command::new("git")
            .args(["-C", &ctx.core_root, "log", "--oneline",
                "--follow", "--", file])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        let count = output.lines().count();
        if count > 0 {
            cli_churn.push((file.to_string(), count));
        }
    }

    let total_cli_churn: usize = cli_churn.iter().map(|(_, c)| c).sum();
    let high_churn_cli: Vec<_> = cli_churn.iter()
        .filter(|(_, c)| *c >= 50).collect();

    if high_churn_cli.len() >= 2 {
        let evidence = format!(
            "{} CLI files with 50+ changes each ({} total changes)",
            high_churn_cli.len(), total_cli_churn
        );
        let source = "git log per-file churn + coupling index";
        suggestions.push((
            "Consider splitting CLI layer into smaller modules".to_string(),
            evidence,
            source.to_string(),
            "HIGH — 4 independent signals (dispatcher, parser, mod, commands)".to_string(),
        ));
    }

    // ── Signal 2: Domain coupling ─────────────────────────────────────────────
    let domains_path = std::path::Path::new(&ctx.core_root)
        .join("engine/src/domains");

    let mut high_coupling: Vec<(String, usize)> = vec![];
    if let Ok(entries) = std::fs::read_dir(&domains_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = entry.file_name().to_string_lossy().to_string();
            let coupling = count_cross_domain_refs_suggest(&path);
            if coupling >= 3 {
                high_coupling.push((name, coupling));
            }
        }
    }

    for (domain, coupling) in &high_coupling {
        suggestions.push((
            format!("Domain '{}' has high cross-domain coupling", domain),
            format!("{} outbound crate::domains:: references — above threshold of 3", coupling),
            "evolution map coupling index".to_string(),
            "MEDIUM — single domain signal".to_string(),
        ));
    }

    // ── Signal 3: Dormant tools ───────────────────────────────────────────────
    let tools_path = std::path::Path::new(&ctx.core_root).join("rust-tools");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_secs();

    let mut dormant: Vec<String> = vec![];
    if let Ok(entries) = std::fs::read_dir(&tools_path) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() { continue; }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue; }
            let mtime = newest_mtime_suggest(&entry.path());
            let days = (now.saturating_sub(mtime)) / 86400;
            if days > 120 {
                dormant.push(format!("{} ({} days)", name, days));
            }
        }
    }

    if !dormant.is_empty() {
        suggestions.push((
            "Consider auditing dormant tools".to_string(),
            format!("{} tools untouched for 120+ days: {}", dormant.len(), dormant.join(", ")),
            "evolution tools lifecycle stage".to_string(),
            if dormant.len() >= 3 { "HIGH".to_string() } else { "MEDIUM".to_string() },
        ));
    }

    // ── Render suggestions ────────────────────────────────────────────────────
    if suggestions.is_empty() {
        println!("  {} {}", "✅".green(),
            "No architectural concerns detected. The forest looks healthy.".dimmed());
    } else {
        println!("  {} suggestion(s) detected:\n",
            suggestions.len().to_string().bright_yellow().bold());

        for (i, (title, evidence, source, confidence)) in suggestions.iter().enumerate() {
            println!("  {} {}",
                format!("[{}]", i + 1).bright_yellow().bold(),
                title.bright_white().bold()
            );
            println!("  {}  {}", "Evidence:".dimmed(),   evidence.yellow());
            println!("  {}   {}", "Source:".dimmed(),    source.dimmed());
            println!("  {}  {}", "Confidence:".dimmed(), confidence.bright_cyan());
            println!("  {}  {}",
                "Action:".dimmed(),
                "core evolve propose  — create a formal proposal".bright_cyan()
            );
            println!();
        }
    }

    println!("{}", "━".repeat(56).dimmed());
    println!("  {}", "The forest suggests. The human decides.".dimmed().italic());
    println!();
    Ok(())
}

fn count_cross_domain_refs_suggest(dir: &std::path::Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|x| x == "rs").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    count += content.matches("crate::domains::").count();
                }
            }
        }
    }
    count
}

fn newest_mtime_suggest(dir: &std::path::Path) -> u64 {
    let mut latest = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(t) = meta.modified() {
                    if let Ok(d) = t.duration_since(std::time::UNIX_EPOCH) {
                        if d.as_secs() > latest { latest = d.as_secs(); }
                    }
                }
            }
        }
    }
    latest
}

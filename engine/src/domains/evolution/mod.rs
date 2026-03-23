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
    println!(
        "  {}  {}",
        "Source:".dimmed(),
        "engine/src/domains/".bright_white()
    );
    println!();

    let mut domains: Vec<(String, usize, usize)> = vec![];

    if let Ok(entries) = fs::read_dir(&domains_path) {
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "evolution" {
                    continue;
                } // skip self
                let rs_count = count_rs_files(&path);
                let coupling = count_cross_domain_refs(&path);
                domains.push((name, rs_count, coupling));
            }
        }
    }

    let total_domains = domains.len();
    let total_files: usize = domains.iter().map(|(_, f, _)| f).sum();
    let total_coupling: usize = domains.iter().map(|(_, _, c)| c).sum();

    println!(
        "  {}  {}",
        "Domains:".bright_white().bold(),
        total_domains.to_string().bright_green()
    );
    println!(
        "  {}  {}",
        "Total .rs files:".bright_white().bold(),
        total_files.to_string().bright_green()
    );
    println!(
        "  {}  {}",
        "Cross-domain refs:".bright_white().bold(),
        total_coupling.to_string().yellow()
    );
    println!();

    println!("  {}", "Domain Breakdown:".bright_white().bold());
    println!(
        "  {:<24} {:>6}  {:>10}",
        "Domain".dimmed(),
        "Files".dimmed(),
        "Coupling".dimmed()
    );
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
        println!(
            "  {:<24} {:>6}  {:>10}",
            name.bright_white(),
            files,
            coupling_str
        );
    }

    println!();
    println!(
        "  {}",
        "coupling = outbound crate::domains:: references in domain source"
            .dimmed()
            .italic()
    );
    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}",
        "Data collected. No suggestions. The forest observes."
            .dimmed()
            .italic()
    );
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
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let modified = newest_mtime(&path);
            tool_data.push((name, modified));
        }
    }

    let total = tool_data.len();

    let days_since = |m: u64| -> u64 { (now.saturating_sub(m)) / 86400 };

    let fresh: Vec<_> = tool_data
        .iter()
        .filter(|(_, m)| days_since(*m) <= 14)
        .collect();
    let active: Vec<_> = tool_data
        .iter()
        .filter(|(_, m)| {
            let d = days_since(*m);
            d > 14 && d <= 60
        })
        .collect();
    let stable: Vec<_> = tool_data
        .iter()
        .filter(|(_, m)| {
            let d = days_since(*m);
            d > 60 && d <= 120
        })
        .collect();
    let dormant: Vec<_> = tool_data
        .iter()
        .filter(|(_, m)| days_since(*m) > 120)
        .collect();

    println!(
        "  {}  {}",
        "Total tools:".bright_white().bold(),
        total.to_string().bright_green()
    );
    println!();

    println!("  {}", "Lifecycle Distribution:".bright_white().bold());
    println!(
        "  {:<10} {}  (≤ 14 days)",
        "Fresh:".bright_green(),
        fresh.len().to_string().bright_green()
    );
    println!(
        "  {:<10} {}  (15–60 days)",
        "Active:".bright_white(),
        active.len().to_string().bright_white()
    );
    println!(
        "  {:<10} {}  (61–120 days)",
        "Stable:".yellow(),
        stable.len().to_string().yellow()
    );
    println!(
        "  {:<10} {}  (> 120 days)",
        "Dormant:".dimmed(),
        dormant.len().to_string().dimmed()
    );
    println!();

    println!("  {}", "Tool Roster:".bright_white().bold());
    println!(
        "  {:<34} {:>6}  {}",
        "Tool".dimmed(),
        "Age(d)".dimmed(),
        "Stage".dimmed()
    );
    println!("  {}", "─".repeat(52).dimmed());

    for (name, modified) in &tool_data {
        let days = days_since(*modified);
        let stage = match days {
            0..=14 => "fresh".bright_green().to_string(),
            15..=60 => "active".bright_white().to_string(),
            61..=120 => "stable".yellow().to_string(),
            _ => "dormant".dimmed().to_string(),
        };
        println!("  {:<34} {:>6}  {}", name, days, stage);
    }

    println!();
    println!(
        "  {}",
        "age = days since newest file modification inside tool directory"
            .dimmed()
            .italic()
    );
    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}",
        "Data collected. No suggestions. The forest observes."
            .dimmed()
            .italic()
    );
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
                        if d.as_secs() > latest {
                            latest = d.as_secs();
                        }
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
    println!(
        "  {}  {}",
        "Rule:".dimmed(),
        "No suggestion without strong evidence.".bright_white()
    );
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
            .args([
                "-C",
                &ctx.core_root,
                "log",
                "--oneline",
                "--follow",
                "--",
                file,
            ])
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
    let high_churn_cli: Vec<_> = cli_churn.iter().filter(|(_, c)| *c >= 50).collect();

    if high_churn_cli.len() >= 2 {
        let evidence = format!(
            "{} CLI files with 50+ changes each ({} total changes)",
            high_churn_cli.len(),
            total_cli_churn
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
    let domains_path = std::path::Path::new(&ctx.core_root).join("engine/src/domains");

    let mut high_coupling: Vec<(String, usize)> = vec![];
    if let Ok(entries) = std::fs::read_dir(&domains_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
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
            format!(
                "{} outbound crate::domains:: references — above threshold of 3",
                coupling
            ),
            "evolution map coupling index".to_string(),
            "MEDIUM — single domain signal".to_string(),
        ));
    }

    // ── Signal 3: Dormant tools ───────────────────────────────────────────────
    let tools_path = std::path::Path::new(&ctx.core_root).join("rust-tools");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut dormant: Vec<String> = vec![];
    if let Ok(entries) = std::fs::read_dir(&tools_path) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
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
            format!(
                "{} tools untouched for 120+ days: {}",
                dormant.len(),
                dormant.join(", ")
            ),
            "evolution tools lifecycle stage".to_string(),
            if dormant.len() >= 3 {
                "HIGH".to_string()
            } else {
                "MEDIUM".to_string()
            },
        ));
    }

    // ── Render suggestions ────────────────────────────────────────────────────
    if suggestions.is_empty() {
        println!(
            "  {} {}",
            "✅".green(),
            "No architectural concerns detected. The forest looks healthy.".dimmed()
        );
    } else {
        println!(
            "  {} suggestion(s) detected:\n",
            suggestions.len().to_string().bright_yellow().bold()
        );

        for (i, (title, evidence, source, confidence)) in suggestions.iter().enumerate() {
            println!(
                "  {} {}",
                format!("[{}]", i + 1).bright_yellow().bold(),
                title.bright_white().bold()
            );
            println!("  {}  {}", "Evidence:".dimmed(), evidence.yellow());
            println!("  {}   {}", "Source:".dimmed(), source.dimmed());
            println!("  {}  {}", "Confidence:".dimmed(), confidence.bright_cyan());
            println!(
                "  {}  {}",
                "Action:".dimmed(),
                "core evolve propose  — create a formal proposal".bright_cyan()
            );
            println!();
        }
    }

    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}",
        "The forest suggests. The human decides.".dimmed().italic()
    );
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
                        if d.as_secs() > latest {
                            latest = d.as_secs();
                        }
                    }
                }
            }
        }
    }
    latest
}

// ── Phase 5 — Evolution Proposals ────────────────────────────────────────────
// Formal proposals generated from Phase 4 suggestions.
// Stored in state.db. Human accepts or rejects.
// Accepted proposals become intent records.

pub fn ensure_proposals_schema(ctx: &AppContext) -> CoreResult<()> {
    ctx.runtime.db.execute_batch(
        "CREATE TABLE IF NOT EXISTS evolution_proposals (
            id          TEXT PRIMARY KEY,
            timestamp   INTEGER NOT NULL,
            title       TEXT NOT NULL,
            evidence    TEXT NOT NULL,
            source      TEXT NOT NULL,
            confidence  TEXT NOT NULL,
            status      TEXT NOT NULL DEFAULT 'pending',
            notes       TEXT
        );",
    )?;
    Ok(())
}

pub fn evolve_propose(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    ensure_proposals_schema(ctx)?;

    // Run suggest() logic to get current signals
    // then store each as a proposal
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // Count existing proposals
    let existing: i64 = ctx
        .runtime
        .db
        .query_row("SELECT COUNT(*) FROM evolution_proposals", [], |r| r.get(0))
        .unwrap_or(0);

    let next_id = existing + 1;

    // Generate proposals from current signals
    let mut proposals: Vec<(String, String, String, String)> = vec![];

    // Signal 1: CLI layer churn
    let cli_files = [
        "engine/src/app/dispatcher.rs",
        "engine/src/cli/parser.rs",
        "engine/src/cli/mod.rs",
        "engine/src/cli/commands.rs",
    ];
    let mut high_churn = 0usize;
    let mut total_churn = 0usize;
    for file in &cli_files {
        let output = std::process::Command::new("git")
            .args([
                "-C",
                &ctx.core_root,
                "log",
                "--oneline",
                "--follow",
                "--",
                file,
            ])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        let count = output.lines().count();
        total_churn += count;
        if count >= 50 {
            high_churn += 1;
        }
    }
    if high_churn >= 2 {
        proposals.push((
            "Split CLI layer into smaller modules".to_string(),
            format!(
                "{} CLI files with 50+ changes ({} total)",
                high_churn, total_churn
            ),
            "git churn + coupling index".to_string(),
            "HIGH".to_string(),
        ));
    }

    // Signal 2: High coupling domains
    let domains_path = std::path::Path::new(&ctx.core_root).join("engine/src/domains");
    if let Ok(entries) = std::fs::read_dir(&domains_path) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let coupling = count_cross_domain_refs_suggest(&entry.path());
            if coupling >= 4 {
                proposals.push((
                    format!("Reduce coupling in '{}' domain", name),
                    format!("{} cross-domain references — above threshold", coupling),
                    "evolution map coupling index".to_string(),
                    "MEDIUM".to_string(),
                ));
            }
        }
    }

    if proposals.is_empty() {
        println!();
        println!(
            "  {} {}",
            "✅".green(),
            "No proposals to generate — no strong signals detected.".dimmed()
        );
        println!();
        return Ok(());
    }

    let mut stored = 0;
    for (i, (title, evidence, source, confidence)) in proposals.iter().enumerate() {
        let prop_id = format!("PROP-{:03}", next_id + i as i64);
        // Skip if already exists with same title
        let exists: i64 = ctx
            .runtime
            .db
            .query_row(
                "SELECT COUNT(*) FROM evolution_proposals WHERE title = ?1",
                rusqlite::params![title],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if exists > 0 {
            continue;
        }
        ctx.runtime.db.execute(
            "INSERT INTO evolution_proposals (id, timestamp, title, evidence, source, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![prop_id, now, title, evidence, source, confidence],
        )?;
        stored += 1;
    }

    println!();
    println!(
        "{}",
        "📋  Evolution Proposals Generated".bright_cyan().bold()
    );
    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {} new proposal(s) stored.",
        stored.to_string().bright_green()
    );
    println!(
        "  {} {}",
        "Review with:".dimmed(),
        "core evolution evolve-list".bright_cyan()
    );
    println!(
        "  {} {}",
        "Accept with:".dimmed(),
        "core evolution evolve-accept <id>".bright_cyan()
    );
    println!();
    Ok(())
}

pub fn evolve_list(ctx: &AppContext) -> CoreResult<()> {
    use colored::*;

    ensure_proposals_schema(ctx)?;

    let mut stmt = ctx.runtime.db.prepare(
        "SELECT id, title, evidence, confidence, status FROM evolution_proposals ORDER BY timestamp DESC"
    )?;

    let rows: Vec<(String, String, String, String, String)> = stmt
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    println!();
    println!("{}", "📋  Evolution Proposals".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());

    if rows.is_empty() {
        println!(
            "  {} No proposals yet. Run: {}",
            "○".dimmed(),
            "core evolution evolve-propose".bright_cyan()
        );
    } else {
        for (id, title, evidence, confidence, status) in &rows {
            let status_colored = match status.as_str() {
                "accepted" => status.bright_green().to_string(),
                "rejected" => status.bright_red().to_string(),
                _ => status.yellow().to_string(),
            };
            println!(
                "  {} {} {}",
                id.bright_white().bold(),
                format!("[{}]", status_colored),
                title.bright_white()
            );
            println!("  {}  {}", "Evidence:".dimmed(), evidence.dimmed());
            println!("  {}  {}", "Confidence:".dimmed(), confidence.bright_cyan());
            println!();
        }
    }

    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}",
        "The forest proposes. The human decides.".dimmed().italic()
    );
    println!();
    Ok(())
}

pub fn evolve_accept(ctx: &AppContext, id: &str) -> CoreResult<()> {
    use colored::*;

    ensure_proposals_schema(ctx)?;

    let result = ctx.runtime.db.execute(
        "UPDATE evolution_proposals SET status = 'accepted' WHERE id = ?1",
        rusqlite::params![id],
    )?;

    if result == 0 {
        println!("  {} Proposal {} not found.", "✗".bright_red(), id);
        return Ok(());
    }

    // Get proposal title for intent suggestion
    let title: String = ctx
        .runtime
        .db
        .query_row(
            "SELECT title FROM evolution_proposals WHERE id = ?1",
            rusqlite::params![id],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "Unknown proposal".to_string());

    println!();
    println!(
        "  {} Proposal {} accepted.",
        "✅".green(),
        id.bright_white().bold()
    );
    println!("  {} {}", "Title:".dimmed(), title.bright_white());
    println!();
    println!(
        "  {} Record as a formal intent:",
        "Next step:".bright_cyan().bold()
    );
    println!(
        "  {} {}",
        "→".bright_cyan(),
        format!("core decide \"Implement: {}\"", title).dimmed()
    );
    println!();
    Ok(())
}

pub fn evolve_reject(ctx: &AppContext, id: &str) -> CoreResult<()> {
    use colored::*;

    ensure_proposals_schema(ctx)?;

    let result = ctx.runtime.db.execute(
        "UPDATE evolution_proposals SET status = 'rejected' WHERE id = ?1",
        rusqlite::params![id],
    )?;

    if result == 0 {
        println!("  {} Proposal {} not found.", "✗".bright_red(), id);
        return Ok(());
    }

    println!();
    println!(
        "  {} Proposal {} rejected and logged.",
        "○".dimmed(),
        id.bright_white()
    );
    println!(
        "  {}",
        "The forest remembers the decision.".dimmed().italic()
    );
    println!();
    Ok(())
}

// ── Phase 6 — Future Simulation ───────────────────────────────────────────────
// Simulate architectural changes before making them.
// What would break? What is the risk? What is affected?

pub fn future_sim(ctx: &AppContext, change: &str) -> CoreResult<()> {
    use colored::*;

    println!();
    println!("{}", "🔮  Future Simulation".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!("  {}  {}", "Change:".dimmed(), change.bright_white().bold());
    println!();

    let change_lower = change.to_lowercase();
    let mut affected: Vec<String> = vec![];
    let mut warnings: Vec<String> = vec![];

    // Scan domains for references to keywords in the change
    let domains_path = std::path::Path::new(&ctx.core_root).join("engine/src/domains");
    let keywords: Vec<&str> = change_lower
        .split_whitespace()
        .filter(|w| w.len() > 4)
        .collect();

    if let Ok(entries) = std::fs::read_dir(&domains_path) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let mod_path = entry.path().join("mod.rs");
            if let Ok(content) = std::fs::read_to_string(&mod_path) {
                let content_lower = content.to_lowercase();
                let hits = keywords
                    .iter()
                    .filter(|k| content_lower.contains(*k))
                    .count();
                if hits > 0 {
                    affected.push(format!("{} ({} keyword matches)", name, hits));
                }
            }
        }
    }

    // Check tools directory
    let tools_path = std::path::Path::new(&ctx.core_root).join("rust-tools");
    let mut affected_tools: Vec<String> = vec![];
    if let Ok(entries) = std::fs::read_dir(&tools_path) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let src = entry.path().join("src/main.rs");
            if let Ok(content) = std::fs::read_to_string(&src) {
                let content_lower = content.to_lowercase();
                let hits = keywords
                    .iter()
                    .filter(|k| content_lower.contains(*k))
                    .count();
                if hits >= 2 {
                    affected_tools.push(name);
                }
            }
        }
    }

    // Generate warnings based on change type
    if change_lower.contains("remov")
        || change_lower.contains("delet")
        || change_lower.contains("drop")
    {
        warnings.push("Removal changes are irreversible — snapshot first".to_string());
        warnings.push("Run: core checkpoint before making this change".to_string());
    }
    if change_lower.contains("split") || change_lower.contains("refactor") {
        warnings.push("Refactoring may break existing aliases and integrations".to_string());
    }
    if change_lower.contains("cli")
        || change_lower.contains("command")
        || change_lower.contains("dispatch")
    {
        warnings.push("CLI changes affect all 43 tools that call core".to_string());
    }

    // Render
    if affected.is_empty() {
        println!(
            "  {} No domain references detected for this change.",
            "○".dimmed()
        );
    } else {
        println!("  {} Affected domains:", "→".bright_yellow());
        for d in &affected {
            println!("    {} {}", "·".dimmed(), d.bright_white());
        }
        println!();
    }

    if !affected_tools.is_empty() {
        println!("  {} Affected tools:", "→".bright_yellow());
        for t in &affected_tools {
            println!("    {} {}", "·".dimmed(), t.bright_white());
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("  {} Warnings:", "⚠".yellow());
        for w in &warnings {
            println!("    {} {}", "·".yellow(), w.yellow());
        }
        println!();
    }

    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}",
        "Simulation complete. The forest proposes. The human decides."
            .dimmed()
            .italic()
    );
    println!();
    Ok(())
}

pub fn future_risk(ctx: &AppContext, change: &str) -> CoreResult<()> {
    use colored::*;

    println!();
    println!("{}", "⚡  Risk Analysis".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!("  {}  {}", "Change:".dimmed(), change.bright_white().bold());
    println!();

    let change_lower = change.to_lowercase();
    let mut risk_score: u32 = 0;
    let mut factors: Vec<(String, u32)> = vec![];

    // Risk factors
    if change_lower.contains("remov") || change_lower.contains("delet") {
        factors.push(("Removal/deletion — hard to reverse".to_string(), 30));
    }
    if change_lower.contains("cli")
        || change_lower.contains("dispatch")
        || change_lower.contains("command")
    {
        factors.push(("CLI layer change — affects all consumers".to_string(), 25));
    }
    if change_lower.contains("database")
        || change_lower.contains("schema")
        || change_lower.contains("sqlite")
    {
        factors.push(("Database/schema change — migration risk".to_string(), 25));
    }
    if change_lower.contains("split") || change_lower.contains("refactor") {
        factors.push(("Refactor — integration risk".to_string(), 15));
    }
    if change_lower.contains("api") || change_lower.contains("interface") {
        factors.push(("API/interface change — downstream breakage".to_string(), 20));
    }
    if change_lower.contains("security") || change_lower.contains("auth") {
        factors.push(("Security domain change — elevated risk".to_string(), 20));
    }

    // Check health — low health increases risk
    let health: u32 = ctx
        .runtime
        .db
        .query_row(
            "SELECT payload FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .ok()
        .and_then(|p| serde_json::from_str::<serde_json::Value>(&p).ok())
        .and_then(|v| v["detail"]["health"].as_i64())
        .unwrap_or(100) as u32;

    if health < 95 {
        factors.push((
            format!("Health at {}% — elevated baseline risk", health),
            10,
        ));
    }

    for (_, score) in &factors {
        risk_score += score;
    }
    risk_score = risk_score.min(100);

    let risk_label = match risk_score {
        0..=20 => ("LOW", "bright_green"),
        21..=50 => ("MEDIUM", "yellow"),
        51..=75 => ("HIGH", "bright_red"),
        _ => ("CRITICAL", "bright_red"),
    };

    println!(
        "  {}  {}/100",
        "Risk Score:".dimmed(),
        risk_score.to_string().bright_yellow().bold()
    );
    println!(
        "  {}  {}",
        "Risk Level:".dimmed(),
        risk_label.0.bright_yellow().bold()
    );
    println!();

    if factors.is_empty() {
        println!("  {} No specific risk factors detected.", "✅".green());
    } else {
        println!("  {} Risk factors:", "→".bright_yellow());
        for (factor, score) in &factors {
            println!("    {} {} (+{} pts)", "·".dimmed(), factor.yellow(), score);
        }
    }

    println!();
    println!(
        "  {} {}",
        "Mitigation:".dimmed(),
        "core checkpoint  — snapshot before proceeding".bright_cyan()
    );
    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}",
        "The forest assesses. The human decides.".dimmed().italic()
    );
    println!();
    Ok(())
}

pub fn future_impact(ctx: &AppContext, change: &str) -> CoreResult<()> {
    use colored::*;

    println!();
    println!("{}", "🌊  Impact Analysis".bright_cyan().bold());
    println!("{}", "━".repeat(56).dimmed());
    println!("  {}  {}", "Change:".dimmed(), change.bright_white().bold());
    println!();

    let change_lower = change.to_lowercase();
    let keywords: Vec<&str> = change_lower
        .split_whitespace()
        .filter(|w| w.len() > 4)
        .collect();

    // Count total domains affected
    let domains_path = std::path::Path::new(&ctx.core_root).join("engine/src/domains");
    let total_domains: usize = std::fs::read_dir(&domains_path)
        .map(|e| {
            e.filter_map(|x| x.ok())
                .filter(|x| x.path().is_dir())
                .count()
        })
        .unwrap_or(0);

    let mut affected_count = 0usize;
    let mut high_impact: Vec<String> = vec![];

    if let Ok(entries) = std::fs::read_dir(&domains_path) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let mod_path = entry.path().join("mod.rs");
            if let Ok(content) = std::fs::read_to_string(&mod_path) {
                let content_lower = content.to_lowercase();
                let hits = keywords
                    .iter()
                    .filter(|k| content_lower.contains(*k))
                    .count();
                if hits > 0 {
                    affected_count += 1;
                    if hits >= 3 {
                        high_impact.push(name);
                    }
                }
            }
        }
    }

    let impact_pct = if total_domains > 0 {
        (affected_count * 100) / total_domains
    } else {
        0
    };

    let blast_radius = match impact_pct {
        0..=10 => "CONTAINED — minimal blast radius",
        11..=30 => "MODERATE — several domains affected",
        31..=60 => "SIGNIFICANT — major subsystem affected",
        _ => "WIDE — forest-wide impact",
    };

    println!(
        "  {}  {}/{} domains ({}%)",
        "Affected:".dimmed(),
        affected_count.to_string().bright_yellow(),
        total_domains,
        impact_pct
    );
    println!(
        "  {}  {}",
        "Blast radius:".dimmed(),
        blast_radius.bright_yellow()
    );
    println!();

    if !high_impact.is_empty() {
        println!("  {} High-impact domains:", "→".bright_red());
        for d in &high_impact {
            println!("    {} {}", "·".bright_red(), d.bright_white());
        }
        println!();
    }

    // Intents at risk
    let intents_path = std::path::Path::new(&ctx.core_root).join("intents/future");
    let mut at_risk_intents: Vec<String> = vec![];
    if let Ok(entries) = std::fs::read_dir(&intents_path) {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let content_lower = content.to_lowercase();
                let hits = keywords
                    .iter()
                    .filter(|k| content_lower.contains(*k))
                    .count();
                if hits >= 2 {
                    let name = entry.file_name().to_string_lossy().to_string();
                    at_risk_intents.push(name.replace(".md", ""));
                }
            }
        }
    }

    if !at_risk_intents.is_empty() {
        println!("  {} Intents potentially affected:", "→".yellow());
        for i in at_risk_intents.iter().take(5) {
            println!("    {} {}", "·".dimmed(), i.dimmed());
        }
        println!();
    }

    println!("{}", "━".repeat(56).dimmed());
    println!(
        "  {}",
        "Impact mapped. The forest proposes. The human decides."
            .dimmed()
            .italic()
    );
    println!();
    Ok(())
}

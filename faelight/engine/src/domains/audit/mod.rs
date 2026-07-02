// core audit — Tool Intelligence Layer
// INT-123 Phase 1: scan, show, stale, coverage
//
// "The forest notices. You decide."

use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

// ── Tool Score ────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct ToolScore {
    name: String,
    score: u32,
    usage_score: u32,
    recency_score: u32,
    doc_score: u32,
    version_score: u32,
    last_event_days: Option<u64>,
    last_commit_days: Option<u64>,
    has_description: bool,
    has_readme: bool,
    issues: Vec<&'static str>,
}

impl ToolScore {
    fn health_label(&self) -> colored::ColoredString {
        if self.score >= 80 {
            "healthy".bright_green()
        } else if self.score >= 60 {
            "fair".yellow()
        } else {
            "needs attention".bright_red()
        }
    }
}

// ── Scoring Logic ─────────────────────────────────────────────────────────────

fn expected_usage(_core_root: &str, tool_name: &str) -> &'static str {
    let registry =
        std::fs::read_to_string(faelight_core::paths::tools_registry())
            .unwrap_or_default();

    // Find the tool section and read expected_usage
    let mut in_tool = false;
    for line in registry.lines() {
        if line.contains(tool_name) && line.starts_with("name") {
            in_tool = true;
        }
        if in_tool && line.starts_with("expected_usage") {
            if line.contains("high") {
                return "high";
            }
            if line.contains("medium") {
                return "medium";
            }
            if line.contains("rare") {
                return "rare";
            }
            return "low";
        }
        // Stop at next tool
        if in_tool && line.starts_with("[[tool]]") {
            break;
        }
    }
    "low"
}

fn score_tool(ctx: &AppContext, name: &str, core_root: &str) -> ToolScore {
    let tool_path = PathBuf::from(core_root).join("rust-tools").join(name);
    let mut issues = Vec::new();

    // ── Usage score (25%) — calibrated by expected_usage ─────────────────
    let usage = expected_usage(core_root, name);
    let usage_score = {
        let db = &ctx.runtime.db;
        let thirty_days_ago = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64 - 30 * 86400)
            .unwrap_or(0);

        let count: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM events WHERE payload LIKE ?1 AND timestamp > ?2",
                rusqlite::params![format!("%{}%", name), thirty_days_ago],
                |r| r.get(0),
            )
            .unwrap_or(0);

        match usage {
            "rare" => {
                // Rare tools not penalized for low event count
                if count == 0 {
                    20
                } else {
                    25
                }
            }
            "low" => match count {
                0 => {
                    issues.push("no events in 30 days");
                    10
                }
                _ => 25,
            },
            "medium" => match count {
                0 => {
                    issues.push("no events in 30 days");
                    5
                }
                1..=3 => 15,
                _ => 25,
            },
            _ => {
                // high
                match count {
                    0 => {
                        issues.push("no events in 30 days");
                        0
                    }
                    1..=5 => 15,
                    6..=20 => 20,
                    _ => 25,
                }
            }
        }
    };

    // ── Recency score (25%) — days since last git commit ──────────────────
    let (recency_score, last_commit_days) = {
        let output = Command::new("git")
            .args([
                "-C",
                core_root,
                "log",
                "--oneline",
                "-1",
                "--format=%ct",
                "--",
                &format!("rust-tools/{}/", name),
            ])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok());

        if let Some(ts_str) = output {
            let ts_str = ts_str.trim();
            if let Ok(ts) = ts_str.parse::<i64>() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let days = ((now - ts) / 86400) as u64;
                let score = match days {
                    0..=7 => 25,
                    8..=30 => 20,
                    31..=60 => 15,
                    61..=90 => 10,
                    _ => {
                        issues.push("not touched in 90+ days");
                        0
                    }
                };
                (score, Some(days))
            } else {
                issues.push("no git history");
                (0, None)
            }
        } else {
            (10, None)
        }
    };

    // ── Documentation score (25%) ─────────────────────────────────────────
    let has_readme = tool_path.join("README.md").exists();
    let cargo_toml = tool_path.join("Cargo.toml");
    let has_description = std::fs::read_to_string(&cargo_toml)
        .map(|t| t.contains("description = \"") && !t.contains("description = \"\""))
        .unwrap_or(false);

    let doc_score = match (has_readme, has_description) {
        (true, true) => 25,
        (true, false) => {
            issues.push("missing description in Cargo.toml");
            15
        }
        (false, true) => {
            issues.push("no README.md");
            15
        }
        (false, false) => {
            issues.push("no README, no description");
            5
        }
    };

    // ── Version currency score (25%) ──────────────────────────────────────
    let version_score = {
        let output = Command::new("git")
            .args([
                "-C",
                core_root,
                "log",
                "--oneline",
                "-1",
                "--format=%ct",
                "--",
                &format!("rust-tools/{}/Cargo.toml", name),
            ])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok());

        if let Some(ts_str) = output {
            let ts_str = ts_str.trim();
            if let Ok(ts) = ts_str.parse::<i64>() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let days = (now - ts) / 86400;
                if days > 90 {
                    issues.push("version not bumped in 90+ days");
                    10
                } else {
                    25
                }
            } else {
                15
            }
        } else {
            15
        }
    };

    let score = usage_score + recency_score + doc_score + version_score;

    ToolScore {
        name: name.to_string(),
        score,
        usage_score,
        recency_score,
        doc_score,
        version_score,
        last_event_days: None,
        last_commit_days,
        has_description,
        has_readme,
        issues,
    }
}

fn get_all_tools(core_root: &str) -> Vec<String> {
    let tools_dir = PathBuf::from(core_root).join("rust-tools");
    let mut tools = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&tools_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("archived") {
                continue;
            }
            if entry.path().join("Cargo.toml").exists() {
                tools.push(name);
            }
        }
    }
    tools.sort();
    tools
}

// ── Commands ──────────────────────────────────────────────────────────────────

pub fn scan(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("audit", &[Capability::FilesystemReadHome])?;

    let core_root = &ctx.core_root;
    let tools = get_all_tools(core_root);
    let total = tools.len();

    println!();
    println!(
        "{}",
        "  ╭─ 🔍 Tool Intelligence Report ──────────────────────────────".bright_cyan()
    );
    println!(
        "  │  Analyzing {} tools...",
        total.to_string().bright_white()
    );
    println!(
        "{}",
        "  ╰────────────────────────────────────────────────────────────".dimmed()
    );

    let scores: Vec<ToolScore> = tools
        .iter()
        .map(|t| score_tool(ctx, t, core_root))
        .collect();

    let needs_attention: Vec<&ToolScore> = scores.iter().filter(|s| s.score < 70).collect();

    let healthy_count = scores.iter().filter(|s| s.score >= 70).count();

    // Summary header
    println!();
    println!(
        "{}",
        "  ╭──────────────────────────────────────────────────────────╮".bright_cyan()
    );
    println!(
        "  │  {} tools analyzed  │  {} healthy  │  {} need attention  │",
        total.to_string().bright_white(),
        healthy_count.to_string().bright_green(),
        needs_attention.len().to_string().yellow(),
    );
    println!(
        "{}",
        "  ╰──────────────────────────────────────────────────────────╯".bright_cyan()
    );

    if !needs_attention.is_empty() {
        println!();
        println!(
            "{}",
            "  ╭─ ⚠️  Needs Attention ────────────────────────────────────".yellow()
        );
        for t in &needs_attention {
            let issue = t.issues.first().copied().unwrap_or("below threshold");
            println!(
                "  │  {:<30} {}  {}/100",
                t.name.bright_white(),
                issue.yellow(),
                t.score.to_string().yellow(),
            );
        }
        println!(
            "{}",
            "  ╰────────────────────────────────────────────────────────".dimmed()
        );
    }

    println!();
    println!(
        "{}",
        "  ╭─ 🟢 Healthy ────────────────────────────────────────────".bright_green()
    );
    let healthy: Vec<&ToolScore> = scores.iter().filter(|s| s.score >= 70).collect();
    let names: Vec<&str> = healthy.iter().map(|s| s.name.as_str()).collect();
    println!(
        "  │  {} tools above threshold",
        healthy_count.to_string().bright_green()
    );
    // Show them in compact form
    for chunk in names.chunks(4) {
        println!("  │  {}", chunk.join("  ").dimmed());
    }
    println!(
        "{}",
        "  ╰────────────────────────────────────────────────────────".dimmed()
    );
    println!();

    // Write to state.db
    write_audit_scores(ctx, &scores);

    Ok(())
}

pub fn show(ctx: &AppContext, tool_name: &str) -> CoreResult<()> {
    ctx.capabilities
        .require("audit", &[Capability::FilesystemReadHome])?;

    let core_root = &ctx.core_root;
    let score = score_tool(ctx, tool_name, core_root);

    println!();
    println!(
        "{}",
        "  ╭─ 🔍 Tool Audit ─────────────────────────────────────────".bright_cyan()
    );
    println!("  │  Tool:    {}", score.name.bright_white().bold());
    println!(
        "  │  Score:   {}/100  ({})",
        score.score,
        score.health_label()
    );
    println!(
        "{}",
        "  ├─────────────────────────────────────────────────────────".dimmed()
    );
    println!(
        "  │  Usage:    {}/25  {}",
        score.usage_score,
        if let Some(d) = score.last_event_days {
            format!("(last event {}d ago)", d).dimmed().to_string()
        } else {
            "".to_string()
        }
    );
    println!(
        "  │  Recency:  {}/25  {}",
        score.recency_score,
        if let Some(d) = score.last_commit_days {
            format!("(last commit {}d ago)", d).dimmed().to_string()
        } else {
            "".to_string()
        }
    );
    println!(
        "  │  Docs:     {}/25  README:{} Description:{}",
        score.doc_score,
        if score.has_readme { "✅" } else { "❌" },
        if score.has_description { "✅" } else { "❌" },
    );
    println!("  │  Version:  {}/25", score.version_score);

    if !score.issues.is_empty() {
        println!(
            "{}",
            "  ├─────────────────────────────────────────────────────────".dimmed()
        );
        println!("  │  {}", "Issues:".yellow().bold());
        for issue in &score.issues {
            println!("  │    {} {}", "→".yellow(), issue.yellow());
        }
    }
    println!(
        "{}",
        "  ╰─────────────────────────────────────────────────────────".dimmed()
    );
    println!();

    Ok(())
}

pub fn stale(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("audit", &[Capability::FilesystemReadHome])?;

    let core_root = &ctx.core_root;
    let tools = get_all_tools(core_root);
    let scores: Vec<ToolScore> = tools
        .iter()
        .map(|t| score_tool(ctx, t, core_root))
        .collect();

    let stale: Vec<&ToolScore> = scores.iter().filter(|s| s.score < 70).collect();

    println!();
    println!(
        "{}",
        "  ╭─ ⚠️  Stale Tools ────────────────────────────────────────".yellow()
    );
    if stale.is_empty() {
        println!("  │  {} All tools are healthy", "✅".green());
    } else {
        println!(
            "  │  {} tools need attention:",
            stale.len().to_string().yellow().bold()
        );
        println!(
            "{}",
            "  ├─────────────────────────────────────────────────────────".dimmed()
        );
        for t in &stale {
            println!(
                "  │  {:<28} score:{}/100",
                t.name.bright_white(),
                t.score.to_string().yellow(),
            );
            for issue in &t.issues {
                println!("  │    {} {}", "→".dimmed(), issue.yellow());
            }
        }
    }
    println!(
        "{}",
        "  ╰─────────────────────────────────────────────────────────".dimmed()
    );
    println!();

    Ok(())
}

pub fn coverage(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities
        .require("audit", &[Capability::FilesystemReadHome])?;

    let core_root = &ctx.core_root;
    let tools = get_all_tools(core_root);

    let mut no_readme = Vec::new();
    let mut no_description = Vec::new();

    for tool in &tools {
        let tool_path = PathBuf::from(core_root).join("rust-tools").join(tool);
        if !tool_path.join("README.md").exists() {
            no_readme.push(tool.clone());
        }
        let has_desc = std::fs::read_to_string(tool_path.join("Cargo.toml"))
            .map(|t| t.contains("description = \"") && !t.contains("description = \"\""))
            .unwrap_or(false);
        if !has_desc {
            no_description.push(tool.clone());
        }
    }

    println!();
    println!(
        "{}",
        "  ╭─ 📋 Documentation Coverage ─────────────────────────────".bright_cyan()
    );
    println!("  │  {} tools analyzed", tools.len());
    println!(
        "  │  {} missing README",
        no_readme.len().to_string().yellow()
    );
    println!(
        "  │  {} missing description",
        no_description.len().to_string().yellow()
    );

    if !no_readme.is_empty() {
        println!(
            "{}",
            "  ├─ No README ─────────────────────────────────────────────".dimmed()
        );
        for t in &no_readme {
            println!("  │  {} {}", "→".yellow(), t.bright_white());
        }
    }
    if !no_description.is_empty() {
        println!(
            "{}",
            "  ├─ No Description ────────────────────────────────────────".dimmed()
        );
        for t in &no_description {
            println!("  │  {} {}", "→".yellow(), t.bright_white());
        }
    }
    println!(
        "{}",
        "  ╰─────────────────────────────────────────────────────────".dimmed()
    );
    println!();

    Ok(())
}

fn write_audit_scores(ctx: &AppContext, scores: &[ToolScore]) {
    let db = &ctx.runtime.db;
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS audit_scores (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            tool_name        TEXT NOT NULL,
            score            INTEGER NOT NULL,
            usage_score      INTEGER,
            recency_score    INTEGER,
            doc_score        INTEGER,
            version_score    INTEGER,
            last_commit_days INTEGER,
            timestamp        INTEGER NOT NULL
        );",
    )
    .ok();

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    for s in scores {
        db.execute(
            "INSERT INTO audit_scores (tool_name, score, usage_score, recency_score, doc_score, version_score, last_commit_days, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                s.name, s.score, s.usage_score, s.recency_score,
                s.doc_score, s.version_score, s.last_commit_days, ts
            ],
        ).ok();
    }
}

/// INT-341: Deferral Ledger -- scan all intent files for ⏸ deferrals
pub fn deferral_list(ctx: &AppContext, _flag_old: bool) -> CoreResult<()> {
    use colored::Colorize;
    use std::fs;

    let core_root = &ctx.core_root;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let thirty_days_ago = now - (30 * 24 * 3600);

    let dirs = ["complete", "in-progress", "future"];
    let mut all_deferrals: Vec<(String, String, String, String, String, i64)> = Vec::new();
    // (intent_id, intent_title, gate, reason, date, date_ts)

    for dir in &dirs {
        let path = format!("{}/intents/{}", core_root, dir);
        let entries = match fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if !fname.ends_with(".md") { continue; }
            let intent_id = fname.split('-').next().unwrap_or("???").to_string();
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            // Get intent title from frontmatter
            let title = content.lines()
                .find(|l| l.starts_with("title:"))
                .map(|l| l.trim_start_matches("title:").trim().trim_matches('"').to_string())
                .unwrap_or_else(|| fname.clone());
            // Find ⏸ lines
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("⏸") || trimmed.starts_with("- [x] ⏸") || trimmed.starts_with("[x] ⏸") {
                    // Parse: ⏸ gate -- deferred: reason -- approved by: christian date
                    let clean = trimmed
                        .trim_start_matches("- [x] ")
                        .trim_start_matches("[x] ")
                        .trim_start_matches("⏸ ");
                    let parts: Vec<&str> = clean.splitn(3, " -- ").collect();
                    let gate = parts.first().copied().unwrap_or(clean).trim().to_string();
                    let reason = parts.get(1).copied()
                        .unwrap_or("deferred")
                        .trim_start_matches("deferred: ")
                        .to_string();
                    let approved_part = parts.get(2).copied().unwrap_or("");
                    let date = approved_part
                        .split_whitespace()
                        .last()
                        .unwrap_or("unknown")
                        .to_string();
                    // Parse date to timestamp
                    let date_ts: i64 = if date.contains('-') {
                        let p: Vec<&str> = date.split('-').collect();
                        if p.len() == 3 {
                            let y: i64 = p[0].parse().unwrap_or(2026);
                            let m: i64 = p[1].parse().unwrap_or(5);
                            let d: i64 = p[2].parse().unwrap_or(1);
                            (y - 1970) * 365 * 86400 + m * 30 * 86400 + d * 86400
                        } else { 0 }
                    } else { 0 };
                    all_deferrals.push((intent_id.clone(), title.clone(), gate, reason, date, date_ts));
                }
            }
        }
    }

    // Sort by intent ID
    all_deferrals.sort_by(|a, b| {
        let ai: i64 = a.0.parse().unwrap_or(0);
        let bi: i64 = b.0.parse().unwrap_or(0);
        ai.cmp(&bi)
    });

    println!();
    println!("  {} {}", "⏸ Deferral Ledger".bright_yellow().bold(), format!("({} total)", all_deferrals.len()).dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let mut old_count = 0;
    let mut current_intent = String::new();
    for (id, title, gate, reason, date, date_ts) in &all_deferrals {
        if *id != current_intent {
            println!();
            let short_title = title.split(" -- ").next().unwrap_or(title);
            println!("  {} INT-{}: {}", "▶".bright_cyan(), id.bright_white(), short_title.dimmed());
            current_intent = id.clone();
        }
        let is_old = *date_ts > 0 && *date_ts < thirty_days_ago;
        if is_old { old_count += 1; }
        let age_marker = if is_old { " ⚠️  >30 days".bright_red().to_string() } else { String::new() };
        let gate_display = if gate.len() > 60 { format!("{}...", &gate[..57]) } else { gate.clone() };
        println!("    {} {} ({}){}", "⏸".yellow(), gate_display.bright_white(), date.dimmed(), age_marker);
        println!("      {} {}", "→".dimmed(), reason.dimmed());
    }

    println!();
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("  {} {} deferrals across {} intents",
        "📊".normal(),
        all_deferrals.len().to_string().bright_white(),
        {
            let mut ids: Vec<&str> = all_deferrals.iter().map(|d| d.0.as_str()).collect();
            ids.dedup();
            ids.len()
        }
    );
    if old_count > 0 {
        println!("  {} {} deferrals older than 30 days -- review before NixOS migration",
            "⚠️".normal(), old_count.to_string().bright_red());
    }
    println!();
    Ok(())
}

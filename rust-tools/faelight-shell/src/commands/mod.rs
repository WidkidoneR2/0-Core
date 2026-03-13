// faelight-shell — command registry
// Phase 1: 10 forest-native commands

use crate::db::ForestDb;
use colored::*;

pub enum CommandResult {
    Output(String),
    Empty,
    Error(String),
    Exit,
}

pub fn execute(line: &str, db: &ForestDb, core_root: &str) -> CommandResult {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let args = &parts[1..];

    match cmd.as_str() {
        "help" | "?" => help(),
        "exit" | "quit" | "q" => CommandResult::Exit,
        "health" => health(db),
        "events" => events(db, args),
        "decisions" => decisions(db),
        "intents" => intents(core_root),
        "tools" => tools(db, core_root),
        "version" => version(core_root),
        "commits" => commits(core_root),
        "story" => story(db),
        "advise" => advise(db),
        "audit" => audit(db, core_root),
        "forecast" => forecast(db),
        "sandbox" => sandbox(db),
        "checkpoint" | "cpc" => checkpoint(db),
        "git" => git_status(core_root),
        "search" | "?" => search(db, args),
        "where" => CommandResult::Error("pipe not yet supported — coming in Phase 2".to_string()),
        "cd" => cd(args),
        "clear" => { print!("\x1B[2J\x1B[1;1H"); CommandResult::Empty }
        _ => CommandResult::Error(format!(
            "Unknown command: {}  — type {} for help",
            cmd.bright_white(), "help".bright_cyan()
        )),
    }
}

fn forecast(db: &ForestDb) -> CommandResult {
    // Read last 10 doctor events and compute trend
    let points: Vec<i64> = {
        let mut stmt = match db.conn.prepare(
            "SELECT payload FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 10"
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Error("No forecast data".to_string()),
        };
        stmt.query_map([], |r| r.get::<_,String>(0))
            .map(|rows| rows.filter_map(|r| r.ok())
                .filter_map(|p| serde_json::from_str::<serde_json::Value>(&p).ok())
                .filter_map(|v| v["detail"]["health"].as_i64())
                .collect())
            .unwrap_or_default()
    };

    if points.len() < 3 {
        return CommandResult::Output(format!(
            "  {} Not enough data for forecast yet — run {} a few times",
            "○".dimmed(), "d".bright_cyan()
        ));
    }

    let current = points[0];
    let recent_avg: f64 = points.iter().take(3).map(|h| *h as f64).sum::<f64>() / 3.0;
    let older_avg: f64 = points.iter().skip(3).map(|h| *h as f64).sum::<f64>() / (points.len() - 3) as f64;
    let trend = recent_avg - older_avg;
    let forecast_24h = (current as f64 + trend * 0.5).round() as i64;
    let forecast_7d = (current as f64 + trend * 2.0).round() as i64;
    let forecast_24h = forecast_24h.max(0).min(100);
    let forecast_7d = forecast_7d.max(0).min(100);

    let trend_icon = if trend > 1.0 { "📈" } else if trend < -1.0 { "📉" } else { "➡️ " };
    let trend_str = if trend > 0.5 { format!("+{:.1}", trend).bright_green().to_string() }
        else if trend < -0.5 { format!("{:.1}", trend).yellow().to_string() }
        else { "stable".dimmed().to_string() };

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 📈 Health Forecast ──────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  Current:  {}%\n", current.to_string().bright_white().bold()));
    out.push_str(&format!("  │  24h:      {}%\n", forecast_24h.to_string().bright_green()));
    out.push_str(&format!("  │  7d:       {}%\n", forecast_7d.to_string().bright_green()));
    out.push_str(&format!("  │  Trend:    {} {}\n", trend_icon, trend_str));
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn sandbox(db: &ForestDb) -> CommandResult {
    let rows: Vec<(String, String, i64)> = {
        let mut stmt = match db.conn.prepare(
            "SELECT payload, action, timestamp FROM events WHERE domain='sandbox' ORDER BY timestamp DESC LIMIT 10"
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Output(format!("  {} No sandbox runs yet", "○".dimmed())),
        };
        stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };

    if rows.is_empty() {
        return CommandResult::Output(format!(
            "  {} No sandbox runs recorded — use {}",
            "○".dimmed(), "faelight-sandbox run".bright_cyan()
        ));
    }

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🧪 Sandbox Runs ───────────────────────────────────".bright_cyan()));
    for (payload, _, ts) in &rows {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
            let cmd = v["detail"]["command"].as_str().unwrap_or("unknown");
            let result = v["result"].as_str().unwrap_or("?");
            let dur = v["detail"]["duration_secs"].as_u64().unwrap_or(0);
            let changed = v["detail"]["files_changed"].as_u64().unwrap_or(0);
            let icon = if result == "ok" { "✅" } else { "❌" };
            let time = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_else(|| "?".to_string());
            let short_cmd = if cmd.len() > 35 { format!("{}...", &cmd[..35]) } else { cmd.to_string() };
            out.push_str(&format!("  │  {} {}  {}  {}s  {} files\n",
                icon,
                time.dimmed(),
                short_cmd.bright_white(),
                dur.to_string().dimmed(),
                changed.to_string().cyan(),
            ));
        }
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn checkpoint(db: &ForestDb) -> CommandResult {
    let rows: Vec<(String, String, i64)> = {
        let mut stmt = match db.conn.prepare(
            "SELECT name, payload, timestamp FROM checkpoints ORDER BY timestamp DESC LIMIT 8"
        ) {
            Ok(s) => s,
            Err(_) => {
                // Try events table for checkpoint events
                let mut stmt2 = match db.conn.prepare(
                    "SELECT action, payload, timestamp FROM events WHERE domain='checkpoint' ORDER BY timestamp DESC LIMIT 8"
                ) {
                    Ok(s) => s,
                    Err(_) => return CommandResult::Output(format!("  {} No checkpoints found", "○".dimmed())),
                };
                return CommandResult::Output({
                    let rows: Vec<(String, String, i64)> = stmt2
                        .query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?)))
                        .map(|rows| rows.filter_map(|r| r.ok()).collect())
                        .unwrap_or_default();
                    if rows.is_empty() {
                        return CommandResult::Output(format!("  {} No checkpoints yet — use {}", "○".dimmed(), "cpc <name>".bright_cyan()));
                    }
                    let mut out = String::new();
                    out.push_str(&format!("\n{}\n", "  ╭─ 📸 Checkpoints ──────────────────────────────────".bright_cyan()));
                    for (action, payload, ts) in &rows {
                        let time = chrono::DateTime::from_timestamp(*ts, 0)
                            .map(|t| t.format("%m-%d %H:%M").to_string())
                            .unwrap_or_else(|| "?".to_string());
                        let name = serde_json::from_str::<serde_json::Value>(payload).ok()
                            .and_then(|v| v["detail"]["name"].as_str().map(|s| s.to_string()))
                            .unwrap_or_else(|| action.clone());
                        out.push_str(&format!("  │  {} {}\n", time.dimmed(), name.bright_white()));
                    }
                    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
                    out
                });
            }
        };
        stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };

    if rows.is_empty() {
        return CommandResult::Output(format!("  {} No checkpoints yet — use {}", "○".dimmed(), "cpc <name>".bright_cyan()));
    }

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 📸 Checkpoints ──────────────────────────────────".bright_cyan()));
    for (name, _, ts) in &rows {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.format("%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "?".to_string());
        out.push_str(&format!("  │  {} {}\n", time.dimmed(), name.bright_white()));
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn git_status(core_root: &str) -> CommandResult {
    let status = std::process::Command::new("git")
        .args(["-C", core_root, "status", "--short"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let branch = std::process::Command::new("git")
        .args(["-C", core_root, "branch", "--show-current"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let recent = std::process::Command::new("git")
        .args(["-C", core_root, "log", "--oneline", "-5"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🌿 Git Status ─────────────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  Branch:  {}\n", branch.bright_green()));

    if status.trim().is_empty() {
        out.push_str(&format!("  │  Status:  {}\n", "clean".bright_green()));
    } else {
        out.push_str(&format!("  │  Status:  {}\n", "uncommitted changes".yellow()));
        for line in status.lines().take(5) {
            out.push_str(&format!("  │    {}\n", line.dimmed()));
        }
    }

    out.push_str(&"  ├─────────────────────────────────────────────────────".dimmed().to_string());
    out.push_str("\n");
    out.push_str(&format!("  │  {}\n", "Recent commits:".dimmed()));
    for line in recent.lines().take(5) {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() == 2 {
            out.push_str(&format!("  │    {} {}\n",
                parts[0].bright_yellow(),
                parts[1].dimmed()
            ));
        }
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn search(db: &ForestDb, args: &[&str]) -> CommandResult {
    let query = args.join(" ").to_lowercase();

    let rows: Vec<(String, i64)> = {
        let mut stmt = match db.conn.prepare(
            "SELECT command, timestamp FROM shell_history ORDER BY timestamp DESC LIMIT 200"
        ) {
            Ok(s) => s,
            Err(_) => return CommandResult::Output(format!("  {} No history yet", "○".dimmed())),
        };
        stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    };

    if query.is_empty() {
        // Show recent history
        let mut out = String::new();
        out.push_str(&format!("\n{}\n", "  ╭─ 📜 Command History ──────────────────────────────".bright_cyan()));
        for (cmd, ts) in rows.iter().take(15) {
            let time = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_else(|| "?".to_string());
            out.push_str(&format!("  │  {} {}\n", time.dimmed(), cmd.bright_white()));
        }
        out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
        return CommandResult::Output(out);
    }

    // Fuzzy search — score by match position and frequency
    let mut matches: Vec<(String, i64, usize)> = rows.iter()
        .filter(|(cmd, _)| cmd.to_lowercase().contains(&query))
        .map(|(cmd, ts)| {
            let score = if cmd.to_lowercase().starts_with(&query) { 0 }
                else if cmd.to_lowercase().contains(&format!(" {}", query)) { 1 }
                else { 2 };
            (cmd.clone(), *ts, score)
        })
        .collect();

    // Deduplicate keeping most recent
    matches.sort_by_key(|(cmd, _, score)| (*score, cmd.clone()));
    matches.dedup_by(|a, b| a.0 == b.0);
    matches.sort_by_key(|(_, ts, score)| (*score, -ts));

    if matches.is_empty() {
        return CommandResult::Output(format!(
            "  {} No matches for {}",
            "○".dimmed(), query.bright_white()
        ));
    }

    let mut out = String::new();
    out.push_str(&format!("\n{}\n",
        format!("  ╭─ 🔍 Search: {} ({} results) ──────────────────────", query, matches.len()).bright_cyan()
    ));
    for (cmd, ts, _) in matches.iter().take(10) {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "?".to_string());
        // Highlight the match
        let highlighted = cmd.replacen(&query, &query.bright_yellow().to_string(), 1);
        out.push_str(&format!("  │  {} {}\n", time.dimmed(), highlighted));
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn cd(args: &[&str]) -> CommandResult {
    let target = args.first().copied().unwrap_or("~");
    let home = std::env::var("HOME").unwrap_or_default();
    let path = if target == "~" || target.is_empty() {
        std::path::PathBuf::from(&home)
    } else if target.starts_with("~/") {
        std::path::PathBuf::from(format!("{}/{}", home, &target[2..]))
    } else {
        std::path::PathBuf::from(target)
    };

    match std::env::set_current_dir(&path) {
        Ok(_) => CommandResult::Empty,
        Err(e) => CommandResult::Error(format!("cd: {}: {}", target, e)),
    }
}

fn help() -> CommandResult {
    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🌲 faelight-shell commands ──────────────────────────".bright_cyan()));
    let cmds = [
        ("health",    "system health and status"),
        ("events",    "recent events  [today|domain]"),
        ("decisions", "open decisions from ledger"),
        ("intents",   "active intents"),
        ("tools",     "tool deployment status"),
        ("audit",     "tool intelligence scores"),
        ("forecast",  "health trend and forecast"),
        ("sandbox",   "recent sandbox runs"),
        ("checkpoint", "recent checkpoints"),
        ("git",        "git status and recent commits"),
        ("search",     "search command history  [query]"),
        ("story",     "30-day forest narrative"),
        ("advise",    "judgment advisory"),
        ("version",   "system version"),
        ("commits",   "commit count and last commit"),
        ("cd",        "change directory"),
        ("clear",     "clear the screen"),
        ("exit",      "leave faelight-shell"),
    ];
    for (cmd, desc) in &cmds {
        out.push_str(&format!("  │  {:<12}  {}\n",
            cmd.bright_cyan(),
            desc.dimmed()
        ));
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn health(db: &ForestDb) -> CommandResult {
    let health = db.health_score().unwrap_or(0);
    let status = if health >= 95 { "HEALTHY".bright_green() }
        else if health >= 80 { "ADVISORY".yellow() }
        else { "DEGRADED".bright_red() };

    let version = std::fs::read_to_string(
        std::path::PathBuf::from(db.core_root()).join("00-meta/VERSION")
    ).unwrap_or_else(|_| "unknown".into()).trim().to_string();

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🏥 Forest Health ─────────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  Health:  {}  {}\n", format!("{}%", health).bright_white().bold(), status));
    out.push_str(&format!("  │  Version: {}\n", version.dimmed()));
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn events(db: &ForestDb, args: &[&str]) -> CommandResult {
    let today_only = args.contains(&"today");
    let domain = args.first().and_then(|a| {
        if *a == "today" { None } else { Some(*a) }
    });

    let label = if today_only { "Today's Events" } else { "Recent Events" };
    let events = db.query_events(domain, today_only, 20);
    if events.is_empty() {
        return CommandResult::Output(format!("  {} No events found", "○".dimmed()));
    }

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", format!("  ╭─ 📊 {} ─────────────────────────────────", label).bright_cyan()));
    for (domain, action, ts) in &events {
        let time = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "?".to_string());
        out.push_str(&format!("  │  {}  {}.{}  {}\n",
            time.dimmed(),
            domain.bright_cyan(),
            action.bright_white(),
            "".to_string()
        ));
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn decisions(db: &ForestDb) -> CommandResult {
    let rows: Vec<(String, String, String)> = db.conn
        .prepare("SELECT dec_id, description, outcome FROM decisions ORDER BY timestamp DESC LIMIT 10")
        .ok()
        .map(|mut s| {
            s.query_map([], |r| {
                Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?))
            })
            .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
            .unwrap_or_default()
        })
        .unwrap_or_default();

    if rows.is_empty() {
        return CommandResult::Output(format!(
            "  {} No decisions recorded yet — use {}",
            "○".dimmed(), "core decide".bright_cyan()
        ));
    }

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ ⚖️  Decisions ──────────────────────────────────────".bright_cyan()));
    for (id, desc, outcome) in &rows {
        let outcome_icon = match outcome.as_str() {
            "success" => "✅".to_string(),
            "failure" => "❌".to_string(),
            "partial" => "⚠️ ".to_string(),
            _ => "○ ".to_string(),
        };
        let short_desc = if desc.len() > 45 {
            format!("{}...", &desc[..45])
        } else {
            desc.clone()
        };
        out.push_str(&format!("  │  {}  {}  {}\n",
            id.bright_yellow(),
            outcome_icon,
            short_desc.dimmed()
        ));
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn intents(core_root: &str) -> CommandResult {
    let future_dir = std::path::PathBuf::from(core_root).join("intents/future");
    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🎯 Active Intents ────────────────────────────────".bright_cyan()));

    if let Ok(entries) = std::fs::read_dir(&future_dir) {
        let mut found = false;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") {
                let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                let title = content.lines()
                    .find(|l| l.starts_with("title:"))
                    .map(|l| l.trim_start_matches("title:").trim().trim_matches('"').to_string())
                    .unwrap_or_else(|| name.clone());
                let id = name.split('-').next().unwrap_or("?");
                out.push_str(&format!("  │  {}  {}\n",
                    format!("INT-{}", id).bright_yellow(),
                    title.dimmed()
                ));
                found = true;
            }
        }
        if !found {
            out.push_str("  │  No active intents\n");
        }
    }
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn tools(db: &ForestDb, core_root: &str) -> CommandResult {
    let tools_dir = std::path::PathBuf::from(core_root).join("rust-tools");
    let total = std::fs::read_dir(&tools_dir)
        .map(|e| e.flatten().filter(|e| e.path().join("Cargo.toml").exists()).count())
        .unwrap_or(0);

    let deployed = std::fs::read_dir(std::path::PathBuf::from(core_root).join("scripts"))
        .map(|e| e.flatten().count())
        .unwrap_or(0);

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🛠  Tools ─────────────────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  Total:    {} tools\n", total.to_string().bright_white().bold()));
    out.push_str(&format!("  │  Deployed: {}/{}\n", deployed.to_string().bright_green(), total));
    out.push_str(&format!("  │  Run {} for intelligence scores\n", "audit".bright_cyan()));
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn version(core_root: &str) -> CommandResult {
    let version = std::fs::read_to_string(
        std::path::PathBuf::from(core_root).join("00-meta/VERSION")
    ).unwrap_or_else(|_| "unknown".into());

    let changelog = std::fs::read_to_string(
        std::path::PathBuf::from(core_root).join("00-meta/CHANGELOG.md")
    ).unwrap_or_default();

    let release_name = changelog.lines()
        .find(|l| l.starts_with("## ["))
        .and_then(|l| l.split('—').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "The Forest Remembers".to_string());

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 🌲 Version ───────────────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  {}  {}\n",
        version.trim().bright_white().bold(),
        release_name.dimmed()
    ));
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn commits(core_root: &str) -> CommandResult {
    let count = std::process::Command::new("git")
        .args(["-C", core_root, "rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "?".to_string());

    let last = std::process::Command::new("git")
        .args(["-C", core_root, "log", "-1", "--format=%s"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "  ╭─ 📚 Commits ───────────────────────────────────────".bright_cyan()));
    out.push_str(&format!("  │  Total:  {}\n", count.bright_white().bold()));
    out.push_str(&format!("  │  Last:   {}\n", last.dimmed()));
    out.push_str(&"  ╰────────────────────────────────────────────────────".dimmed().to_string());
    CommandResult::Output(out)
}

fn story(db: &ForestDb) -> CommandResult {
    // Delegate to core story via process
    let core_root = db.core_root();
    let output = std::process::Command::new(
        format!("{}/scripts/core", core_root)
    )
    .args(["story"])
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .unwrap_or_else(|| "core story not available".to_string());

    CommandResult::Output(output)
}

fn advise(db: &ForestDb) -> CommandResult {
    let core_root = db.core_root();
    let output = std::process::Command::new(
        format!("{}/scripts/core", core_root)
    )
    .args(["advise"])
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .unwrap_or_else(|| "core advise not available".to_string());

    CommandResult::Output(output)
}

fn audit(db: &ForestDb, core_root: &str) -> CommandResult {
    let output = std::process::Command::new(
        format!("{}/scripts/core", core_root)
    )
    .args(["audit", "scan"])
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .unwrap_or_else(|| "core audit not available".to_string());

    CommandResult::Output(output)
}

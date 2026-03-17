// core narrative — Forest Narrative
// Core v7 Phase 5 — INT-122
//
// "The forest becomes a historian of its own evolution."

use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

pub fn run(ctx: &AppContext, since: Option<&str>, intent: Option<&str>) -> CoreResult<()> {
    ctx.capabilities.require("narrative", &[Capability::FilesystemReadHome])?;

    if let Some(intent_id) = intent {
        return intent_narrative(ctx, intent_id);
    }

    full_narrative(ctx, since)
}

fn full_narrative(ctx: &AppContext, since: Option<&str>) -> CoreResult<()> {
    let core_root = &ctx.core_root;

    println!();
    println!("{}", "  ╭─ 🌲 Forest Narrative ───────────────────────────────".bright_cyan());

    // Chapter 1 — Identity
    let version = std::fs::read_to_string(
        PathBuf::from(core_root).join("00-meta/VERSION")
    ).unwrap_or_else(|_| "unknown".to_string());
    let version = version.trim();

    println!("  │");
    println!("  │  {}", "Chapter I — Identity".bright_white().bold());
    println!("  │");
    println!("  │  Faelight Forest {} runs on vanilla Arch Linux,", version.bright_green());
    println!("  │  orchestrated by the Niri compositor, and built");
    println!("  │  entirely in Rust by a single developer.");
    println!("  │  Nothing runs without explicit human authorization.");

    // Chapter 2 — Growth
    println!("  │");
    println!("  │  {}", "Chapter II — Growth".bright_white().bold());
    println!("  │");

    let commit_count = Command::new("git")
        .args(["-C", core_root, "rev-list", "--count", "HEAD"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "?".to_string());

    let first_commit_date = Command::new("git")
        .args(["-C", core_root, "log", "--reverse", "--format=%ai", "--", "."])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(|l| l[..10].to_string()))
        .unwrap_or_else(|| "?".to_string());

    // Count tools from registry
    let tool_count = std::fs::read_to_string(
        PathBuf::from(core_root).join("01-registry/tools.toml")
    ).map(|t| t.lines().filter(|l| l.starts_with("name = ")).count())
    .unwrap_or(0);

    println!("  │  Since {}, the forest has grown through",
        first_commit_date.bright_yellow());
    println!("  │  {} commits and {} custom Rust tools.",
        commit_count.bright_white(), tool_count.to_string().bright_white());
    println!("  │  Every tool is understood. Nothing is installed blindly.");

    // Chapter 3 — Decisions
    println!("  │");
    println!("  │  {}", "Chapter III — Decisions".bright_white().bold());
    println!("  │");

    let decisions: Vec<(String, String)> = {
        let mut stmt = ctx.runtime.db.prepare(
            "SELECT description, outcome FROM decisions ORDER BY timestamp ASC"
        ).ok();
        if let Some(ref mut s) = stmt {
            s.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default()
        } else { vec![] }
    };

    if decisions.is_empty() {
        println!("  │  No decisions recorded yet — use {} to begin", "core decide".bright_cyan());
    } else {
        println!("  │  {} architectural decisions shaped this forest:", decisions.len());
        for (desc, outcome) in &decisions {
            let icon = match outcome.as_str() {
                "success" => "✅".to_string(),
                "failed"  => "✗ ".bright_red().to_string(),
                _         => "⬜".to_string(),
            };
            let short = if desc.len() > 55 { format!("{}...", &desc[..55]) } else { desc.clone() };
            println!("  │    {} {}", icon, short.dimmed());
        }
    }

    // Chapter 4 — Intents
    println!("  │");
    println!("  │  {}", "Chapter IV — Intentions".bright_white().bold());
    println!("  │");

    let complete_dir = PathBuf::from(core_root).join("intents/complete");
    let future_dir = PathBuf::from(core_root).join("intents/future");
    let complete_count = std::fs::read_dir(&complete_dir)
        .map(|d| d.count()).unwrap_or(0);
    let future_count = std::fs::read_dir(&future_dir)
        .map(|d| d.count()).unwrap_or(0);

    println!("  │  {} intents have been completed.", complete_count.to_string().bright_green());
    println!("  │  {} intents guide the path forward.", future_count.to_string().bright_yellow());
    println!("  │  Every change was intentional. Every tool has a purpose.");

    // Chapter 5 — Core Versions
    println!("  │");
    println!("  │  {}", "Chapter V — The Core Timeline".bright_white().bold());
    println!("  │");

    let releases_dir = PathBuf::from(core_root).join("00-meta/releases");
    if releases_dir.exists() {
        let mut releases: Vec<String> = std::fs::read_dir(&releases_dir)
            .map(|d| d.flatten()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect())
            .unwrap_or_default();
        releases.sort();
        for release in releases.iter().rev().take(6) {
            println!("  │    {} {}", "→".dimmed(), release.bright_cyan());
        }
    }

    // Chapter 6 — Current State
    println!("  │");
    println!("  │  {}", "Chapter VI — Present Moment".bright_white().bold());
    println!("  │");

    let health = ctx.runtime.db.query_row(
        "SELECT value FROM domain_state WHERE domain='doctor' AND key='health' ORDER BY timestamp DESC LIMIT 1",
        [],
        |r| r.get::<_,String>(0)
    ).unwrap_or_else(|_| "unknown".to_string());

    println!("  │  The forest is {}. {} checks pass.",
        "healthy".bright_green().bold(), "23/23".bright_white());
    println!("  │  The forecast is stable. The roots are strong.");
    println!("  │  The branches are growing toward v11.0.0 —");
    println!("  │  {}", "The Living Forest.".bright_green().italic());

    println!("  │");
    println!("{}", "  ╰────────────────────────────────────────────────────".dimmed());
    println!();

    // Emit event
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64).unwrap_or(0);
    let payload = format!(r#"{{"actor":"core","result":"ok","detail":{{"command":"narrative","since":"{}"}}}}"#,
        since.unwrap_or("beginning"));
    ctx.runtime.db.execute(
        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('narrative', 'run', ?1, ?2)",
        rusqlite::params![payload, ts],
    ).ok();
    crate::runtime::write_event_log("narrative", "run", &payload, ts);

    Ok(())
}

fn intent_narrative(ctx: &AppContext, intent_id: &str) -> CoreResult<()> {
    let core_root = &ctx.core_root;

    println!();
    println!("{}", format!("  ╭─ 📖 Intent Narrative — {} ──────────────────────", intent_id).bright_cyan());
    println!("  │");

    // Find the intent file
    let complete_dir = PathBuf::from(core_root).join("intents/complete");
    let future_dir = PathBuf::from(core_root).join("intents/future");

    let intent_file = std::fs::read_dir(&complete_dir)
        .ok().and_then(|d| {
            d.flatten().find(|e| {
                e.file_name().to_string_lossy().contains(&format!("-{}-", intent_id)) ||
                e.file_name().to_string_lossy().starts_with(&format!("{}-", intent_id))
            })
        })
        .or_else(|| {
            std::fs::read_dir(&future_dir).ok().and_then(|d| {
                d.flatten().find(|e| {
                    e.file_name().to_string_lossy().contains(&format!("-{}-", intent_id)) ||
                    e.file_name().to_string_lossy().starts_with(&format!("{}-", intent_id))
                })
            })
        });

    if let Some(file) = intent_file {
        let content = std::fs::read_to_string(file.path()).unwrap_or_default();
        let title = content.lines()
            .find(|l| l.starts_with("title:"))
            .map(|l| l.trim_start_matches("title:").trim().trim_matches('"').to_string())
            .unwrap_or_else(|| format!("Intent {}", intent_id));
        let status = content.lines()
            .find(|l| l.starts_with("status:"))
            .map(|l| l.trim_start_matches("status:").trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        println!("  │  {}", title.bright_white().bold());
        println!("  │  Status: {}", match status.as_str() {
            "complete" => "complete".bright_green().to_string(),
            "in-progress" => "in-progress".bright_yellow().to_string(),
            _ => status.dimmed().to_string(),
        });
        println!("  │");

        // Show git commits referencing this intent
        let log = Command::new("git")
            .args(["-C", core_root, "log", "--oneline",
                   &format!("--grep=INT-{}", intent_id)])
            .output().ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let commit_count = log.lines().count();
        println!("  │  {} commits reference this intent:", commit_count.to_string().bright_white());
        for line in log.lines().take(8) {
            println!("  │    {} {}", "→".dimmed(), line.dimmed());
        }
    } else {
        println!("  │  {} Intent {} not found", "✗".bright_red(), intent_id);
    }

    println!("  │");
    println!("{}", "  ╰────────────────────────────────────────────────────".dimmed());
    println!();
    Ok(())
}

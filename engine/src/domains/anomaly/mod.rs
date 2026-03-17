// core anomaly — Anomaly Detection
// Core v7 Phase 1 — INT-122
//
// "If a file changes without a corresponding decision or intent,
//  the forest notices. Not blocking — observing."

use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::process::Command;

#[derive(Debug)]
struct Anomaly {
    description: String,
    file: String,
    commit: String,
    date: String,
    severity: Severity,
}

#[derive(Debug)]
#[allow(dead_code)]
enum Severity { Low, Medium, High }

impl Severity {
    fn label(&self) -> colored::ColoredString {
        match self {
            Severity::Low    => "low".dimmed(),
            Severity::Medium => "medium".yellow(),
            Severity::High   => "high".bright_red(),
        }
    }
    fn icon(&self) -> &'static str {
        match self {
            Severity::Low    => "○",
            Severity::Medium => "⚠",
            Severity::High   => "✗",
        }
    }
}

pub fn scan(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require("anomaly", &[Capability::FilesystemReadHome])?;
    let core_root = &ctx.core_root;
    let anomalies = detect_anomalies(core_root);

    println!();
    println!("{}", "  ╭─ 🔍 Anomaly Detection Scan ─────────────────────────".bright_cyan());

    if anomalies.is_empty() {
        println!("  │  {} No anomalies detected", "✅".green());
        println!("  │  All recent changes traceable to intents or decisions");
    } else {
        println!("  │  {} anomalies detected", anomalies.len().to_string().yellow().bold());
        println!("{}", "  ├─────────────────────────────────────────────────────".dimmed());
        for a in &anomalies {
            println!("  │  {} {} ({})",
                a.severity.icon().yellow(),
                a.description.bright_white(),
                a.severity.label()
            );
            if !a.file.is_empty() {
                println!("  │    {} {}", "file:".dimmed(), a.file.dimmed());
            }
            if !a.commit.is_empty() {
                println!("  │    {} {} — {}", "commit:".dimmed(), a.commit.bright_yellow(), a.date.dimmed());
            }
        }
    }

    println!("{}", "  ╰─────────────────────────────────────────────────────".dimmed());
    println!();

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64).unwrap_or(0);
    let payload = format!(
        r#"{{"actor":"core","result":"ok","detail":{{"anomalies":{},"scan":"anomaly"}}}}"#,
        anomalies.len()
    );
    ctx.runtime.db.execute(
        "INSERT INTO events (domain, action, payload, timestamp) VALUES ('anomaly', 'scan', ?1, ?2)",
        rusqlite::params![payload, ts],
    ).ok();
    crate::runtime::write_event_log("anomaly", "scan", &payload, ts);

    Ok(())
}

pub fn history(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require("anomaly", &[Capability::FilesystemReadHome])?;
    let db = &ctx.runtime.db;
    let mut stmt = db.prepare(
        "SELECT payload, timestamp FROM events WHERE domain='anomaly' ORDER BY timestamp DESC LIMIT 10"
    )?;

    let rows: Vec<(String, i64)> = stmt.query_map([], |r| {
        Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?))
    })?.filter_map(|r| r.ok()).collect();

    println!();
    println!("{}", "  ╭─ 📜 Anomaly History ────────────────────────────────".bright_cyan());

    if rows.is_empty() {
        println!("  │  {} No anomaly scans yet — run: {}", "○".dimmed(), "core anomaly scan".bright_cyan());
    } else {
        for (payload, ts) in &rows {
            let date = chrono::DateTime::from_timestamp(*ts, 0)
                .map(|t| t.format("%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "?".to_string());
            let count = serde_json::from_str::<serde_json::Value>(payload).ok()
                .and_then(|v| v["detail"]["anomalies"].as_i64()).unwrap_or(0);
            let icon = if count == 0 { "✅".to_string() } else { "⚠".yellow().to_string() };
            println!("  │  {} {}  {} anomalies", icon, date.dimmed(), count.to_string().bright_white());
        }
    }
    println!("{}", "  ╰─────────────────────────────────────────────────────".dimmed());
    println!();
    Ok(())
}

pub fn alert(ctx: &AppContext) -> CoreResult<()> {
    ctx.capabilities.require("anomaly", &[Capability::FilesystemReadHome])?;
    let anomalies = detect_anomalies(&ctx.core_root);
    let high_count = anomalies.iter().filter(|a| matches!(a.severity, Severity::High)).count();

    if high_count == 0 {
        println!("  {} No high-severity anomalies", "✅".green());
    } else {
        println!("{}", format!("  {} high-severity anomalies require attention", high_count).bright_red().bold());
        for a in anomalies.iter().filter(|a| matches!(a.severity, Severity::High)) {
            println!("    {} {}", "→".bright_red(), a.description.bright_white());
        }
    }
    Ok(())
}

fn detect_anomalies(core_root: &str) -> Vec<Anomaly> {
    let mut anomalies = Vec::new();
    anomalies.extend(detect_unintented_commits(core_root));
    anomalies.extend(detect_registry_anomalies(core_root));
    anomalies
}

fn detect_unintented_commits(core_root: &str) -> Vec<Anomaly> {
    let mut anomalies = Vec::new();
    let output = Command::new("git")
        .args(["-C", core_root, "log", "--format=%H|%s|%ai",
               "--since=30 days ago", "--", "engine/", "rust-tools/"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() < 3 { continue; }
        let hash = &parts[0][..7.min(parts[0].len())];
        let msg = parts[1];
        let date = &parts[2][..10.min(parts[2].len())];
        let has_intent = msg.contains("INT-") || msg.starts_with("ledger:")
            || msg.starts_with("docs:") || msg.starts_with("fix:")
            || msg.starts_with("chore:") || msg.starts_with("update ");
        if !has_intent {
            anomalies.push(Anomaly {
                description: format!("Commit without intent ref: {}", &msg[..msg.len().min(50)]),
                file: String::new(),
                commit: hash.to_string(),
                date: date.to_string(),
                severity: Severity::Low,
            });
        }
    }
    anomalies
}

fn detect_registry_anomalies(core_root: &str) -> Vec<Anomaly> {
    let mut anomalies = Vec::new();
    let output = Command::new("git")
        .args(["-C", core_root, "log", "--format=%H|%s|%ai",
               "--since=7 days ago", "--", "01-registry/"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let count = output.lines().count();
    if count > 5 {
        anomalies.push(Anomaly {
            description: format!("Registry modified {} times in 7 days", count),
            file: "01-registry/".to_string(),
            commit: String::new(),
            date: String::new(),
            severity: Severity::Medium,
        });
    }
    anomalies
}

// INT-143 Phase 1 — Forest Digest
// Morning/long-gap summary of what changed while you were away.
// Triggered when session gap > 4 hours.

use crate::db::ForestDb;
use crate::session::SessionMemory;
use chrono::Timelike;
use colored::*;

pub fn should_show(mem: &SessionMemory) -> bool {
    // Show on long gaps (4+ hours) or morning sessions (5am-10am)
    let long_gap = mem.hours_since().map(|h| h >= 4).unwrap_or(false);
    let morning = {
        let hour = chrono::Local::now().hour();
        (5..=10).contains(&hour)
    };
    long_gap || morning
}

pub fn render(mem: &SessionMemory, db: &ForestDb, core_root: &str) -> String {
    use chrono::Timelike;
    let mut lines: Vec<String> = vec![];

    // Time-aware greeting
    let hour = chrono::Local::now().hour();
    let greeting = match hour {
        5..=11 => "Good morning.",
        12..=17 => "Good afternoon.",
        18..=21 => "Good evening.",
        _ => "Welcome back.",
    };

    lines.push(format!(
        "  {} {}",
        "🌲".normal(),
        greeting.bright_white().bold()
    ));
    lines.push(String::new());

    // Commits since last session
    let new_commits = mem.new_commits();
    if new_commits > 0 && mem.last_commit_count > 0 {
        lines.push(format!("  {} Since last session:", "→".bright_cyan()));
        lines.push(format!(
            "    {} {} new commit{}",
            "·".dimmed(),
            new_commits.to_string().bright_green().bold(),
            if new_commits == 1 { "" } else { "s" }
        ));
    }

    // Health + forecast
    let health = db.health_score().unwrap_or(0);
    let health_str = if health >= 95 {
        format!("{}% healthy", health).bright_green().to_string()
    } else if health >= 80 {
        format!("{}% advisory", health).yellow().to_string()
    } else {
        format!("{}% degraded", health).bright_red().to_string()
    };
    lines.push(format!("    {} Health: {}", "·".dimmed(), health_str));

    // Active intents
    let intents_path = std::path::Path::new(core_root).join("intents/future");
    let mut active_intents: Vec<String> = vec![];
    if let Ok(entries) = std::fs::read_dir(&intents_path) {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if content.contains("status: in-progress") {
                    let name = entry
                        .file_name()
                        .to_string_lossy()
                        .trim_end_matches(".md")
                        .to_string();
                    // Extract INT number
                    if let Some(num) = name.split('-').next() {
                        active_intents.push(format!("INT-{}", num));
                    }
                }
            }
        }
    }

    if !active_intents.is_empty() {
        lines.push(format!(
            "    {} Working on: {}",
            "·".dimmed(),
            active_intents.join(", ").bright_cyan()
        ));
    }

    // Pending decisions older than 7 days
    let old_pending: i64 = db
        .conn
        .query_row(
            &format!(
                "SELECT COUNT(*) FROM decisions WHERE outcome='pending' AND timestamp < {}",
                chrono::Utc::now().timestamp() - 7 * 86400
            ),
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if old_pending > 0 {
        lines.push(format!(
            "    {} {} pending decision{} older than 7 days",
            "·".yellow(),
            old_pending.to_string().yellow(),
            if old_pending == 1 { "" } else { "s" }
        ));
    }

    // Low audit score tools
    let low_score: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_scores WHERE score < 70",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if low_score > 0 {
        lines.push(format!(
            "    {} {} tool{} with audit score < 70",
            "·".yellow(),
            low_score.to_string().yellow(),
            if low_score == 1 { "" } else { "s" }
        ));
    }

    lines.push(String::new());
    lines.push(format!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    ));

    lines.join("\n")
}

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

/// Blocked and ready planned-intent counts.
///
/// EXTRACTED so the banner and the digest read ONE computation. It was inline in
/// render, which meant the count only appeared after a four-hour gap or in the
/// morning -- and putting a second copy in the banner would have been the
/// two-owners defect this ledger keeps removing.
///
/// Reads depends_on from frontmatter and tests each dependency against the ids in
/// complete/. Agrees with core intent blocked.
pub fn blocked_ready() -> (usize, usize) {
    let intents_future = faelight_core::paths::intents_dir().join("future");
    let intents_complete = faelight_core::paths::intents_dir().join("complete");
    let mut complete_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(entries) = std::fs::read_dir(&intents_complete) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(num) = name.split('-').next() {
                complete_ids.insert(num.to_string());
            }
        }
    }
    let mut blocked_count = 0usize;
    let mut ready_count = 0usize;
    if let Ok(entries) = std::fs::read_dir(&intents_future) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(&p) {
                if !text.contains("status: planned") {
                    continue;
                }
                let deps: Vec<String> = text
                    .lines()
                    .find(|l| l.trim_start().starts_with("depends_on:"))
                    .and_then(|l| {
                        let v = l
                            .splitn(2, ':')
                            .nth(1)?
                            .trim()
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .to_string();
                        if v.is_empty() {
                            return Some(vec![]);
                        }
                        Some(v.split(',').map(|s| s.trim().to_string()).collect())
                    })
                    .unwrap_or_default();
                let has_unmet = deps.iter().any(|d| {
                    let d = d.trim();
                    !d.is_empty() && !complete_ids.contains(d)
                });
                if has_unmet {
                    blocked_count += 1;
                } else {
                    ready_count += 1;
                }
            }
        }
    }
    (blocked_count, ready_count)
}

pub fn render(_mem: &SessionMemory, db: &ForestDb, _core_root: &str) -> String {
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
        "◉".normal(),
        greeting.bright_white().bold()
    ));
    lines.push(String::new());

    // Active intents
    let intents_path = faelight_core::paths::intents_dir().join("future");
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

    // INT-247 Phase 5: blocked + ready counts, computed in blocked_ready above.
    {
        let (_blocked_count, ready_count) = blocked_ready();
        // BLOCKED IS ON THE BANNER, every session. It was printed here too, and
        // twice on screen is worse than once in the wrong place. READY STAYS: it is
        // a prompt to act rather than a piece of state, and the digest exists to say
        // what to pick up after being away.
        if ready_count > 0 {
            lines.push(format!(
                "    · {} intent{} ready -- run: core intent next",
                ready_count.to_string().bright_green(),
                if ready_count == 1 { "" } else { "s" }
            ));
        }
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

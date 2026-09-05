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
    // INT-230: was a hand-rolled complete/ id set plus its own depends_on
    // frontmatter parser -- a second implementation of what the adapter
    // owns. Absence yields (0, 0), which renders as no ready prompt.
    crate::core_integration::ledger()
        .map(|l| l.blocked_ready())
        .unwrap_or((0, 0))
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
    // INT-230: scanned future/ for `status: in-progress`. cistart moves a
    // started intent into in-progress/, so this list has been empty for as
    // long as that move has existed -- the banner could not name the work
    // actually in progress. The adapter reads the lifecycle folders using
    // the same frontmatter predicate core does.
    let mut active_intents: Vec<String> = match crate::core_integration::ledger() {
        Some(l) => l.active().iter().map(|i| format!("INT-{}", i.id)).collect(),
        None => vec![],
    };
    active_intents.sort();

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

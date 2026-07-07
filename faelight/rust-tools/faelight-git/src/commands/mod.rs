//! Command implementations
pub mod rollback;

pub mod branch;
pub mod commit;
pub mod done;
pub mod log;
pub mod quick;
pub mod risk;
pub mod status;
pub mod sync;

use rusqlite;

/// Active in-progress intent, scanned from intents/in-progress/ (matches fsh prompt + `fg done`).
/// Returns (id, short_title). Single source of truth -- callers: done.rs, sync.rs.
pub(crate) fn get_active_intent() -> Option<(String, String)> {
    let _home = std::env::var("HOME").ok()?;
    let dir = faelight_core::paths::intents_dir()
        .join("in-progress")
        .to_string_lossy()
        .to_string();
    let mut entries: Vec<_> = std::fs::read_dir(&dir).ok()?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).ok()?;
        if content.contains("status: in-progress") {
            let id = name.split('-').next().unwrap_or("").to_string();
            let title = content
                .lines()
                .find(|l| l.starts_with("title:"))
                .map(|l| {
                    l.trim_start_matches("title:")
                        .trim()
                        .trim_matches('"')
                        .to_string()
                })
                .unwrap_or_else(|| format!("INT-{}", id));
            let short = title
                .split(" -- ")
                .next()
                .unwrap_or(&title)
                .trim()
                .to_string();
            return Some((id, short));
        }
    }
    None
}

/// Record a commit into the intent_commits genealogy table (INT-312 recorder, INT-071 shared).
/// Single recorder; called by BOTH `fg done` and `fg sync` so the table never goes stale again.
/// gate_hint deferred to Phase 3; intent_status honest (in-progress when an intent is active, else none).
pub(crate) fn record_commit(hash: &str, message: &str) {
    let db_path = faelight_core::paths::state_db();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    // INT-071 attribution: the commit message is ground truth. Parse a leading
    // INT-NNN (e.g. "INT-071 Phase 1: ...") first; fall back to the active-intent
    // scan only when the message carries no explicit intent. Avoids mis-attributing
    // to the lowest-numbered active intent when several are in-progress.
    let msg_intent: Option<i64> = message.trim_start().strip_prefix("INT-").and_then(|rest| {
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse().ok()
    });
    let intent_id: Option<i64> =
        msg_intent.or_else(|| get_active_intent().and_then(|(id, _)| id.parse().ok()));
    let intent_status = if intent_id.is_some() {
        "in-progress"
    } else {
        "none"
    };
    // INT-071 gate_hint: record the next open gate (first unchecked - [ ] in the charter)
    // for the RESOLVED intent_id -- the same intent attribution chose (message or active),
    // not a second independent get_active_intent() call. Scans in-progress/ and future/.
    let gate_hint: Option<String> = intent_id.and_then(|iid| {
        let _home = std::env::var("HOME").unwrap_or_default();
        let prefix = format!("{:03}-", iid);
        for sub in ["in-progress", "future"] {
            let dir = faelight_core::paths::intents_dir()
                .join(sub)
                .to_string_lossy()
                .to_string();
            let entries = match std::fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.starts_with(&prefix) {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for line in content.lines() {
                        if let Some(rest) = line.strip_prefix("- [ ] ") {
                            return Some(rest.trim().chars().take(80).collect::<String>());
                        }
                    }
                }
            }
        }
        None
    });
    let author: Option<String> = std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let health: Option<i64> = conn
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM shell_state WHERE key = 'last_health'",
            [],
            |r| r.get(0),
        )
        .ok();
    let friday_facts: Option<i64> = conn
        .query_row("SELECT COUNT(*) FROM friday_knowledge", [], |r| r.get(0))
        .ok();
    let friday_patterns: Option<i64> = conn
        .query_row("SELECT COUNT(*) FROM friday_patterns", [], |r| r.get(0))
        .ok();
    let session_id: Option<String> = conn
        .query_row(
            "SELECT value FROM shell_state WHERE key = 'current_session'",
            [],
            |r| r.get(0),
        )
        .ok();
    let phase_hint = message
        .split_whitespace()
        .zip(message.split_whitespace().skip(1))
        .find_map(|(a, b)| {
            if a.to_lowercase() == "phase" {
                Some(format!(
                    "Phase {}",
                    b.trim_end_matches(':').trim_end_matches(',')
                ))
            } else {
                None
            }
        });
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let short_hash = &hash[..12.min(hash.len())];
    let _ = conn.execute(
        "INSERT OR IGNORE INTO intent_commits
         (commit_hash, intent_id, intent_status, phase_hint, gate_hint, health_at,
          friday_facts, friday_patterns, session_id, committed_at, author, message)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        rusqlite::params![
            short_hash,
            intent_id,
            intent_status,
            phase_hint,
            gate_hint,
            health,
            friday_facts,
            friday_patterns,
            session_id,
            now,
            author,
            message
        ],
    );
}

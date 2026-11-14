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

/// Active in-progress intent, from the status line. Callers: done.rs, sync.rs.
/// Returns (id, short_title).
pub(crate) fn get_active_intent() -> Option<(String, String)> {
    let mut hits: Vec<_> = zero_core::ledger::load()
        .into_iter()
        .filter(|file| !file.is_record() && file.status == "in-progress")
        .collect();
    hits.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
    let file = hits.into_iter().next()?;
    let title = if file.title.is_empty() {
        format!("INT-{}", file.id)
    } else {
        file.title
    };
    let short = title
        .split(" -- ")
        .next()
        .unwrap_or(&title)
        .trim()
        .to_string();
    Some((file.id, short))
}

/// The first unchecked gate in the live file for this intent id.
pub(crate) fn next_open_gate(intent_id: i64) -> Option<String> {
    let want = intent_id.to_string();
    let file = zero_core::ledger::live()
        .into_iter()
        .find(|f| !f.is_record() && f.id == want)?;
    for line in file.content.lines() {
        if let Some(rest) = line.strip_prefix("- [ ] ") {
            return Some(rest.trim().chars().take(80).collect());
        }
    }
    None
}

/// Record a commit into the intent_commits genealogy table (INT-312 recorder, INT-071 shared).
/// Single recorder; called by BOTH `fg done` and `fg sync` so the table never goes stale again.
/// gate_hint deferred to Phase 3; intent_status honest (in-progress when an intent is active, else none).
pub(crate) fn record_commit(hash: &str, message: &str) {
    let db_path = zero_core::paths::state_db();
    let conn = match zero_core::state_db::open_at(
        std::path::Path::new(&db_path),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
    ) {
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
    // INT-071 gate_hint: the first unchecked box in the live file for the
    // resolved intent id. The file comes from the ledger, not from in-progress/ or future/.
    let gate_hint: Option<String> = intent_id.and_then(next_open_gate);
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

#[cfg(test)]
mod tests {
    use super::commit::in_progress_refs;
    use super::next_open_gate;

    static HOME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn shelf() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "zero-git-ledger-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let open = dir.join("0-core/zero/intents/open");
        std::fs::create_dir_all(&open).unwrap();
        std::fs::write(
            open.join("010-planned.md"),
            "---\nid: 10\nstatus: planned\ntitle: \"Stay\"\n---\n\nstatus: in-progress\n- [ ] not this one\n",
        )
        .unwrap();
        std::fs::write(
            open.join("020-active.md"),
            "---\nid: 20\nstatus: in-progress\ntitle: \"Working\"\n---\n\n- [ ] write the gate\n- [ ] later\n",
        )
        .unwrap();
        let future = dir.join("0-core/zero/intents/future");
        std::fs::create_dir_all(&future).unwrap();
        std::fs::write(
            future.join("040-old.md"),
            "---\nid: 40\nstatus: in-progress\ntitle: \"Old\"\n---\n\n- [ ] old gate\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn active_refs_come_from_the_live_shelf() {
        let dir = shelf();
        let _guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var("HOME").ok();
        // SAFETY: restored before the test returns.
        unsafe { std::env::set_var("HOME", &dir) };
        let refs = in_progress_refs();
        match prev {
            Some(h) => unsafe { std::env::set_var("HOME", h) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(refs, vec![("INT-20".to_string(), "Working".to_string())]);
    }

    #[test]
    fn the_open_gate_is_read_from_the_live_file() {
        let dir = shelf();
        let _guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var("HOME").ok();
        // SAFETY: restored before the test returns.
        unsafe { std::env::set_var("HOME", &dir) };
        let gate = next_open_gate(20);
        let missing = next_open_gate(40);
        match prev {
            Some(h) => unsafe { std::env::set_var("HOME", h) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(gate.as_deref(), Some("write the gate"));
        assert_eq!(missing, None);
    }
}

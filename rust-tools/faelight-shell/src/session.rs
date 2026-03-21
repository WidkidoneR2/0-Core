// faelight-shell — Session Memory
// INT-135 Pillar 1: The shell remembers where you left off.
//
// Reads from state.db:
//   - Last session timestamp
//   - Commit count at last session
//   - Last focused intent
//
// Produces: a personalized welcome message based on what changed.

use colored::*;

pub struct SessionMemory {
    pub last_commit_count: u64,
    pub current_commit_count: u64,
    pub last_intent: Option<String>,
    pub last_session_ts: Option<i64>,
}

impl SessionMemory {
    pub fn load(core_root: &str) -> Option<Self> {
        let db_path = std::path::Path::new(core_root).join("runtime/state.db");
        let conn = rusqlite::Connection::open(&db_path).ok()?;

        // Ensure session_state table exists
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS session_state (
                key TEXT PRIMARY KEY,
                value TEXT
            )
        ").ok()?;

        // Read last session data
        let last_commits: u64 = conn.query_row(
            "SELECT value FROM session_state WHERE key='last_commit_count'",
            [], |r| r.get::<_, String>(0)
        ).ok().and_then(|v| v.parse().ok()).unwrap_or(0);

        let last_intent: Option<String> = conn.query_row(
            "SELECT value FROM session_state WHERE key='last_intent'",
            [], |r| r.get(0)
        ).ok();

        let last_ts: Option<i64> = conn.query_row(
            "SELECT value FROM session_state WHERE key='last_session_ts'",
            [], |r| r.get::<_, String>(0)
        ).ok().and_then(|v| v.parse().ok());

        // Read current commit count
        let commits_path = std::path::Path::new("/etc/faelight/COMMITS");
        let current_commits: u64 = std::fs::read_to_string(commits_path)
            .unwrap_or_default().trim().parse().unwrap_or(0);

        Some(SessionMemory {
            last_commit_count:    last_commits,
            current_commit_count: current_commits,
            last_intent,
            last_session_ts:      last_ts,
        })
    }

    pub fn save(core_root: &str, current_intent: Option<&str>) {
        let db_path = std::path::Path::new(core_root).join("runtime/state.db");
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let _ = conn.execute_batch("
                CREATE TABLE IF NOT EXISTS session_state (
                    key TEXT PRIMARY KEY,
                    value TEXT
                )
            ");

            let commits = std::fs::read_to_string("/etc/faelight/COMMITS")
                .unwrap_or_default().trim().to_string();
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default();

            let _ = conn.execute(
                "INSERT OR REPLACE INTO session_state (key, value) VALUES ('last_commit_count', ?1)",
                rusqlite::params![commits]
            );
            let _ = conn.execute(
                "INSERT OR REPLACE INTO session_state (key, value) VALUES ('last_session_ts', ?1)",
                rusqlite::params![ts]
            );
            if let Some(intent) = current_intent {
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO session_state (key, value) VALUES ('last_intent', ?1)",
                    rusqlite::params![intent]
                );
            }
        }
    }

    pub fn days_since(&self) -> Option<u64> {
        let ts = self.last_session_ts?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?.as_secs() as i64;
        Some(((now - ts) / 86400) as u64)
    }

    pub fn hours_since(&self) -> Option<u64> {
        let ts = self.last_session_ts?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?.as_secs() as i64;
        Some(((now - ts) / 3600) as u64)
    }

    pub fn new_commits(&self) -> u64 {
        self.current_commit_count.saturating_sub(self.last_commit_count)
    }
}

// ── Read active intents from filesystem ──────────────────────────────────────────

fn active_intents(core_root: &str) -> Vec<String> {
    let future = std::path::Path::new(core_root).join("intents/future");
    let mut intents = vec![];
    if let Ok(entries) = std::fs::read_dir(&future) {
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".md") { continue; }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if content.contains("status: in-progress") {
                    // Extract INT number from filename (e.g. "120-faelight-shell.md" -> "INT-120")
                    let int_num = name.split('-').next().unwrap_or("?");
                    intents.push(format!("INT-{}", int_num));
                }
            }
        }
    }
    intents
}

// ── Render session memory message ─────────────────────────────────────────────

pub fn render(mem: &SessionMemory, core_root: &str) -> String {
    let mut lines: Vec<String> = vec![];

    // Time away message
    if let Some(hours) = mem.hours_since() {
        if hours == 0 {
            lines.push(format!("  {} {}",
                "↺".bright_cyan(),
                "Welcome back.".dimmed()
            ));
        } else if hours < 2 {
            lines.push(format!("  {} {}",
                "↺".bright_cyan(),
                "Welcome back — picking up where you left off.".dimmed()
            ));
        } else if hours < 24 {
            lines.push(format!("  {} {} hours since last session.",
                "↺".bright_cyan(),
                hours.to_string().bright_white()
            ));
        } else if let Some(days) = mem.days_since() {
            if days == 1 {
                lines.push(format!("  {} {}",
                    "↺".bright_cyan(),
                    "The forest waited. Welcome back.".dimmed()
                ));
            } else {
                lines.push(format!("  {} {} {} {}",
                    "↺".bright_cyan(),
                    days.to_string().bright_white(),
                    "days since last session.".dimmed(),
                    "The forest remembers.".dimmed().italic()
                ));
            }
        }
    }

    // Last intent
    if let Some(ref intent) = mem.last_intent {
        lines.push(format!("  {} {}",
            "Last:".dimmed(),
            intent.bright_white()
        ));
    }

    // Active intents from filesystem
    let intents = active_intents(core_root);
    if !intents.is_empty() {
        let intent_str = intents.iter()
            .map(|i| i.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("  {} {}",
            "Working on:".dimmed(),
            intent_str.bright_cyan()
        ));
    }

    // New commits since last session (skip on first ever session)
    let new = mem.new_commits();
    if new > 0 && mem.last_commit_count > 0 {
        lines.push(format!("  {} {} {} {}",
            "Since last session:".dimmed(),
            new.to_string().bright_green(),
            "new commit".dimmed(),
            if new == 1 { "" } else { "s" }
        ));
    }

    if lines.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    out
}

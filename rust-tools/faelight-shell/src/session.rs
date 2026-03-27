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
    #[allow(dead_code)]
    pub last_intent: Option<String>,
    pub last_session_ts: Option<i64>,
    pub last_dir: Option<String>,
}

impl SessionMemory {
    pub fn load(core_root: &str) -> Option<Self> {
        let db_path = std::path::Path::new(core_root).join("runtime/state.db");
        let conn = rusqlite::Connection::open(&db_path).ok()?;

        // Ensure session_state table exists
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS session_state (
                key TEXT PRIMARY KEY,
                value TEXT
            )
        ",
        )
        .ok()?;

        // Read last session data
        let last_commits: u64 = conn
            .query_row(
                "SELECT value FROM session_state WHERE key='last_commit_count'",
                [],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let last_intent: Option<String> = conn
            .query_row(
                "SELECT value FROM session_state WHERE key='last_intent'",
                [],
                |r| r.get(0),
            )
            .ok();

        let last_ts: Option<i64> = conn
            .query_row(
                "SELECT value FROM session_state WHERE key='last_session_ts'",
                [],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .and_then(|v| v.parse().ok());

        let last_dir: Option<String> = conn
            .query_row(
                "SELECT value FROM session_state WHERE key='last_dir'",
                [],
                |r| r.get(0),
            )
            .ok();

        // Read current commit count
        let commits_path = std::path::Path::new("/etc/faelight/COMMITS");
        let current_commits: u64 = std::fs::read_to_string(commits_path)
            .unwrap_or_default()
            .trim()
            .parse()
            .unwrap_or(0);

        Some(SessionMemory {
            last_commit_count: last_commits,
            current_commit_count: current_commits,
            last_intent,
            last_session_ts: last_ts,
            last_dir,
        })
    }

    pub fn save(core_root: &str, current_intent: Option<&str>) {
        let db_path = std::path::Path::new(core_root).join("runtime/state.db");
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let _ = conn.execute_batch(
                "
                CREATE TABLE IF NOT EXISTS session_state (
                    key TEXT PRIMARY KEY,
                    value TEXT
                )
            ",
            );

            let commits = std::fs::read_to_string("/etc/faelight/COMMITS")
                .unwrap_or_default()
                .trim()
                .to_string();
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
                rusqlite::params![ts],
            );
            // Save current directory
            if let Ok(cwd) = std::env::current_dir() {
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO session_state (key, value) VALUES ('last_dir', ?1)",
                    rusqlite::params![cwd.to_string_lossy().to_string()],
                );
            }
            if let Some(intent) = current_intent {
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO session_state (key, value) VALUES ('last_intent', ?1)",
                    rusqlite::params![intent],
                );
            }
        }
    }

    pub fn days_since(&self) -> Option<u64> {
        let ts = self.last_session_ts?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs() as i64;
        Some(((now - ts) / 86400) as u64)
    }

    pub fn hours_since(&self) -> Option<u64> {
        let ts = self.last_session_ts?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs() as i64;
        Some(((now - ts) / 3600) as u64)
    }

    pub fn new_commits(&self) -> u64 {
        self.current_commit_count
            .saturating_sub(self.last_commit_count)
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
            if !name.ends_with(".md") {
                continue;
            }
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

// ── Shell Personality Modes — Pillar 2 ───────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum ShellMode {
    Recovery,  // health < 95% — calm, methodical
    Streak,    // 5+ commits since last session — encouraging
    Idle,      // 2+ days away — gentle reorientation
    Milestone, // 0 in-progress intents — something just completed
    Focused,   // normal, active work session
}

pub fn detect_mode(mem: &SessionMemory, core_root: &str, active_count: usize) -> ShellMode {
    // Read health from events table (same as db.health_score())
    let health: u32 = {
        let db_path = std::path::Path::new(core_root).join("runtime/state.db");
        rusqlite::Connection::open(&db_path)
            .ok()
            .and_then(|conn| {
                conn.query_row(
                "SELECT payload FROM events WHERE domain='doctor' ORDER BY timestamp DESC LIMIT 1",
                [], |r| r.get::<_, String>(0)
            ).ok()
            })
            .and_then(|p| serde_json::from_str::<serde_json::Value>(&p).ok())
            .and_then(|v| v["detail"]["health"].as_i64())
            .unwrap_or(100) as u32
    };

    // Recovery — health degraded
    if health < 95 {
        return ShellMode::Recovery;
    }

    // Idle — long gap
    if let Some(days) = mem.days_since() {
        if days >= 2 {
            return ShellMode::Idle;
        }
    }

    // Streak — lots of commits
    if mem.new_commits() >= 5 && mem.last_commit_count > 0 {
        return ShellMode::Streak;
    }

    // Milestone — no active intents (something just completed)
    if active_count == 0 && mem.last_commit_count > 0 {
        return ShellMode::Milestone;
    }

    ShellMode::Focused
}

pub fn render(mem: &SessionMemory, core_root: &str) -> String {
    let mut lines: Vec<String> = vec![];
    let intents = active_intents(core_root);
    let mode = detect_mode(mem, core_root, intents.len());

    // Mode-aware welcome message
    let welcome = match &mode {
        ShellMode::Recovery => format!(
            "  {} {}",
            "⚕".yellow(),
            "Health advisory — let\'s see what needs attention.".yellow()
        ),
        ShellMode::Streak => format!(
            "  {} {}",
            "↺".bright_green(),
            "Strong streak. The forest is growing fast.".bright_green()
        ),
        ShellMode::Idle => {
            if let Some(days) = mem.days_since() {
                format!(
                    "  {} {} {}",
                    "↺".bright_cyan(),
                    format!("{} days away.", days).bright_white(),
                    "The forest waited patiently.".dimmed()
                )
            } else {
                format!("  {} {}", "↺".bright_cyan(), "Welcome back.".dimmed())
            }
        }
        ShellMode::Milestone => format!(
            "  {} {}",
            "✦".bright_yellow(),
            "All intents complete. What grows next?".bright_yellow()
        ),
        ShellMode::Focused => {
            if let Some(hours) = mem.hours_since() {
                if hours == 0 {
                    format!("  {} {}", "↺".bright_cyan(), "Welcome back.".dimmed())
                } else if hours < 2 {
                    format!(
                        "  {} {}",
                        "↺".bright_cyan(),
                        "Welcome back — picking up where you left off.".dimmed()
                    )
                } else if hours < 24 {
                    format!(
                        "  {} {}",
                        "↺".bright_cyan(),
                        format!("{} hours since last session.", hours).dimmed()
                    )
                } else {
                    format!("  {} {}", "↺".bright_cyan(), "Welcome back.".dimmed())
                }
            } else {
                format!("  {} {}", "↺".bright_cyan(), "Welcome back.".dimmed())
            }
        }
    };
    lines.push(welcome);

    // Active intents
    if !intents.is_empty() {
        let intent_str = intents
            .iter()
            .map(|i| i.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "  {} {}",
            "Working on:".dimmed(),
            intent_str.bright_cyan()
        ));
    }

    // New commits since last session
    let new = mem.new_commits();
    if new > 0 && mem.last_commit_count > 0 {
        lines.push(format!(
            "  {} {} {}",
            "Since last session:".dimmed(),
            new.to_string().bright_green(),
            if new == 1 {
                "new commit".dimmed()
            } else {
                "new commits".dimmed()
            }
        ));
    }

    // Pillar 3 — momentum detection
    let momentum = detect_momentum(core_root);
    if let Some(mom_str) = render_momentum(&momentum) {
        lines.push(mom_str);
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
// ── Pillar 3 — Momentum Detection ────────────────────────────────────────────

pub struct Momentum {
    pub feat_commits_today: usize,
    pub total_commits_today: usize,
    pub total_commits_week: usize,
}

pub fn detect_momentum(core_root: &str) -> Momentum {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let week_ago = (chrono::Local::now() - chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();

    // Use git log to count commits
    let today_log = std::process::Command::new("git")
        .args([
            "-C",
            core_root,
            "log",
            "--oneline",
            &format!("--since={} 00:00:00", today),
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let week_log = std::process::Command::new("git")
        .args([
            "-C",
            core_root,
            "log",
            "--oneline",
            &format!("--since={} 00:00:00", week_ago),
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    let total_today = today_log.lines().count();
    let feat_today = today_log
        .lines()
        .filter(|l| l.contains("feat:") || l.contains("feat("))
        .count();
    let total_week = week_log.lines().count();

    Momentum {
        feat_commits_today: feat_today,
        total_commits_today: total_today,
        total_commits_week: total_week,
    }
}

pub fn render_momentum(m: &Momentum) -> Option<String> {
    use colored::*;
    // Only show if meaningful activity
    if m.total_commits_today == 0 {
        return None;
    }

    let mut parts: Vec<String> = vec![];

    if m.feat_commits_today >= 5 {
        parts.push(format!(
            "  {} {} feat commits today — {}",
            "🔥".normal(),
            m.feat_commits_today.to_string().bright_green().bold(),
            "strong build session.".dimmed()
        ));
    } else if m.feat_commits_today >= 2 {
        parts.push(format!(
            "  {} {} feat commits today.",
            "⚡".normal(),
            m.feat_commits_today.to_string().bright_green()
        ));
    }

    if m.total_commits_week >= 20 {
        parts.push(format!(
            "  {} {} commits this week — {}",
            "↗".bright_green(),
            m.total_commits_week.to_string().bright_green().bold(),
            "the forest is growing fast.".dimmed()
        ));
    } else if m.total_commits_week >= 10 {
        parts.push(format!(
            "  {} {} commits this week.",
            "↗".bright_cyan(),
            m.total_commits_week.to_string().bright_white()
        ));
    }

    if parts.is_empty() && m.total_commits_today > 0 {
        parts.push(format!(
            "  {} {} commit{} today.",
            "○".dimmed(),
            m.total_commits_today.to_string().bright_white(),
            if m.total_commits_today == 1 { "" } else { "s" }
        ));
    }

    if parts.is_empty() {
        return None;
    }
    Some(parts.join("\n"))
}

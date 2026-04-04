// faelight-shell — prompt and status line
// render_line    — single-line readline prompt (no emoji, Tab completion safe)
// render_context — two-line forest context printed BEFORE the input line
// status_line    — pretty status printed after clear or on welcome

use crate::db::ForestDb;
use colored::*;

// ── Shared helpers ─────────────────────────────────────────────────────────

fn cwd_str(max_len: usize) -> String {
    let cwd = std::env::current_dir()
        .map(|p| {
            let home = std::env::var("HOME").unwrap_or_default();
            let path = p.to_string_lossy().to_string();
            if path.starts_with(&home) {
                format!("~{}", &path[home.len()..])
            } else {
                path
            }
        })
        .unwrap_or_else(|_| "?".to_string());
    if cwd.len() > max_len {
        let parts: Vec<&str> = cwd.split('/').collect();
        if parts.len() > 2 {
            format!("~/{}", parts.last().copied().unwrap_or(""))
        } else {
            cwd
        }
    } else {
        cwd
    }
}

fn health_str(health: i64) -> colored::ColoredString {
    if health >= 95 {
        format!("{}%", health).bright_green()
    } else if health >= 80 {
        format!("{}%", health).yellow()
    } else {
        format!("{}%", health).bright_red()
    }
}

fn git_info() -> Option<(String, bool)> {
    // Branch — read .git/HEAD directly, no subprocess
    let cwd = std::env::current_dir().ok()?;
    let mut dir = cwd.as_path();
    let git_root = loop {
        let git_head = dir.join(".git/HEAD");
        if git_head.exists() {
            break dir.to_path_buf();
        }
        dir = dir.parent()?;
    };
    let head = std::fs::read_to_string(git_root.join(".git/HEAD")).ok()?;
    let branch = head
        .trim()
        .strip_prefix("ref: refs/heads/")
        .unwrap_or("HEAD")
        .to_string();

    // Dirty check — git status --porcelain, 8ms on this repo
    let dirty = std::process::Command::new("git")
        .args(["-C", &git_root.to_string_lossy(), "status", "--porcelain"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    Some((branch, dirty))
}

fn active_intent(db: &ForestDb) -> Option<String> {
    db.conn
        .query_row(
            "SELECT value FROM shell_state WHERE key='focus_intent'",
            [],
            |r| r.get::<_, String>(0),
        )
        .ok()
}

fn commits_today(db: &ForestDb) -> i64 {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    db.conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE domain='git' AND action='commit' \
         AND datetime(timestamp, 'unixepoch', 'localtime') LIKE ?1",
            rusqlite::params![format!("{}%", today)],
            |r| r.get(0),
        )
        .unwrap_or(0)
}

// ── Phase 17 — Prompt v2 Context Lines ────────────────────────────────────
// Printed BEFORE readline — avoids rustyline cursor issues with emoji/wide chars

pub struct PromptContext {
    pub last_duration_ms: Option<u64>,
    pub last_exit_code: Option<i32>,
    pub job_count: usize,
}

pub fn render_context(db: &ForestDb, ctx: &PromptContext) {
    let theme = db.get_theme();
    if theme == "minimal" {
        // Minimal: just path, no health/git
        let cwd = cwd_str(40);
        println!("  {}", cwd.bright_cyan());
        return;
    }
    // All other themes use full context below
    let _ = ctx; // suppress unused warning for now
    let is_jarvis = theme == "jarvis";
    let cwd = cwd_str(35);
    let health = db.health_score().unwrap_or(95);
    let git = git_info();

    // ── Line 1: System state ─────────────────────────────────────────────
    let mut line1 = format!("  {}", cwd.bright_cyan().bold());

    if let Some((ref b, dirty)) = git {
        let symbol = if dirty { "*" } else { "" };
        line1.push_str(&format!(
            " {} {}{}",
            "(".dimmed().to_string(),
            b.bright_yellow().to_string(),
            format!("{})", symbol).dimmed().to_string()
        ));
    }

    // Exit code — only show if non-zero
    if let Some(code) = ctx.last_exit_code {
        if code != 0 {
            line1.push_str(&format!(" {}", format!("[✗ {}]", code).bright_red()));
        }
    }

    // Job count
    if ctx.job_count > 0 {
        line1.push_str(&format!(
            " {}",
            format!(
                "[{} job{}]",
                ctx.job_count,
                if ctx.job_count == 1 { "" } else { "s" }
            )
            .yellow()
        ));
    }

    // Execution time — only show if > 100ms
    if let Some(ms) = ctx.last_duration_ms {
        if ms >= 2000 {
            line1.push_str(&format!(
                " {}",
                format!("[{:.1}s]", ms as f64 / 1000.0).dimmed()
            ));
        } else if ms >= 100 {
            line1.push_str(&format!(" {}", format!("[{}ms]", ms).dimmed()));
        }
    }

    // ── Line 2: Forest context ────────────────────────────────────────────
    let h_str = health_str(health);
    let intent = active_intent(db);
    let today_commits = commits_today(db);

    let mut parts: Vec<String> = vec![h_str.to_string()];

    if let Some(ref i) = intent {
        parts.push(i.bright_cyan().to_string());
    }

    if today_commits > 0 {
        parts.push(
            format!("{} today", today_commits.to_string().bright_white())
                .dimmed()
                .to_string(),
        );
    }
    // Jarvis theme — add prediction insight inline
    if is_jarvis {
        // Read next predicted intent
        let home = std::env::var("HOME").unwrap_or_default();
        let next_intent = std::fs::read_dir(format!("{}/0-core/intents/future", home))
            .ok()
            .and_then(|entries| {
                let mut in_progress: Vec<String> = entries
                    .flatten()
                    .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                    .filter_map(|e| {
                        let content = std::fs::read_to_string(e.path()).ok()?;
                        if !content.contains("status: in-progress") { return None; }
                        let id = e.file_name()
                            .to_string_lossy()
                            .split('-')
                            .next()
                            .unwrap_or("?")
                            .to_string();
                        Some(format!("INT-{}", id))
                    })
                    .collect();
                in_progress.sort();
                in_progress.first().cloned()
            });

        // Read health trend
        let trend_hint = {
            let cache = std::fs::read_to_string(
                format!("{}/.cache/faelight/health-status", home)
            ).unwrap_or_else(|_| "100".to_string());
            let h: u32 = cache.trim().parse().unwrap_or(100);
            if h >= 100 { "peak".bright_green().to_string() }
            else if h >= 95 { "stable".dimmed().to_string() }
            else { "advisory".yellow().to_string() }
        };

        let jarvis_hint = match next_intent {
            Some(id) => format!("▸ {} · {}", id.bright_cyan(), trend_hint),
            None => format!("▸ {}", trend_hint),
        };
        parts.push(jarvis_hint);
    }

    let line2 = format!(
        "  {} {}",
        "→".dimmed(),
        parts.join(&" · ".dimmed().to_string())
    );

    println!("{}", line1);
    println!("{}", line2);
}

// ── readline prompt — no emoji, ANSI wrapped, Tab completion safe ──────────
// Emoji and wide chars break rustyline cursor position silently.

pub fn render_line(db: &ForestDb) -> String {
    let theme = db.get_theme();
    let raw = match theme.as_str() {
        "minimal" => format!("  {} ", "❯".bright_green().bold()),
        "classic" => {
            let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
            let host = std::fs::read_to_string("/etc/hostname")
                .unwrap_or_else(|_| "host".to_string());
            let host = host.trim();
            let cwd = cwd_str(30);
            format!("  {}@{} {} $ ", user.dimmed(), host.dimmed(), cwd.bright_cyan())
        }
        _ => format!("  {} ", "fsh ❯".bright_green().bold()), // forest + jarvis use same prompt line
    };
    rl_wrap(&raw)
}

// ── status line — printed after clear, shown in welcome ───────────────────

#[allow(dead_code)]
pub fn status_line(db: &ForestDb) -> String {
    let h = health_str(db.health_score().unwrap_or(95));
    let cwd = cwd_str(30);
    format!(
        "\n  {} {}  {}  {}\n",
        "🌲".normal(),
        cwd.bright_cyan().bold(),
        h,
        "forest".dimmed(),
    )
}

// ── ANSI wrap — required for rustyline cursor accuracy ────────────────────

pub fn rl_wrap(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            out.push('\x01');
            out.push('\x1b');
            while let Some(&nc) = chars.peek() {
                out.push(nc);
                chars.next();
                if nc == 'm' {
                    break;
                }
            }
            out.push('\x02');
        } else {
            out.push(c);
        }
    }
    out
}

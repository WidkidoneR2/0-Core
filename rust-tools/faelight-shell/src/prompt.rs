// faelight-shell — prompt and status line
// render_line  — single-line readline prompt (no emoji, Tab completion safe)
// status_line  — pretty status printed after clear or on welcome

use crate::db::ForestDb;
use colored::*;

// ── Shared helper — build cwd and health strings ──────────────────────────────

fn cwd_str(max_len: usize) -> String {
    let cwd = std::env::current_dir()
        .map(|p| {
            let home = std::env::var("HOME").unwrap_or_default();
            let path = p.to_string_lossy().to_string();
            if path.starts_with(&home) {
                format!("~{}", &path[home.len()..])
            } else { path }
        })
        .unwrap_or_else(|_| "?".to_string());
    if cwd.len() > max_len {
        let parts: Vec<&str> = cwd.split('/').collect();
        if parts.len() > 2 {
            format!("~/{}", parts.last().copied().unwrap_or(""))
        } else { cwd }
    } else { cwd }
}

fn health_str(health: i64) -> String {
    if health >= 95 {
        format!("{}%", health).bright_green().to_string()
    } else if health >= 80 {
        format!("{}%", health).yellow().to_string()
    } else {
        format!("{}%", health).bright_red().to_string()
    }
}

// ── readline prompt — no emoji, ANSI wrapped, Tab completion safe ─────────────
// Emoji and wide chars break rustyline cursor position silently.
// The 🌲 lives in status_line instead.

pub fn render_line(db: &ForestDb) -> String {
    let h = health_str(db.health_score().unwrap_or(95));
    let cwd = cwd_str(20);
    let raw = format!("{} {} > ", cwd.bright_cyan(), h);
    rl_wrap(&raw)
}

// ── status line — printed after clear, shown in welcome ───────────────────────
// This is where the forest personality lives.
// Called by: c/clear command, print_welcome

pub fn status_line(db: &ForestDb) -> String {
    let h = health_str(db.health_score().unwrap_or(95));
    let cwd = cwd_str(30);
    format!("\n  {} {}  {}  {}\n",
        "🌲".normal(),
        cwd.bright_cyan().bold(),
        h,
        "forest".dimmed(),
    )
}

// ── ANSI wrap — required for rustyline cursor accuracy ────────────────────────

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
                if nc == 'm' { break; }
            }
            out.push('\x02');
        } else {
            out.push(c);
        }
    }
    out
}

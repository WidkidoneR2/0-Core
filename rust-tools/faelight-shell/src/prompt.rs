#![allow(clippy::all)]
// faelight-shell -- prompt and status line
// render_line    -- single-line readline prompt (no emoji, Tab completion safe)
// render_context -- two-line forest context printed BEFORE the input line
// status_line    -- pretty status printed after clear or on welcome
// INT-033        -- neon candy truecolor semantic colors

use crate::db::ForestDb;

// OSC 133 shell integration sequences (INT-296)
pub const OSC133_PROMPT_START: &str = "\x1b]133;A\x1b\\"; // prompt start
pub const OSC133_PROMPT_END: &str   = "\x1b]133;B\x1b\\"; // command input start
pub const OSC133_OUTPUT_START: &str = "\x1b]133;C\x1b\\"; // output start
pub fn osc133_command_end(exit_code: i32) -> String {
    format!("\x1b]133;D;{}\x1b\\", exit_code)
}

// ── Truecolor helpers ───────────────────────────────────────────────────────
fn fc(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}
fn fc_bold(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[1m\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}
fn fc_dim(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[2m\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}
fn fc_bold_rl(r: u8, g: u8, b: u8, text: &str) -> String {
    // rl_wrap-safe bold truecolor for rustyline prompt
    format!("\x01\x1b[1m\x1b[38;2;{};{};{}m\x02{}\x01\x1b[0m\x02", r, g, b, text)
}
fn fc_rl(r: u8, g: u8, b: u8, text: &str) -> String {
    // rl_wrap-safe truecolor for rustyline prompt
    format!("\x01\x1b[38;2;{};{};{}m\x02{}\x01\x1b[0m\x02", r, g, b, text)
}

// ── Semantic color tokens (INT-033) ─────────────────────────────────────────
// Health
const C_HEALTH_PEAK:     (u8,u8,u8) = (57,  255, 20);
const C_HEALTH_ADVISORY: (u8,u8,u8) = (255, 200, 50);
const C_HEALTH_CRITICAL: (u8,u8,u8) = (255, 80,  80);
// Prompt
const C_CWD:             (u8,u8,u8) = (50,  220, 255);
const C_PROMPT_OK:       (u8,u8,u8) = (57,  255, 20);
const C_PROMPT_FAIL:     (u8,u8,u8) = (255, 80,  80);
const C_INTENT:          (u8,u8,u8) = (180, 130, 255);
const C_BRANCH_CLEAN:    (u8,u8,u8) = (57,  255, 20);
const C_BRANCH_DIRTY:    (u8,u8,u8) = (255, 200, 50);
const C_DIMMED:          (u8,u8,u8) = (120, 140, 130);

// ── Shared helpers ──────────────────────────────────────────────────────────

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

fn health_str(health: i64) -> String {
    let text = format!("{}%", health);
    if health >= 95 {
        fc_bold(C_HEALTH_PEAK.0, C_HEALTH_PEAK.1, C_HEALTH_PEAK.2, &text)
    } else if health >= 80 {
        fc_bold(C_HEALTH_ADVISORY.0, C_HEALTH_ADVISORY.1, C_HEALTH_ADVISORY.2, &text)
    } else {
        fc_bold(C_HEALTH_CRITICAL.0, C_HEALTH_CRITICAL.1, C_HEALTH_CRITICAL.2, &text)
    }
}

fn git_info() -> Option<(String, bool)> {
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

// ── Phase 17 -- Prompt v2 Context Lines ─────────────────────────────────────

pub struct PromptContext {
    pub last_duration_ms: Option<u64>,
    pub last_exit_code: Option<i32>,
    pub job_count: usize,
}

pub fn render_context(db: &ForestDb, ctx: &PromptContext) {
    let theme = db.get_theme();
    if theme == "minimal" {
        let cwd = cwd_str(40);
        println!("  {}", fc(C_CWD.0, C_CWD.1, C_CWD.2, &cwd));
        return;
    }
    let _ = ctx;
    let is_friday = theme == "friday";
    let cwd = cwd_str(35);
    let health = db.health_score().unwrap_or(95);
    let git = git_info();

    // ── Line 1: path + git + exit + jobs + time ──────────────────────────
    let mut line1 = format!("  {}", fc_bold(C_CWD.0, C_CWD.1, C_CWD.2, &cwd));

    if let Some((ref b, dirty)) = git {
        let symbol = if dirty { "*" } else { "" };
        let (r,g,bl) = if dirty { C_BRANCH_DIRTY } else { C_BRANCH_CLEAN };
        line1.push_str(&format!(
            " {} {}{}",
            fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, "("),
            fc_bold(r, g, bl, &format!("{}{}", b, symbol)),
            fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, ")")
        ));
    }

    if let Some(code) = ctx.last_exit_code {
        if code != 0 {
            line1.push_str(&format!(
                " {}",
                fc_bold(C_PROMPT_FAIL.0, C_PROMPT_FAIL.1, C_PROMPT_FAIL.2,
                    &format!("[✗ {}]", code))
            ));
        }
    }

    if ctx.job_count > 0 {
        line1.push_str(&format!(
            " {}",
            fc(C_HEALTH_ADVISORY.0, C_HEALTH_ADVISORY.1, C_HEALTH_ADVISORY.2,
                &format!("[{} job{}]", ctx.job_count,
                    if ctx.job_count == 1 { "" } else { "s" }))
        ));
    }

    if let Some(ms) = ctx.last_duration_ms {
        if ms >= 2000 {
            line1.push_str(&format!(
                " {}",
                fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2,
                    &format!("[{:.1}s]", ms as f64 / 1000.0))
            ));
        } else if ms >= 100 {
            line1.push_str(&format!(
                " {}",
                fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, &format!("[{}ms]", ms))
            ));
        }
    }

    // ── Line 2: health · intent · commits ───────────────────────────────
    let h_str = health_str(health);
    let intent = active_intent(db);
    let today_commits = commits_today(db);

    let mut parts: Vec<String> = vec![h_str];

    if let Some(ref i) = intent {
        parts.push(fc_bold(C_INTENT.0, C_INTENT.1, C_INTENT.2, i));
    }

    if today_commits > 0 {
        parts.push(fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2,
            &format!("{} today", today_commits)));
    }

    if is_friday {
        let home = std::env::var("HOME").unwrap_or_default();
        let next_intent = std::fs::read_dir(format!("{}/0-core/intents/future", home))
            .ok()
            .and_then(|entries| {
                let mut in_progress: Vec<String> = entries
                    .flatten()
                    .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
                    .filter_map(|e| {
                        let content = std::fs::read_to_string(e.path()).ok()?;
                        if !content.contains("status: in-progress") {
                            return None;
                        }
                        let id = e
                            .file_name()
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

        let trend_hint = {
            let cache = std::fs::read_to_string(
                format!("{}/.cache/faelight/health-status", home))
                .unwrap_or_else(|_| "100".to_string());
            let h: u32 = cache.trim().parse().unwrap_or(100);
            if h >= 100 {
                fc_bold(C_HEALTH_PEAK.0, C_HEALTH_PEAK.1, C_HEALTH_PEAK.2, "peak")
            } else if h >= 95 {
                fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, "stable")
            } else {
                fc(C_HEALTH_ADVISORY.0, C_HEALTH_ADVISORY.1, C_HEALTH_ADVISORY.2, "advisory")
            }
        };

        let friday_hint = match next_intent {
            Some(id) => format!("▸ {} · {}",
                fc(C_INTENT.0, C_INTENT.1, C_INTENT.2, &id), trend_hint),
            None => format!("▸ {}", trend_hint),
        };
        parts.push(friday_hint);

        let db_path = format!("{}/0-core/runtime/state.db", home);
        let has_friday_msg = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .ok()
        .and_then(|c| {
            c.query_row(
                "SELECT COUNT(*) FROM friday_daemon_messages WHERE read = 0",
                [],
                |r| r.get::<_, i64>(0),
            )
            .ok()
        })
        .unwrap_or(0) > 0;
        if has_friday_msg {
            parts.push("🌲".to_string());
        }
    }

    let sep = fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, " · ");
    let line2 = format!(
        "  {} {}",
        fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, "→"),
        parts.join(&sep)
    );

    println!("{}", line1);
    println!("{}", line2);
}

// ── readline prompt -- no emoji, ANSI wrapped, Tab completion safe ───────────

pub fn render_line(db: &ForestDb, _last_exit: Option<i32>) -> String {
    let theme = db.get_theme();
    let cache_file =
        std::env::var("HOME").unwrap_or_default() + "/.cache/faelight/last-exit-status";
    let last_status = std::fs::read_to_string(&cache_file).unwrap_or_default();
    let last_status = last_status.trim();
    let caret = if last_status == "failure" {
        fc_bold_rl(C_PROMPT_FAIL.0, C_PROMPT_FAIL.1, C_PROMPT_FAIL.2, "❯")
    } else {
        fc_bold_rl(C_PROMPT_OK.0, C_PROMPT_OK.1, C_PROMPT_OK.2, "❯")
    };
    let nix_indicator = if std::env::var("IN_NIX_SHELL").is_ok() {
        format!("{} ", fc_rl(50, 220, 255, "❄"))
    } else if std::env::var("DIRENV_DIR").is_ok() {
        format!("{} ", fc_rl(80, 140, 255, "❄"))
    } else {
        String::new()
    };
    let raw = match theme.as_str() {
        "minimal" => format!("  {}{} ", nix_indicator, caret),
        "classic" => {
            let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
            let host =
                std::fs::read_to_string("/etc/hostname").unwrap_or_else(|_| "host".to_string());
            let host = host.trim();
            let cwd = cwd_str(30);
            format!(
                "  {}{}@{} {} $ ",
                nix_indicator,
                fc_rl(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, &user),
                fc_rl(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, host),
                fc_rl(C_CWD.0, C_CWD.1, C_CWD.2, &cwd)
            )
        }
        _ => format!("  {}{}{}  ",
            nix_indicator,
            fc_bold_rl(C_PROMPT_OK.0, C_PROMPT_OK.1, C_PROMPT_OK.2, "fsh"),
            caret),
    };
    raw
}

// ── status line ─────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn status_line(db: &ForestDb) -> String {
    let h = health_str(db.health_score().unwrap_or(95));
    let cwd = cwd_str(30);
    format!(
        "\n  {} {}  {}  {}\n",
        "🌲",
        fc_bold(C_CWD.0, C_CWD.1, C_CWD.2, &cwd),
        h,
        fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, "forest"),
    )
}



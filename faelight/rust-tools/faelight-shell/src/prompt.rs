#![allow(clippy::all)]
// faelight-shell -- prompt and status line
// render_line    -- single-line readline prompt (no emoji, Tab completion safe)
// render_context -- two-line forest context printed BEFORE the input line
// status_line    -- pretty status printed after clear or on welcome
// INT-033        -- neon candy truecolor semantic colors

use crate::db::ForestDb;

// OSC 133 shell integration sequences (INT-296)
pub const OSC133_PROMPT_START: &str = "\x1b]133;A\x1b\\"; // prompt start
pub const OSC133_PROMPT_END: &str = "\x1b]133;B\x1b\\"; // command input start
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
    format!(
        "\x01\x1b[1m\x1b[38;2;{};{};{}m\x02{}\x01\x1b[0m\x02",
        r, g, b, text
    )
}
fn fc_rl(r: u8, g: u8, b: u8, text: &str) -> String {
    // rl_wrap-safe truecolor for rustyline prompt
    format!(
        "\x01\x1b[38;2;{};{};{}m\x02{}\x01\x1b[0m\x02",
        r, g, b, text
    )
}

// ── Semantic color tokens (INT-033) ─────────────────────────────────────────
// ── Candy-neon family (INT-103): launcher/logout palette ──
// Meaning: lime=structure/peak, aqua=location, lavender=intent/focus,
//          gold=caution, rose=attention/fail, near-green=quiet separators.
// Health
const C_HEALTH_PEAK: (u8, u8, u8) = (176, 246, 42); // electric lime (neon70)
const C_HEALTH_ADVISORY: (u8, u8, u8) = (252, 213, 78); // gold (neon70)
const C_HEALTH_CRITICAL: (u8, u8, u8) = (255, 95, 135); // hot rose (neon70)
                                                        // Prompt
const C_CWD: (u8, u8, u8) = (40, 242, 216); // aqua (neon70)
const C_PROMPT_OK: (u8, u8, u8) = (176, 246, 42); // electric lime (neon70)
const C_PROMPT_FAIL: (u8, u8, u8) = (255, 130, 168); // rose (neon70)
const C_INTENT: (u8, u8, u8) = (186, 156, 255); // lavender (neon70)
const C_BRANCH_CLEAN: (u8, u8, u8) = (158, 224, 78); // soft lime (neon70)
const C_BRANCH_DIRTY: (u8, u8, u8) = (252, 213, 78); // gold (neon70)
const C_DIMMED: (u8, u8, u8) = (90, 110, 95); // near-green quiet
                                              // Directory-context accents (INT-103): path color tells you WHAT KIND of place
const C_DIR_FOREST: (u8, u8, u8) = (176, 246, 42); // forest core: lime (neon70)
const C_DIR_RUST: (u8, u8, u8) = (255, 138, 44); // Rust territory: orange (neon70)
const C_DIR_NIX: (u8, u8, u8) = (74, 196, 255); // Nix domain: ice-blue (neon70)
const C_DIR_INTENTS: (u8, u8, u8) = (186, 156, 255); // intents/: lavender (neon70)
const C_DIR_DOTFILES: (u8, u8, u8) = (255, 130, 168); // dotfiles/: rose (neon70)
const C_DIR_HOME: (u8, u8, u8) = (40, 242, 216); // elsewhere in ~: aqua (neon70)
const C_DIR_SYSTEM: (u8, u8, u8) = (252, 213, 78); // system: gold (neon70)
const C_DIR_ROOT: (u8, u8, u8) = (200, 90, 110); // outside home entirely: dim rose
const C_DEVSHELL: (u8, u8, u8) = (40, 242, 216); // devshell: aqua (neon70)

// ── Powerline (INT-103) ─────────────────────────────────────────────────────
// Nerd Font glyphs (JetBrainsMono NF confirmed present)
const PL_ARROW: &str = "\u{e0b0}"; // right-filled arrow (segment flow)
const PL_FOLDER: &str = "\u{e5ff}"; // folder
const PL_GIT: &str = "\u{e0a0}"; // git branch
const PL_NIX: &str = "\u{f313}"; // nix snowflake-ish
const DARK: (u8, u8, u8) = (12, 20, 15); // near-black-green text on candy bg

struct Seg {
    text: String,
    bg: (u8, u8, u8),
    fg: (u8, u8, u8),
}

// Render a run of segments with flowing powerline arrows between them.
// Each segment paints its bg; the arrow after it is that bg as FG on the next bg.
fn powerline(segs: &[Seg]) -> String {
    let mut out = String::new();
    for (i, seg) in segs.iter().enumerate() {
        let (br, bg, bb) = seg.bg;
        let (fr, fg_, fb) = seg.fg;
        // segment body: bold fg text on bg, padded
        out.push_str(&format!(
            "\x1b[48;2;{};{};{}m\x1b[1m\x1b[38;2;{};{};{}m {} \x1b[0m",
            br, bg, bb, fr, fg_, fb, seg.text
        ));
        // arrow into next segment (or off the end)
        if let Some(next) = segs.get(i + 1) {
            let (nr, ng, nb) = next.bg;
            out.push_str(&format!(
                "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m{}\x1b[0m",
                nr, ng, nb, br, bg, bb, PL_ARROW
            ));
        } else {
            // tail arrow on terminal default bg
            out.push_str(&format!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                br, bg, bb, PL_ARROW
            ));
        }
    }
    out
}

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

fn cwd_color() -> (u8, u8, u8) {
    use std::path::Path;
    let cwdp = std::env::current_dir().unwrap_or_default();
    let cwd = cwdp.to_string_lossy().to_string();
    let home = std::env::var("HOME").unwrap_or_default();
    let forest = format!("{}/0-core", home);
    let in_forest = cwd == forest || cwd.starts_with(&format!("{}/", forest));

    // Zone precedence: forest sub-zones win first (so the workspace root reads
    // as forest, not "rust" just because a workspace Cargo.toml sits there).
    if in_forest {
        if cwd.contains("/rust-tools") {
            return C_DIR_RUST; // Rust territory inside the forest
        }
        if cwd.contains("/intents") {
            return C_DIR_INTENTS; // the ledger / thought-space
        }
        if cwd.contains("/home/dotfiles") {
            return C_DIR_DOTFILES; // personal config
        }
        if cwd.contains("/nix") {
            return C_DIR_NIX; // the OS domain
        }
        return C_DIR_FOREST; // forest core (root, faelight, etc.)
    }
    // Outside the forest: marker-file detection.
    if Path::new("Cargo.toml").exists() {
        return C_DIR_RUST; // a Rust project anywhere
    }
    if Path::new("flake.nix").exists() {
        return C_DIR_NIX; // a Nix project anywhere
    }
    // System dirs -- careful
    if cwd.starts_with("/etc")
        || cwd.starts_with("/nix")
        || cwd.starts_with("/usr")
        || cwd.starts_with("/var")
    {
        return C_DIR_SYSTEM;
    }
    // Elsewhere in home vs outside
    if !home.is_empty() && cwd.starts_with(&home) {
        C_DIR_HOME
    } else {
        C_DIR_ROOT
    }
}
fn health_str(health: i64) -> String {
    let text = format!("{}%", health);
    if health >= 95 {
        fc_bold(C_HEALTH_PEAK.0, C_HEALTH_PEAK.1, C_HEALTH_PEAK.2, &text)
    } else if health >= 80 {
        fc_bold(
            C_HEALTH_ADVISORY.0,
            C_HEALTH_ADVISORY.1,
            C_HEALTH_ADVISORY.2,
            &text,
        )
    } else {
        fc_bold(
            C_HEALTH_CRITICAL.0,
            C_HEALTH_CRITICAL.1,
            C_HEALTH_CRITICAL.2,
            &text,
        )
    }
}

fn git_info() -> Option<(String, bool, bool)> {
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
    let porcelain = std::process::Command::new("git")
        .args(["-C", &git_root.to_string_lossy(), "status", "--porcelain"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let dirty = !porcelain.is_empty();
    let flake_dirty = porcelain.lines().any(|l| {
        let path = l.get(3..).unwrap_or("");
        path == "flake.nix"
            || path == "flake.lock"
            || path.ends_with("/flake.nix")
            || path.ends_with("/flake.lock")
    });
    Some((branch, dirty, flake_dirty))
}

fn flake_info() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let mut dir = cwd.as_path();
    let flake_root = loop {
        if dir.join("flake.nix").exists() {
            break dir;
        }
        dir = dir.parent()?;
    };
    flake_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
}

fn system_drift() -> Option<bool> {
    let home = std::env::var("HOME").ok()?;
    let head = std::fs::read_to_string(format!("{}/0-core/.git/HEAD", home)).ok()?;
    let head = head.trim();
    let current = if let Some(r) = head.strip_prefix("ref: ") {
        std::fs::read_to_string(format!("{}/0-core/.git/{}", home, r))
            .ok()?
            .trim()
            .to_string()
    } else {
        head.to_string()
    };
    let built =
        std::fs::read_to_string(format!("{}/.cache/faelight/last-system-rev", home)).ok()?;
    Some(current.as_str() != built.trim())
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

    // ── Line 1: candy powerline (INT-103) -- directory / repo / devshell ──
    let mut segs: Vec<Seg> = Vec::new();
    let dir_bg = cwd_color();
    segs.push(Seg {
        text: format!("{} {}", PL_FOLDER, cwd),
        bg: dir_bg,
        fg: DARK,
    });
    if let Some((ref b, dirty, flake_dirty)) = git {
        let star = if dirty { "*" } else { "" };
        let bg = if dirty {
            C_BRANCH_DIRTY
        } else {
            C_BRANCH_CLEAN
        };
        let fx = if flake_dirty {
            format!(" {}", PL_NIX)
        } else {
            String::new()
        };
        segs.push(Seg {
            text: format!("{} {}{}{}", PL_GIT, b, star, fx),
            bg,
            fg: DARK,
        });
    }
    let devshell = std::env::var("name")
        .ok()
        .map(|n| n.strip_suffix("-env").unwrap_or(n.as_str()).to_string())
        .filter(|n| !n.is_empty() && n != "0-core");
    if let Some(d) = devshell {
        segs.push(Seg {
            text: format!("{} {}", PL_NIX, d),
            bg: C_DEVSHELL,
            fg: DARK,
        });
    }
    let mut line1 = format!("  {}", powerline(&segs));
    // INT-153: debug-build marker -- prefix. cfg!(debug_assertions) is
    // compile-time true in debug, false in release; both blocks vanish in release.
    if cfg!(debug_assertions) {
        line1 = format!("\u{1f527}{}", line1);
    }

    if let Some(code) = ctx.last_exit_code {
        if code != 0 {
            line1.push_str(&format!(
                " {}",
                fc_bold(
                    C_PROMPT_FAIL.0,
                    C_PROMPT_FAIL.1,
                    C_PROMPT_FAIL.2,
                    &format!("[✗ {}]", code)
                )
            ));
        }
    }

    if ctx.job_count > 0 {
        line1.push_str(&format!(
            " {}",
            fc(
                C_HEALTH_ADVISORY.0,
                C_HEALTH_ADVISORY.1,
                C_HEALTH_ADVISORY.2,
                &format!(
                    "[{} job{}]",
                    ctx.job_count,
                    if ctx.job_count == 1 { "" } else { "s" }
                )
            )
        ));
    }

    if let Some(ms) = ctx.last_duration_ms {
        if ms >= 2000 {
            line1.push_str(&format!(
                " {}",
                fc_dim(
                    C_DIMMED.0,
                    C_DIMMED.1,
                    C_DIMMED.2,
                    &format!("[{:.1}s]", ms as f64 / 1000.0)
                )
            ));
        } else if ms >= 100 {
            line1.push_str(&format!(
                " {}",
                fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, &format!("[{}ms]", ms))
            ));
        }
    }

    // INT-153: debug-build marker -- suffix tag at far right of line 1.
    if cfg!(debug_assertions) {
        line1.push_str(&format!(
            " {}",
            fc_bold(
                C_PROMPT_FAIL.0,
                C_PROMPT_FAIL.1,
                C_PROMPT_FAIL.2,
                "[DEBUG BUILD]"
            )
        ));
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
        parts.push(fc_dim(
            C_DIMMED.0,
            C_DIMMED.1,
            C_DIMMED.2,
            &format!("{} today", today_commits),
        ));
    }

    if let Some(true) = system_drift() {
        parts.push(fc_bold(
            C_HEALTH_ADVISORY.0,
            C_HEALTH_ADVISORY.1,
            C_HEALTH_ADVISORY.2,
            "⇡ rebuild",
        ));
    }

    if is_friday {
        let next_intent = std::fs::read_dir(faelight_core::paths::intents_dir().join("future"))
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
            // Absent health used to fall back to 100 -- twice -- and the branch
            // below then printed "peak". A machine that had never run the doctor
            // claimed peak health on every render. None now yields no hint at all.
            match faelight_core::paths::read_health().map(u32::from) {
                // "unknown" IS a trend hint. An early return here would have
                // exited render_context entirely and dropped the intent hint
                // with it -- invisible except on a machine that has never run
                // the doctor, which is the machine nobody tests on.
                None => fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, "unknown"),
                Some(h) if h >= 100 => {
                    fc_bold(C_HEALTH_PEAK.0, C_HEALTH_PEAK.1, C_HEALTH_PEAK.2, "peak")
                }
                Some(h) if h >= 95 => fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, "stable"),
                Some(_) => fc(
                    C_HEALTH_ADVISORY.0,
                    C_HEALTH_ADVISORY.1,
                    C_HEALTH_ADVISORY.2,
                    "advisory",
                ),
            }
        };

        let friday_hint = match next_intent {
            Some(id) => format!(
                "▸ {} · {}",
                fc(C_INTENT.0, C_INTENT.1, C_INTENT.2, &id),
                trend_hint
            ),
            None => format!("▸ {}", trend_hint),
        };
        parts.push(friday_hint);

        let db_path = faelight_core::paths::state_db();
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
        .unwrap_or(0)
            > 0;
        if has_friday_msg {
            parts.push("🌲".to_string());
        }
    }

    let sep = fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, " · ");
    let _line2 = format!(
        "  {} {}",
        fc_dim(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, "→"),
        parts.join(&sep)
    );

    println!("{}", line1);
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
    let devshell_name = std::env::var("name")
        .ok()
        .map(|n| n.strip_suffix("-env").unwrap_or(n.as_str()).to_string())
        .filter(|n| !n.is_empty());
    let flake = flake_info();
    let label = match (&flake, &devshell_name) {
        (Some(f), Some(d)) => Some(format!("{}·{}", f, d)),
        (Some(f), None) => Some(f.clone()),
        (None, Some(d)) => Some(d.clone()),
        (None, None) => None,
    };
    // ⚠️ TWO DIFFERENT FACTS, AND THEY WERE PRINTING THE SAME GLYPH.
    //   IN_NIX_SHELL  -- a Nix environment IS LOADED. The snowflake is earned.
    //   DIRENV_DIR    -- direnv KNOWS ABOUT a directory with an .envrc. That is all it means:
    //                    direnv sets it on discovery, before and regardless of whether the file
    //                    was allowed or the environment loaded. On a machine with no Nix and an
    //                    .envrc reading `use flake`, this was set while nothing had loaded --
    //                    so the prompt claimed an environment that did not exist.
    // A snowflake means Nix. direnv is not Nix, and knowing about a file is not loading it.
    let nix_indicator = if std::env::var("IN_NIX_SHELL").is_ok() {
        let _ = &label;
        format!("{} ", fc_rl(54, 224, 208, "❄"))
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
        _ => format!(
            "  {}{}{}  ",
            nix_indicator,
            fc_bold_rl(C_PROMPT_OK.0, C_PROMPT_OK.1, C_PROMPT_OK.2, "fsh"),
            caret
        ),
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

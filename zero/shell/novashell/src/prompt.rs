#![allow(clippy::all)]
// NovaShell -- prompt and status line
// render_line    -- single-line readline prompt (no emoji, Tab completion safe)
// render_context -- the context line printed BEFORE the input line
// INT-033        -- neon candy truecolor semantic colors

use crate::db::StateDb;

// OSC 133 shell integration sequences (INT-296)
pub const OSC133_PROMPT_START: &str = "\x1b]133;A\x1b\\"; // prompt start
pub const OSC133_PROMPT_END: &str = "\x1b]133;B\x1b\\"; // command input start
pub const OSC133_OUTPUT_START: &str = "\x1b]133;C\x1b\\"; // output start
pub fn osc133_command_end(exit_code: i32) -> String {
    format!("\x1b]133;D;{}\x1b\\", exit_code)
}

/// INT-277 G6 (F-0099): OSC 7 tells the terminal the shell's cwd, so a new window or split can
/// open there (foot's spawn-terminal, ghostty splits). Every byte outside the URI unreserved set
/// and `/` is percent-encoded, so a space or a UTF-8 byte cannot end the sequence or change the path.
pub fn osc7_cwd(host: &str, cwd: &str) -> String {
    let mut path = String::with_capacity(cwd.len());
    for b in cwd.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'.' | b'_' | b'~' => {
                path.push(b as char)
            }
            _ => path.push_str(&format!("%{:02X}", b)),
        }
    }
    format!("\x1b]7;file://{}{}\x1b\\", host, path)
}

/// INT-277 G6 (F-0100): OSC 2 sets the window title to `user@host:cwd`, the form Omarchy's bash
/// gives, with a path under `home` shown as `~/...`. Control characters are dropped so a
/// directory name cannot inject a second escape sequence into the terminal.
pub fn osc2_title(user: &str, host: &str, cwd: &str, home: &str) -> String {
    let shown = match cwd.strip_prefix(home) {
        Some("") if !home.is_empty() => "~".to_string(),
        Some(rest) if !home.is_empty() && rest.starts_with('/') => format!("~{rest}"),
        _ => cwd.to_string(),
    };
    let title: String = format!("{user}@{host}:{shown}")
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    format!("\x1b]2;{}\x1b\\", title)
}

fn hostname() -> String {
    let mut buf = [0u8; 256];
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
    if rc != 0 {
        return String::new();
    }
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).to_string()
}

/// What the terminal reads at each prompt: the title and the cwd. Nothing for TERM unset or
/// `dumb`, which cannot use them, and nothing when the cwd cannot be read (a deleted directory).
pub fn terminal_marks() -> String {
    let term = std::env::var("TERM").unwrap_or_default();
    if term.is_empty() || term == "dumb" {
        return String::new();
    }
    let cwd = match std::env::current_dir() {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => return String::new(),
    };
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "user".to_string());
    let home = std::env::var("HOME").unwrap_or_default();
    let host = hostname();
    format!(
        "{}{}",
        osc2_title(&user, &host, &cwd, &home),
        osc7_cwd(&host, &cwd)
    )
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
const C_HEALTH_ADVISORY: (u8, u8, u8) = (252, 213, 78); // gold (neon70)
                                                        // Prompt
const C_CWD: (u8, u8, u8) = (40, 242, 216); // aqua (neon70)
const C_PROMPT_OK: (u8, u8, u8) = (176, 246, 42); // electric lime (neon70)
const C_PROMPT_FAIL: (u8, u8, u8) = (255, 130, 168); // rose (neon70)
const C_BRANCH_CLEAN: (u8, u8, u8) = (158, 224, 78); // soft lime (neon70)
const C_BRANCH_DIRTY: (u8, u8, u8) = (252, 213, 78); // gold (neon70)
const C_DIMMED: (u8, u8, u8) = (90, 110, 95); // near-green quiet
                                              // Directory-context accents (INT-103): path color tells you WHAT KIND of place
const C_DIR_REPO: (u8, u8, u8) = (176, 246, 42); // repo core: lime (neon70)
const C_DIR_RUST: (u8, u8, u8) = (255, 138, 44); // Rust territory: orange (neon70)
const C_DIR_INTENTS: (u8, u8, u8) = (186, 156, 255); // intents/: lavender (neon70)
const C_DIR_DOTFILES: (u8, u8, u8) = (255, 130, 168); // dotfiles/: rose (neon70)
const C_DIR_HOME: (u8, u8, u8) = (40, 242, 216); // elsewhere in ~: aqua (neon70)
const C_DIR_SYSTEM: (u8, u8, u8) = (252, 213, 78); // system: gold (neon70)
const C_DIR_ROOT: (u8, u8, u8) = (200, 90, 110); // outside home entirely: dim rose

// ── Powerline (INT-103) ─────────────────────────────────────────────────────
// Nerd Font glyphs (JetBrainsMono NF confirmed present)
const PL_ARROW: &str = "\u{e0b0}"; // right-filled arrow (segment flow)
const PL_FOLDER: &str = "\u{e5ff}"; // folder
const PL_GIT: &str = "\u{e0a0}"; // git branch
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
    let repo_dir = format!("{}/0-core", home);
    let in_repo = cwd == repo_dir || cwd.starts_with(&format!("{}/", repo_dir));

    // Zone precedence: repo sub-zones win first (so the workspace root reads
    // as the repo, not "rust" just because a workspace Cargo.toml sits there).
    if in_repo {
        let in_crates = zero_core::paths::crate_parent_markers()
            .iter()
            .any(|m| cwd.contains(&format!("/{}", m.trim_end_matches('/'))));
        if in_crates {
            return C_DIR_RUST; // Rust territory inside the repo
        }
        if cwd.contains("/intents") {
            return C_DIR_INTENTS; // the ledger / thought-space
        }
        if cwd.contains("/home/dotfiles") {
            return C_DIR_DOTFILES; // personal config
        }
        return C_DIR_REPO; // repo core
    }
    // Outside the repo: marker-file detection.
    if Path::new("Cargo.toml").exists() {
        return C_DIR_RUST; // a Rust project anywhere
    }
    // System dirs -- careful
    if cwd.starts_with("/etc") || cwd.starts_with("/usr") || cwd.starts_with("/var") {
        return C_DIR_SYSTEM;
    }
    // Elsewhere in home vs outside
    if !home.is_empty() && cwd.starts_with(&home) {
        C_DIR_HOME
    } else {
        C_DIR_ROOT
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
    let porcelain = std::process::Command::new("git")
        .args(["-C", &git_root.to_string_lossy(), "status", "--porcelain"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let dirty = !porcelain.is_empty();
    Some((branch, dirty))
}

// ── Phase 17 -- Prompt context line ─────────────────────────────────────────

pub struct PromptContext {
    pub last_duration_ms: Option<u64>,
    pub last_exit_code: Option<i32>,
    pub job_count: usize,
}

pub fn render_context(db: &StateDb, ctx: &PromptContext) {
    let theme = db.get_theme();
    if theme == "minimal" {
        let cwd = cwd_str(40);
        println!("  {}", fc(C_CWD.0, C_CWD.1, C_CWD.2, &cwd));
        return;
    }
    let _ = ctx;
    let cwd = cwd_str(35);
    let git = git_info();

    // ── Line 1: candy powerline (INT-103) -- directory / repo / devshell ──
    let mut segs: Vec<Seg> = Vec::new();
    let dir_bg = cwd_color();
    segs.push(Seg {
        text: format!("{} {}", PL_FOLDER, cwd),
        bg: dir_bg,
        fg: DARK,
    });
    if let Some((ref b, dirty)) = git {
        let star = if dirty { "*" } else { "" };
        let bg = if dirty {
            C_BRANCH_DIRTY
        } else {
            C_BRANCH_CLEAN
        };
        segs.push(Seg {
            text: format!("{} {}{}", PL_GIT, b, star),
            bg,
            fg: DARK,
        });
    }
    // ⚠️ A SECOND DEVSHELL SEGMENT WAS HERE, removed 2026-09-20 (INT-255). It read the same
    // $name variable, stripped the same -env suffix, and drew the same snowflake as the block
    // already removed from the other theme -- TWO COPIES OF ONE IDEA, each unaware of the
    // other. Neither could ever fire: $name is empty on this machine.
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
    // a001b8e8 stopped printing the second line ("health/rebuild line removed
    // (lives on bar + d)") and left the work that built it: the health read,
    // the focus-intent query, today's commit count, and on the friday theme a
    // ledger scan plus a second open of state.db. Nothing printed that line,
    // and nothing else reads those results.
    println!("{}", line1);
}

// ── readline prompt -- no emoji, ANSI wrapped, Tab completion safe ───────────

pub fn render_line(db: &StateDb, _last_exit: Option<i32>) -> String {
    let theme = db.get_theme();
    // INT-247 Layer 3a: one owner for the path.
    let last_status =
        std::fs::read_to_string(zero_core::paths::last_exit_status_file()).unwrap_or_default();
    let last_status = last_status.trim();
    let caret = if last_status == "failure" {
        fc_bold_rl(C_PROMPT_FAIL.0, C_PROMPT_FAIL.1, C_PROMPT_FAIL.2, "❯")
    } else {
        fc_bold_rl(C_PROMPT_OK.0, C_PROMPT_OK.1, C_PROMPT_OK.2, "❯")
    };
    // ⭐ KNOWING ABOUT A FILE IS NOT LOADING IT -- true of any environment manager. DIRENV_DIR
    // is set when direnv discovers a directory with an .envrc, before and regardless of whether
    // the file was allowed or loaded. A prompt glyph keyed on it claims an environment that may
    // not exist. DIRENV_DIR is still set here today; nothing in the prompt keys on it.
    let raw = match theme.as_str() {
        "minimal" => format!("  {} ", caret),
        "classic" => {
            let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
            let host =
                std::fs::read_to_string("/etc/hostname").unwrap_or_else(|_| "host".to_string());
            let host = host.trim();
            let cwd = cwd_str(30);
            format!(
                "  {}@{} {} $ ",
                fc_rl(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, &user),
                fc_rl(C_DIMMED.0, C_DIMMED.1, C_DIMMED.2, host),
                fc_rl(C_CWD.0, C_CWD.1, C_CWD.2, &cwd)
            )
        }
        _ => format!(
            "  {}{}  ",
            fc_bold_rl(C_PROMPT_OK.0, C_PROMPT_OK.1, C_PROMPT_OK.2, "nsh"),
            caret
        ),
    };
    raw
}

// INT-230 G4: status_line DELETED. It was #[allow(dead_code)] with no
// caller, and it defaulted health to 95 -- asserting near-perfect health on
// a machine that had never run the doctor. Dead code carrying a lie is worse
// than dead code: nobody sees it fail, so nobody fixes it, and whoever wires
// it up later inherits the defect silently.

#[cfg(test)]
mod line1_tests {
    use super::*;
    use std::io::{Read, Write};
    use std::os::unix::io::FromRawFd;

    // Recorded from render_context before the unprinted second line was removed.
    // Fixture: a clean git repo named work, branch main, HOME set to its parent,
    // in-memory state db. The debug-build marker is not part of these literals;
    // shown() adds it the same way render_context does.
    const QUIET: &str = "  \u{1b}[48;2;40;242;216m\u{1b}[1m\u{1b}[38;2;12;20;15m \u{e5ff} ~/work \u{1b}[0m\u{1b}[48;2;158;224;78m\u{1b}[38;2;40;242;216m\u{e0b0}\u{1b}[0m\u{1b}[48;2;158;224;78m\u{1b}[1m\u{1b}[38;2;12;20;15m \u{e0a0} main \u{1b}[0m\u{1b}[38;2;158;224;78m\u{e0b0}\u{1b}[0m";
    const BUSY: &str = "  \u{1b}[48;2;40;242;216m\u{1b}[1m\u{1b}[38;2;12;20;15m \u{e5ff} ~/work \u{1b}[0m\u{1b}[48;2;158;224;78m\u{1b}[38;2;40;242;216m\u{e0b0}\u{1b}[0m\u{1b}[48;2;158;224;78m\u{1b}[1m\u{1b}[38;2;12;20;15m \u{e0a0} main \u{1b}[0m\u{1b}[38;2;158;224;78m\u{e0b0}\u{1b}[0m \u{1b}[1m\u{1b}[38;2;255;130;168m[\u{2717} 7]\u{1b}[0m \u{1b}[38;2;252;213;78m[2 jobs]\u{1b}[0m \u{1b}[2m\u{1b}[38;2;90;110;95m[2.5s]\u{1b}[0m";
    const MILLIS: &str = "  \u{1b}[48;2;40;242;216m\u{1b}[1m\u{1b}[38;2;12;20;15m \u{e5ff} ~/work \u{1b}[0m\u{1b}[48;2;158;224;78m\u{1b}[38;2;40;242;216m\u{e0b0}\u{1b}[0m\u{1b}[48;2;158;224;78m\u{1b}[1m\u{1b}[38;2;12;20;15m \u{e0a0} main \u{1b}[0m\u{1b}[38;2;158;224;78m\u{e0b0}\u{1b}[0m \u{1b}[2m\u{1b}[38;2;90;110;95m[150ms]\u{1b}[0m";
    const FAST: &str = "  \u{1b}[48;2;40;242;216m\u{1b}[1m\u{1b}[38;2;12;20;15m \u{e5ff} ~/work \u{1b}[0m\u{1b}[48;2;158;224;78m\u{1b}[38;2;40;242;216m\u{e0b0}\u{1b}[0m\u{1b}[48;2;158;224;78m\u{1b}[1m\u{1b}[38;2;12;20;15m \u{e0a0} main \u{1b}[0m\u{1b}[38;2;158;224;78m\u{e0b0}\u{1b}[0m \u{1b}[38;2;252;213;78m[1 job]\u{1b}[0m";
    const MINIMAL: &str = "  \u{1b}[38;2;40;242;216m~/work\u{1b}[0m";

    fn shown(release: &str) -> String {
        if cfg!(debug_assertions) {
            format!(
                "\u{1f527}{release} {}",
                fc_bold(
                    C_PROMPT_FAIL.0,
                    C_PROMPT_FAIL.1,
                    C_PROMPT_FAIL.2,
                    "[DEBUG BUILD]"
                )
            )
        } else {
            release.to_string()
        }
    }

    struct StdoutGuard {
        saved: i32,
        write_end: i32,
    }

    impl Drop for StdoutGuard {
        fn drop(&mut self) {
            let _ = std::io::stdout().flush();
            unsafe {
                libc::dup2(self.saved, libc::STDOUT_FILENO);
                libc::close(self.saved);
                libc::close(self.write_end);
            }
        }
    }

    fn capture_stdout(f: impl FnOnce()) -> String {
        let _ = std::io::stdout().flush();
        let mut fds = [0i32; 2];
        assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0, "pipe");
        let saved = unsafe { libc::dup(libc::STDOUT_FILENO) };
        assert!(saved >= 0, "dup");
        assert_eq!(
            unsafe { libc::dup2(fds[1], libc::STDOUT_FILENO) },
            libc::STDOUT_FILENO,
            "dup2"
        );
        let guard = StdoutGuard {
            saved,
            write_end: fds[1],
        };
        f();
        drop(guard);
        let mut file = unsafe { std::fs::File::from_raw_fd(fds[0]) };
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        buf
    }

    fn git(repo: &std::path::Path, args: &[&str]) {
        let template = repo
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("empty-template");
        std::fs::create_dir_all(&template).unwrap();
        let st = std::process::Command::new("git")
            .args(args)
            .current_dir(repo)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_TEMPLATE_DIR", &template)
            .status()
            .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
        assert!(st.success(), "git {args:?} -> {st}");
    }

    fn render_cases() -> String {
        let base = std::env::temp_dir().join(format!("nsh-prompt-line1-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let home = base.join("home");
        let repo = home.join("work");
        std::fs::create_dir_all(&repo).unwrap();
        git(&repo, &["init", "-b", "main"]);
        git(
            &repo,
            &[
                "-c",
                "user.email=t@example.com",
                "-c",
                "user.name=Test",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--allow-empty",
                "-m",
                "init",
            ],
        );
        unsafe {
            std::env::set_var("HOME", &home);
            std::env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
            std::env::set_var("GIT_CONFIG_NOSYSTEM", "1");
        }
        std::env::set_current_dir(&repo).unwrap();

        let db = crate::db::StateDb::open_in_memory().unwrap();
        let quiet = PromptContext {
            last_duration_ms: None,
            last_exit_code: Some(0),
            job_count: 0,
        };
        let busy = PromptContext {
            last_duration_ms: Some(2500),
            last_exit_code: Some(7),
            job_count: 2,
        };
        let ms = PromptContext {
            last_duration_ms: Some(150),
            last_exit_code: None,
            job_count: 0,
        };
        let fast = PromptContext {
            last_duration_ms: Some(99),
            last_exit_code: None,
            job_count: 1,
        };

        let mut out = String::new();
        for (name, theme, ctx) in [
            ("zero-quiet", "zero", &quiet),
            ("friday-quiet", "friday", &quiet),
            ("classic-quiet", "classic", &quiet),
            ("other-quiet", "other", &quiet),
            ("minimal-quiet", "minimal", &quiet),
            ("zero-busy", "zero", &busy),
            ("friday-busy", "friday", &busy),
            ("zero-ms", "zero", &ms),
            ("zero-fast", "zero", &fast),
        ] {
            db.set_theme(theme).unwrap();
            let printed = capture_stdout(|| render_context(&db, ctx));
            out.push_str(&format!("== {name} ==\n"));
            out.push_str(&printed);
            if !printed.ends_with('\n') {
                out.push('\n');
            }
        }

        db.conn
            .execute(
                "INSERT OR REPLACE INTO shell_state (key, value) VALUES ('focus_intent', 'INT-9')",
                [],
            )
            .unwrap();
        db.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS events (
                    id INTEGER PRIMARY KEY,
                    domain TEXT,
                    action TEXT,
                    payload TEXT,
                    timestamp INTEGER
                );
                INSERT INTO events (domain, action, payload, timestamp)
                VALUES ('git', 'commit', 'x', strftime('%s','now'));",
            )
            .unwrap();
        for (name, theme) in [("zero-loaded", "zero"), ("friday-loaded", "friday")] {
            db.set_theme(theme).unwrap();
            let printed = capture_stdout(|| render_context(&db, &quiet));
            out.push_str(&format!("== {name} ==\n"));
            out.push_str(&printed);
            if !printed.ends_with('\n') {
                out.push('\n');
            }
        }
        let _ = std::fs::remove_dir_all(&base);
        out
    }

    fn section<'a>(report: &'a str, name: &str) -> &'a str {
        let marker = format!("== {name} ==\n");
        let start = report
            .find(&marker)
            .unwrap_or_else(|| panic!("missing section {name}"));
        let rest = &report[start + marker.len()..];
        let end = rest.find("\n== ").map(|i| i + 1).unwrap_or(rest.len());
        &rest[..end]
    }

    fn expect(name: &str, line: &str) -> (String, String) {
        (name.to_string(), format!("{line}\n"))
    }

    #[test]
    fn prompt_line1_matches_the_recorded_output() {
        if std::env::var_os("NSH_PROMPT_LINE1_CHILD").is_some() {
            let path = std::env::var("NSH_PROMPT_LINE1_OUT").expect("out path");
            std::fs::write(&path, render_cases()).unwrap();
            return;
        }
        let out = std::env::temp_dir().join(format!(
            "nsh-prompt-line1-{}-{}.txt",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let exe = std::env::current_exe().unwrap();
        let status = std::process::Command::new(&exe)
            .args([
                "prompt::line1_tests::prompt_line1_matches_the_recorded_output",
                "--exact",
                "--test-threads=1",
                "--nocapture",
            ])
            .env("NSH_PROMPT_LINE1_CHILD", "1")
            .env("NSH_PROMPT_LINE1_OUT", &out)
            .status()
            .unwrap();
        assert!(status.success(), "child exit {status}");
        let got = std::fs::read_to_string(&out).unwrap();
        let _ = std::fs::remove_file(&out);
        let quiet = shown(QUIET);
        let expected = [
            expect("zero-quiet", &quiet),
            expect("friday-quiet", &quiet),
            expect("classic-quiet", &quiet),
            expect("other-quiet", &quiet),
            expect("minimal-quiet", MINIMAL),
            expect("zero-busy", &shown(BUSY)),
            expect("friday-busy", &shown(BUSY)),
            expect("zero-ms", &shown(MILLIS)),
            expect("zero-fast", &shown(FAST)),
            expect("zero-loaded", &quiet),
            expect("friday-loaded", &quiet),
        ];
        for (name, line) in &expected {
            assert_eq!(section(&got, name), line.as_str(), "{name}");
        }
    }
}

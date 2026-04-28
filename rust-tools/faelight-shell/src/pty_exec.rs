#![allow(dead_code)]

//! INT-249b/Path-3: PTY-based command execution with output scanning.
//!
//! Runs a shell command through a PTY, preserving colors and TTY-aware behavior
//! while letting fsh inspect the output stream. Used initially by the heredoc
//! handler to fire INT-249 leak warnings on standalone delimiter-shaped lines
//! that appear in command output.
//!
//! This module is the foundation for broader output-intelligence work
//! (INT-245 Pillar 5). New commands that need output scanning should call
//! `run_with_capture_and_scan` instead of spawning sh directly.

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{BufRead, BufReader, Write};

/// Run `cmd` via `sh -c` through a PTY. Print its output to fsh's stdout
/// while scanning each line for INT-249 heredoc leak patterns. Returns the
/// child's exit code, or 1 on PTY/spawn failure.
pub fn run_with_capture_and_scan(cmd: &str) -> i32 {
    let pty_system = native_pty_system();
    let pair = match pty_system.openpty(PtySize {
        rows: 24,
        cols: 120,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("  pty: openpty failed: {}", e);
            return 1;
        }
    };

    let mut child_cmd = CommandBuilder::new("sh");
    child_cmd.args(["-c", cmd]);
    if let Ok(cwd) = std::env::current_dir() {
        child_cmd.cwd(cwd);
    }

    let mut child = match pair.slave.spawn_command(child_cmd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  pty: spawn failed: {}", e);
            return 1;
        }
    };
    drop(pair.slave);

    let reader = match pair.master.try_clone_reader() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  pty: reader clone failed: {}", e);
            return 1;
        }
    };

    let mut buf = BufReader::new(reader);
    let mut line = Vec::new();
    let mut stdout = std::io::stdout().lock();

    loop {
        line.clear();
        match buf.read_until(b'\n', &mut line) {
            Ok(0) => break,
            Ok(_) => {
                let _ = stdout.write_all(&line);
                let _ = stdout.flush();

                // INT-249 leak detection: scan the line (stripped of ANSI)
                let s = String::from_utf8_lossy(&line);
                let stripped = strip_ansi(&s);
                let trimmed = stripped.trim();
                if is_heredoc_leak(trimmed) {
                    let _ = stdout.write_all(b"  \xe2\x9a\xa0 possible unclosed heredoc -- ");
                    let _ = write!(stdout, "{:?}", trimmed);
                    let _ = stdout.write_all(b" appeared as standalone output line\n");
                    let _ = stdout.flush();
                }
            }
            Err(_) => break,
        }
    }

    let status = child.wait().ok();
    drop(pair.master);

    status
        .and_then(|s| s.exit_code().try_into().ok())
        .unwrap_or(1)
}

/// Match standalone delimiter pattern: ^[A-Z_]{3,}EOF$ on a single line.
/// Excludes plain "EOF" by requiring at least 3 uppercase/underscore prefix chars
/// before the EOF suffix.
fn is_heredoc_leak(s: &str) -> bool {
    if !s.ends_with("EOF") || s.len() < 6 {
        return false;
    }
    let prefix = &s[..s.len() - 3];
    if prefix.len() < 3 {
        return false;
    }
    prefix.chars().all(|c| c.is_ascii_uppercase() || c == '_')
}

/// Strip ANSI escape sequences from a string (for clean leak-pattern matching).
/// Keeps the original string in the output stream — this is only for scanning.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // skip until we hit a letter (end of CSI) or run out
            for nc in chars.by_ref() {
                if nc.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

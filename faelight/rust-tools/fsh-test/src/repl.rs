//! REPL-driving test support -- INT-172.
//!
//! WHY THIS MODULE EXISTS. fsh-test's run_fsh() invokes `fsh -c`. Measured
//! 2026-07-17 against the same deployed binary on the same day:
//!
//!     fsh -c 'echo hello 2>/dev/null | grep -c hello'   -> 1       CORRECT
//!     the same line typed at the prompt                 -> hello   WRONG
//!
//! `fsh -c` never had the INT-172 redirect bug. Only the interactive REPL did.
//! So 83/83 green never meant "fsh works" -- it meant "the -c path works". It
//! said nothing about the shell we actually type into, and it could not have: no
//! test written against -c could ever fail on that bug.
//!
//! INT-171 gate 3 (2026-07-19) escalated this from one bug to a CLASS. All six
//! INT-143 regression bugs -- double-exec on redirect, typo-&&-leak, python3 flag
//! stripping, bash-script non-exec, env passthrough, inline-var scope -- are
//! INVISIBLE through `fsh -c`: run through /bin/sh, every one returns the correct
//! answer. Six bugs, all in the door the suite did not knock on. The finding is
//! not "a bug slipped through" -- it is "an entire category of fsh behaviour was
//! untested." The six repl_143_* tests below are the fix; INT-173 formalises the
//! rule (see The Rule in that intent): interactive behaviour is tested through the
//! REPL door; `-c` tests are for the `-c`/sh path, which is real (INT-190 boot
//! depends on it) but is NOT fsh's dispatch.
//!
//! A pty is how you make fsh believe a human is there. fsh asks isatty(); with a
//! plain pipe it answers "no terminal" and takes another path, and the REPL --
//! rustyline, the highlighter, the prompt -- never exists at all. This module
//! opens a real pty so the tests knock on the door we walk through.

use nix::pty::openpty;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

fn fsh_bin() -> String {
    std::env::var("FSH_BIN")
        .unwrap_or_else(|_| "/run/current-system/sw/bin/faelight-shell".to_string())
}

/// Strip ANSI CSI/OSC sequences and control bytes; map CR to LF.
fn strip_ansi(raw: &[u8]) -> String {
    let mut out: Vec<u8> = Vec::with_capacity(raw.len());
    let mut i = 0usize;
    while i < raw.len() {
        let b = raw[i];
        if b == 0x1b {
            if i + 1 < raw.len() && raw[i + 1] == b'[' {
                i += 2;
                while i < raw.len() && !(0x40..=0x7e).contains(&raw[i]) {
                    i += 1;
                }
                i += 1;
                continue;
            }
            if i + 1 < raw.len() && raw[i + 1] == b']' {
                i += 2;
                while i < raw.len() {
                    if raw[i] == 0x07 {
                        i += 1;
                        break;
                    }
                    if raw[i] == 0x1b && i + 1 < raw.len() && raw[i + 1] == b'\\' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            i += 2;
            continue;
        }
        if b == b'\r' {
            out.push(b'\n');
        } else if b == b'\n' || b == b'\t' || b >= 0x20 {
            out.push(b);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn find_last(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    (0..=hay.len() - needle.len())
        .rev()
        .find(|&i| &hay[i..i + needle.len()] == needle)
}

fn drain(rx: &mpsc::Receiver<Vec<u8>>, quiet: Duration) -> Vec<u8> {
    let mut acc = Vec::new();
    while let Ok(chunk) = rx.recv_timeout(quiet) {
        acc.extend_from_slice(&chunk);
    }
    acc
}

/// Type one line into a real interactive fsh over a pty; return the lines it printed.
///
/// This is NOT `fsh -c`. That distinction is the entire reason this exists.
pub fn run_repl(cmd: &str) -> Result<Vec<String>, String> {
    run_repl_lines(&[cmd])
}

/// Type SEVERAL lines into ONE fsh session, in order, and return what the LAST one printed.
///
/// ★ WHY THIS EXISTS: a file outlives the shell, so a background job's EFFECT can be verified in a
/// second session -- but REPL STATE cannot. A job table, an exported variable and the working
/// directory all die with the process, so any test about them needs two commands in ONE session.
/// That gap is why `jobs` could break for six generations with nothing noticing: the regression
/// lived in exactly the class the harness could not express.
///
/// ⚠️ ONLY THE LAST COMMAND'S OUTPUT COMES BACK. The reader keeps everything after the LAST
/// bracketed-paste-off marker, so earlier lines are setup, not assertions -- a test asserting on
/// an earlier line would silently see nothing.
pub fn run_repl_lines(cmds: &[&str]) -> Result<Vec<String>, String> {
    let pty = openpty(None, None).map_err(|e| format!("openpty: {}", e))?;

    let s_in = pty.slave.try_clone().map_err(|e| e.to_string())?;
    let s_out = pty.slave.try_clone().map_err(|e| e.to_string())?;
    let s_err = pty.slave.try_clone().map_err(|e| e.to_string())?;

    let mut child = Command::new(fsh_bin())
        .current_dir("/tmp")
        .stdin(Stdio::from(s_in))
        .stdout(Stdio::from(s_out))
        .stderr(Stdio::from(s_err))
        .spawn()
        .map_err(|e| format!("spawn fsh: {}", e))?;
    drop(pty.slave);

    let mut master = std::fs::File::from(pty.master);
    let mut reader = master.try_clone().map_err(|e| e.to_string())?;
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                // master read returns EIO once every slave fd is closed
                Err(_) => break,
            }
        }
    });

    // fsh spawns `nixos-rebuild list-generations --json` in its banner path on
    // every start (main.rs:3795), so the first prompt is not instant. Let it
    // settle, then throw the banner away.
    std::thread::sleep(Duration::from_millis(2500));
    let _ = drain(&rx, Duration::from_millis(400));

    // Each line is submitted and given time to run before the next -- a settle per line, not
    // one at the end, because a later command may depend on an earlier one having finished.
    for cmd in cmds {
        master
            .write_all(cmd.as_bytes())
            .map_err(|e| e.to_string())?;
        master.write_all(b"\n").map_err(|e| e.to_string())?;
        std::thread::sleep(Duration::from_millis(1200));
    }
    let raw = drain(&rx, Duration::from_millis(500));

    let _ = master.write_all(b"exit\n");
    std::thread::sleep(Duration::from_millis(300));
    let _ = child.kill();
    let _ = child.wait();

    // Everything after the last bracketed-paste-off marker is the command's real
    // output. Before it are ~40 rounds of per-keystroke repaint: the highlighter
    // redraws the whole line on every character typed.
    let tail = match find_last(&raw, b"\x1b[?2004l") {
        Some(i) => &raw[i + 8..],
        None => &raw[..],
    };
    Ok(strip_ansi(tail)
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

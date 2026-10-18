//! REPL-driving test support -- INT-172.
//!
//! WHY THIS MODULE EXISTS. nsh-test's run_fsh() invokes `fsh -c`. Measured
//! 2026-07-17 against the same deployed binary on the same day:
//!
//! ```text
//! fsh -c 'echo hello 2>/dev/null | grep -c hello'   -> 1       CORRECT
//! the same line typed at the prompt                 -> hello   WRONG
//! ```
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
use std::time::{Duration, Instant};

/// THE SHELL UNDER TEST -- resolved in ONE place, and verified to exist.
///
/// The fallback used to be silent on BOTH doors, so a typo or a dropped NSH_BIN quietly
/// tested the DEPLOYED shell and the suite reported green. That is INT-110's lesson, and it
/// recurred on 2026-08-21: three tests stayed red through six rebuilds because the harness
/// was measuring a binary from before the fix. A default that hides a missing input is the
/// same disease as a check that cannot fail.
pub fn fsh_bin() -> String {
    match std::env::var("NSH_BIN") {
        Ok(p) if std::path::Path::new(&p).exists() => absolute_shell(&p),
        Ok(p) => {
            eprintln!("  NSH_BIN names a binary that does not exist: {}", p);
            eprintln!("  Refusing to fall back to the deployed shell -- that is how a stale binary passes.");
            std::process::exit(2);
        }
        // paths::bin_dir owns where a deployed binary lives, so the fallback follows it
        // instead of naming a location. Same accessor ship writes to.
        Err(_) => {
            // THE BINARY IS nsh. It had the old project name until 2026-09-01, and this fallback
            // kept pointing at the old name after the rename -- so the suite reported
            // "under test:" with the old binary name and passed 178 cases about a binary the
            // machine was no longer running. The same defect as the store path this
            // fallback was written to fix, arriving from the other side: then it could not
            // find its subject, now it finds the WRONG one confidently.
            let p = zero_core::paths::bin_dir().join("nsh");
            // BOTH ARMS REFUSE NOW. The Ok arm has always exited 2 when NSH_BIN names a
            // missing binary, with the right reasoning: refusing to fall back is how a
            // stale binary is kept from passing. The fallback arm did no such check, so
            // when its path died with the migration the suite spawned nothing 158 times
            // and reported a 2% shell instead of an absent one. One standard, both arms.
            if !p.exists() {
                eprintln!(
                    "  The shell under test is not where it should be: {}",
                    p.display()
                );
                eprintln!("  Nothing was tested. A missing subject is not a failing suite.");
                eprintln!();
                eprintln!("  Recovery");
                eprintln!("    - ship nsh                   install it, then re-run");
                eprintln!("    - NSH_BIN=<path> nsh-test    test a binary somewhere else");
                std::process::exit(2);
            }
            p.display().to_string()
        }
    }
}

/// INT-287 G8 (F-0034): the shell path every case spawns, made absolute against nsh-test's own
/// working directory. The REPL cases start nsh with current_dir /tmp, so a relative NSH_BIN that
/// exists from here names nothing there, and all of them failed with "No such file".
fn absolute_shell(p: &str) -> String {
    std::path::absolute(p)
        .map(|a| a.display().to_string())
        .unwrap_or_else(|_| p.to_string())
}

/// INT-287 G4 (F-0030): the zero-deadwood binary under test, from DEADWOOD_BIN or else the deployed
/// one at paths::bin_dir, the same two arms as fsh_bin. A binary that is not on disk is refused
/// with its path, never handed to PATH to resolve: a missing deadwood is not a passing check.
pub fn deadwood_bin() -> Result<String, String> {
    resolve_bin(
        "DEADWOOD_BIN",
        std::env::var("DEADWOOD_BIN").ok(),
        zero_core::paths::bin_dir().join("zero-deadwood"),
    )
}

/// The rule both arms keep: a named binary must exist, and so must the deployed fallback.
fn resolve_bin(
    var: &str,
    named: Option<String>,
    deployed: std::path::PathBuf,
) -> Result<String, String> {
    match named {
        Some(p) if std::path::Path::new(&p).exists() => Ok(p),
        Some(p) => Err(format!("{var} names a binary that does not exist: {p}")),
        None if deployed.exists() => Ok(deployed.display().to_string()),
        None => Err(format!(
            "{var} is not set and the deployed binary is not there: {}",
            deployed.display()
        )),
    }
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

/// First occurrence of `needle` at or after `from`. The forward twin of find_last, and the
/// second half of a bounded window: find_last picks which command, this picks where it ends.
fn find_from(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() || from > hay.len() - needle.len() {
        return None;
    }
    (from..=hay.len() - needle.len()).find(|&i| &hay[i..i + needle.len()] == needle)
}

/// Read until `needle` appears at or after `from`, or `limit` elapses.
///
/// THIS REPLACES A SLEEP WITH AN OBSERVATION. A sleep encodes a guess about how fast the machine
/// is; this waits for the thing the guess was approximating and returns the instant it arrives.
///
/// ⚠️ None means the marker never came. Callers MUST treat that as a failure: an empty capture
/// reads as a passing assertion about absence.
fn wait_for(
    rx: &mpsc::Receiver<Vec<u8>>,
    acc: &mut Vec<u8>,
    needle: &[u8],
    from: usize,
    limit: Duration,
) -> Option<usize> {
    let started = Instant::now();
    loop {
        if let Some(i) = find_from(acc, needle, from) {
            return Some(i);
        }
        let left = limit.checked_sub(started.elapsed())?;
        match rx.recv_timeout(left.min(Duration::from_millis(50))) {
            Ok(chunk) => acc.extend_from_slice(&chunk),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            // the shell closed the pty: one last look, then give up
            Err(mpsc::RecvTimeoutError::Disconnected) => return find_from(acc, needle, from),
        }
    }
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
/// ⚠️ ONLY THE LAST COMMAND'S OUTPUT COMES BACK. The capture is the window between the LAST
/// bracketed-paste-off marker and the `133;A` that follows it, so earlier lines are setup rather
/// than assertions -- a test asserting on an earlier line would silently see nothing.
pub fn run_repl_lines(cmds: &[&str]) -> Result<Vec<String>, String> {
    run_repl_lines_env(cmds, &[])
}

/// Same, with environment variables set on the shell being tested.
///
/// ★ WHY THIS EXISTS: every case ran SPINE-ROUTED because the spawn set no environment, so the
/// spine answered every test and the legacy path was never exercised. That is why
/// `repl_background_job_keeps_quoted_arguments` passed for months while legacy was mangling
/// quoted arguments -- the spine claims `sh -c "..." &` and handles the quoting correctly, so the
/// test never reached the code its name describes. A harness that can only knock on one door
/// cannot report on the other.
///
/// ⚠️ ADDITIVE ON PURPOSE. Roughly forty call sites use run_repl/run_repl_lines; changing their
/// signature would have churned all of them to express something almost none of them need.
pub fn run_repl_lines_env(cmds: &[&str], env: &[(&str, &str)]) -> Result<Vec<String>, String> {
    run_repl_lines_status(cmds, env).map(|(lines, _)| lines)
}

/// Same, and also the exit status of the LAST command.
///
/// ★ `133;D;<status>` is fsh telling the terminal what the command exited with, and it lands
/// inside the window the capture already computes -- so this is a search through bytes we are
/// holding, not new plumbing.
///
/// ⚠️ Option, not i32, and the distinction is the whole point. A command that takes the screen or
/// ends the session may emit no `133;D` at all, and returning a manufactured 0 there would make a
/// comparison look performed when it was not. Unknown must stay unknown.
/// A database path nothing else will use, under one directory this run owns.
///
/// The counter is process-local and the pid keeps concurrent runs apart. The directory is removed at
/// the end of main -- not by each case, because a case that fails would skip its own cleanup, which
/// is the trap this harness already records against itself.
pub fn case_db_path() -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = case_db_dir();
    let _ = std::fs::create_dir_all(&dir);
    format!("{}/case{}.db", dir, N.fetch_add(1, Ordering::SeqCst))
}

/// The one directory this run owns. Named from the pid so two runs never collide.
pub fn case_db_dir() -> String {
    format!("/tmp/nsh-test-{}", std::process::id())
}

/// How the physical lines of one construct reach fsh's line editor.
///
/// THIS IS A PROPERTY OF THE READLINE LAYER, NOT OF SHELL SYNTAX, and the difference was
/// measured rather than assumed. Both variants were wrong for the other class first.
///
///                     physical input delivery
///                              |
///              +---------------+---------------+
///              |                               |
///        Incremental                       Buffered
///        for / if / while                  quote / heredoc
///        wait per line                     write the whole buffer
///
/// Incremental: after `for i in alpha beta` fsh returns to a CONTINUATION READ and emits
/// ?2004h again. That marker is a handshake -- ready for the next line, not command
/// finished -- and each line must wait for it. Writing all three at once put them in the pty
/// before rustyline came back, so it read them as ONE buffered submission, ran the first line
/// alone, swallowed the rest, and sat at `... ` waiting for a body that was already gone.
///
/// Buffered: after `echo 'first` or `cat <<EOF` rustyline stays INSIDE one read_line and
/// emits no intermediate ?2004h at all. Waiting for one times out at 30 seconds. The whole
/// construct is written and a single ?2004l follows when it completes.
///
/// A future reader seeing multiline in a name should not assume every multi-line syntax
/// belongs to one path. The shell has two behaviours; so does this.
#[derive(Clone, Copy, PartialEq)]
pub enum InputDelivery {
    Incremental,
    Buffered,
    /// INT-289 G1 c: Incremental, with these line indexes HELD -- lines that open or continue a
    /// multi-line construct, so no command completes and no 133;D comes. A held line ends at its
    /// ready marker; every other line ends at its own 133;D and the ready marker after it. The
    /// case declares them; the runner never infers them from the prompt text.
    Held(&'static [usize]),
}

/// Submit several lines that form ONE continued construct, then capture its output.
///
/// `held` names the lines that open or continue the construct. They complete no command, so no
/// 133;D comes for them (INT-289 G1 c): a held line ends at its ready marker, the continuation
/// prompt, and every other line ends at its own 133;D and the ready marker after it. Every line is
/// still waited for before the next is sent: writing them together puts all of them in the pty
/// before the editor comes back for the next one, and it reads them as one submission.
///
/// WHY THE CONSTRUCT MUST BE LAST. The capture window runs from the LAST `?2004l` to the
/// following `133;A`, so it holds the final command's output and nothing else. A trailing
/// `echo alive` would therefore be the thing captured and the construct's own output would be
/// lost. Put the construct last and assert on it directly.
///
/// fsh continues correctly -- measured 2026-09-01: two lines and a close came back as one
/// string spanning both, and the session counted the right number of commands. Until this
/// existed the suite could not express that, or any for/while/if typed across lines.
pub fn run_repl_multiline(
    cmds: &[&str],
    held: &'static [usize],
    env: &[(&str, &str)],
) -> Result<(Vec<String>, Option<i32>), String> {
    run_session(cmds, env, None, InputDelivery::Held(held))
}

/// Submit a construct whose lines rustyline consumes in ONE read: a quote held open, a
/// heredoc. No intermediate prompt comes back, so no per-line wait is possible.
pub fn run_repl_buffered(
    cmds: &[&str],
    env: &[(&str, &str)],
) -> Result<(Vec<String>, Option<i32>), String> {
    run_session(cmds, env, None, InputDelivery::Buffered)
}

pub fn run_repl_lines_status(
    cmds: &[&str],
    env: &[(&str, &str)],
) -> Result<(Vec<String>, Option<i32>), String> {
    run_session(cmds, env, None, InputDelivery::Incremental)
}

/// Submit ONE line expected to hit fsh's safety guard, and answer the guard's prompt.
///
/// THE GUARD IS NOT AT A PROMPT WHEN IT ASKS. safety_guard::challenge_gate leaves rustyline and
/// blocks on a raw std::io::stdin().read_line, so the READY marker every other submitted line
/// waits for never arrives. `needle` is the guard's own prompt text, and synchronising on it is
/// what makes this expressible at all.
///
/// `needle` is a parameter rather than a constant on purpose: the harness should not hold a copy
/// of the shell's wording. The case that cares states it.
pub fn run_repl_answered(cmd: &str, needle: &str, reply: &str) -> Result<Vec<String>, String> {
    run_session(
        &[cmd],
        &[],
        Some((needle, reply)),
        InputDelivery::Incremental,
    )
    .map(|(lines, _)| lines)
}

/// Submit SETUP LINES, then one line expected to hit the guard, and answer its prompt.
///
/// INT-197: the single-command limit above existed because the answer applied to every line. It
/// applies to the LAST line now, so setup can precede the line under test -- an alias defined on
/// one line and invoked on the next, which is the only way to test the alias gate through the REPL
/// door, since a semicolon puts the invocation in a segment the guard does not expand.
/// ⚠️ NO CALLER YET, and that is recorded rather than hidden. The case this was built for --
/// INT-197 gate 6, an alias defined on one line and invoked on the next -- times out in the pty
/// even though the same two lines gate correctly when piped to the shell by hand. The harness
/// problem is unresolved and named in INT-197; this door is kept because the change beneath it,
/// applying the answer to the LAST line rather than every line, corrects a real limitation that the
/// single-command door above documents.
#[allow(dead_code)]
pub fn run_repl_answered_after(
    setup: &[&str],
    cmd: &str,
    needle: &str,
    reply: &str,
) -> Result<Vec<String>, String> {
    let mut all: Vec<&str> = setup.to_vec();
    all.push(cmd);
    run_session(&all, &[], Some((needle, reply)), InputDelivery::Incremental)
        .map(|(lines, _)| lines)
}

/// INT-286: WHO OWNS THE PTY. Measured 2026-10-12 on nsh 5.3.1: as the harness has always spawned
/// it, the pty is nsh's stdio but NOT its controlling terminal -- tcgetpgrp fails with ENOTTY,
/// tty::is_interactive() is false, and nsh runs every REPL case without job control. Started as a
/// session leader on the same pty it owns the foreground and hands it to its children. Inherited
/// keeps the old spawn byte for byte, so no existing case changes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Terminal {
    /// The spawn every case has always had: no session, no controlling terminal.
    Inherited,
    /// nsh is a session leader with the pty as its controlling terminal, and owns the foreground.
    Owned,
    /// As Owned, but a holder process has the foreground before nsh runs, so the shell starts on
    /// a terminal it does not own -- where a background job of another shell would be.
    NotOwned,
}

/// What one session ended with. `exit` is nsh's OWN process status, which no command's 133;D
/// mark can report: a shell that ends with 0 after losing its terminal looks like success there.
pub struct SessionEnd {
    pub lines: Vec<String>,
    pub status: Option<i32>,
    pub banner: String,
    /// None when nsh was ended by a signal, including the harness's own kill.
    pub exit: Option<i32>,
    /// INT-290: the signal that ended nsh, when one did. `exit` is None then; this says which.
    pub signal: Option<i32>,
    /// false when nsh ended before its first prompt.
    pub prompted: bool,
    /// Owned modes only (empty for Inherited): every line nsh printed after its first prompt
    /// until it exited, the harness's own teardown included.
    pub transcript: Vec<String>,
}

/// Submit lines to nsh on a pty it owns or does not own, and say how the session ended.
///
/// `gate` is (line index, marker): the line at that index is not sent until the marker has been
/// printed. A case whose next line must follow an event the shell does not report -- a background
/// job taking the terminal -- synchronises on the event, never on a sleep.
pub fn run_repl_gated(
    cmds: &[&str],
    env: &[(&str, &str)],
    terminal: Terminal,
    gate: Option<(usize, &str)>,
) -> Result<SessionEnd, String> {
    run_session_full(cmds, env, None, InputDelivery::Incremental, terminal, gate)
}

thread_local! {
    static HANGUP_ENDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// INT-290: run an owned session and END it the way a closed window does: no `exit` is typed,
/// the pty master is closed and the kernel hangs the shell up. `SessionEnd.signal` says what
/// ended it. The flag is thread-local, so cases running side by side cannot affect each other.
pub fn run_repl_hangup(
    cmds: &[&str],
    env: &[(&str, &str)],
    gate: Option<(usize, &str)>,
) -> Result<SessionEnd, String> {
    HANGUP_ENDING.with(|c| c.set(true));
    let end = run_repl_gated(cmds, env, Terminal::Owned, gate);
    HANGUP_ENDING.with(|c| c.set(false));
    end
}

/// INT-290: pids of live `sleep` processes that carry `arg` as an argument. Read from /proc, so
/// no tool is needed; a defunct process has an empty command line and is not counted.
pub fn live_pids_with_cmdline(arg: &str) -> Vec<i32> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir("/proc") {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if let Ok(pid) = name.parse::<i32>() {
                if let Ok(raw) = std::fs::read(format!("/proc/{}/cmdline", pid)) {
                    let args: Vec<&[u8]> = raw.split(|b| *b == 0).collect();
                    let is_sleep = args.first().map(|a| a.ends_with(b"sleep")).unwrap_or(false);
                    if is_sleep && args.iter().any(|a| *a == arg.as_bytes()) {
                        out.push(pid);
                    }
                }
            }
        }
    }
    out
}

/// Give the spawn the terminal arrangement asked for. Inherited touches nothing.
///
/// Runs in the forked child before exec, after stdio is in place, so fd 0 is the pty. Only
/// async-signal-safe calls: setsid, ioctl, fork, setpgid, signal, tcsetpgrp, close, alarm, pause.
/// The holder ignores nothing and outlives no one: it is in the foreground group, so the kernel
/// sends it SIGHUP when nsh, the controlling process, exits; alarm(20) ends it regardless.
fn own_terminal(cmd: &mut Command, terminal: Terminal) {
    use nix::libc;
    use std::os::unix::process::CommandExt;
    if terminal == Terminal::Inherited {
        return;
    }
    let not_owned = terminal == Terminal::NotOwned;
    unsafe {
        cmd.pre_exec(move || {
            if libc::setsid() < 0 || libc::ioctl(0, libc::TIOCSCTTY, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if not_owned {
                let holder = libc::fork();
                if holder < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if holder == 0 {
                    libc::setpgid(0, 0);
                    libc::close(0);
                    libc::close(1);
                    libc::close(2);
                    libc::alarm(20);
                    loop {
                        libc::pause();
                    }
                }
                libc::setpgid(holder, holder);
                libc::signal(libc::SIGTTOU, libc::SIG_IGN);
                let rc = libc::tcsetpgrp(0, holder);
                libc::signal(libc::SIGTTOU, libc::SIG_DFL);
                if rc < 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            Ok(())
        });
    }
}

/// THE ONE OWNER OF THE SESSION PROTOCOL: spawn, reader thread, submit, capture, teardown.
///
/// `answer` exists for exactly ONE situation, and it is a genuine state transition rather than a
/// harness convenience. fsh's safety guard LEAVES rustyline and performs a raw
/// `std::io::stdin().read_line`, so NO bracketed-paste READY marker is emitted while it waits for
/// the reply. A caller submitting a guarded line must therefore synchronise on the guard's own
/// prompt text and send the reply UNWRAPPED: a pasted reply would carry the paste markers into
/// `input` and fail `input.trim() == "yes"` for the wrong reason, which is a test passing by
/// accident.
///
/// The tuple is (prompt needle to wait for, text to send). `None` reproduces the previous
/// behaviour exactly, because the `if let` below never fires.
///
/// LIMIT, STATED RATHER THAN DISCOVERED LATER: an answer applies to EVERY submitted line, so it is
/// meaningful only for single-command sessions. The answered door enforces that by taking one
/// command.
/// The session runner every case but one uses: what nsh printed before its first prompt is
/// dropped. INT-262: a thin wrapper, so the shell is still started in exactly one place.
fn run_session(
    cmds: &[&str],
    env: &[(&str, &str)],
    answer: Option<(&str, &str)>,
    delivery: InputDelivery,
) -> Result<(Vec<String>, Option<i32>), String> {
    run_session_banner(cmds, env, answer, delivery).map(|(lines, status, _)| (lines, status))
}

/// INT-262: submit lines, and also return what nsh printed BEFORE its first prompt. A startup
/// report -- an unreadable persisted store -- lands there, where no other runner can see it.
pub fn run_repl_lines_banner(
    cmds: &[&str],
    env: &[(&str, &str)],
) -> Result<(Vec<String>, String), String> {
    run_session_banner(cmds, env, None, InputDelivery::Incremental)
        .map(|(lines, _, banner)| (lines, banner))
}

fn run_session_banner(
    cmds: &[&str],
    env: &[(&str, &str)],
    answer: Option<(&str, &str)>,
    delivery: InputDelivery,
) -> Result<(Vec<String>, Option<i32>, String), String> {
    // INT-289 G1 a: OWNED IS THE DEFAULT. nsh runs as it does in a window -- a session leader on
    // the pty, owning the foreground -- so a case that passes here passed with job control.
    // Inherited is kept for one case, repl_289_inherited_terminal_still_answers.
    let end = run_session_full(cmds, env, answer, delivery, Terminal::Owned, None)?;
    // An owned session may end before its first prompt (INT-286 asserts on that). For every case
    // that is not asking for it, that is still the error it always was.
    if !end.prompted {
        return Err("nsh never reached its first prompt".to_string());
    }
    Ok((end.lines, end.status, end.banner))
}

/// How the wait for one submitted line ended.
enum LineEnd {
    Done,
    TimedOut,
    /// A line the case declared held produced its own 133;D: the declaration is wrong.
    HeldButCompleted,
}

/// INT-289 G1 c: WHERE A LINE'S ANSWER ENDS. A ready marker alone does not end a line: a take-back
/// prints a second one before it answers (F-0046), and any child that starts a line editor
/// prints one too. A line ends at its OWN `133;D` status mark, then the ready marker after it.
/// Measured 2026-10-16: `133;D` follows the line after a take-back in 30 of 30 runs, before its
/// `133A` and its ready marker.
///
/// A line the case declared HELD completes no command, so no `133;D` comes; it ends at its
/// ready marker, the continuation prompt. If a `133;D` does come, the declaration is wrong, and
/// that is an error rather than a guess.
///
/// Every search starts at `mark`, so nothing printed for an earlier line can end this one.
fn wait_line_end(
    rx: &mpsc::Receiver<Vec<u8>>,
    raw: &mut Vec<u8>,
    ready: &[u8],
    mark: usize,
    held: bool,
) -> LineEnd {
    const OSC_D_MARK: &[u8] = b"\x1b]133;D";
    let limit = Duration::from_secs(30);
    if held {
        let Some(at) = wait_for(rx, raw, ready, mark, limit) else {
            return LineEnd::TimedOut;
        };
        return match find_from(raw, OSC_D_MARK, mark) {
            Some(d) if d < at => LineEnd::HeldButCompleted,
            _ => LineEnd::Done,
        };
    }
    let Some(d) = wait_for(rx, raw, OSC_D_MARK, mark, limit) else {
        return LineEnd::TimedOut;
    };
    match wait_for(rx, raw, ready, d + OSC_D_MARK.len(), limit) {
        Some(_) => LineEnd::Done,
        None => LineEnd::TimedOut,
    }
}
/// INT-286: the session protocol, with the terminal arrangement and a gate. Every runner above
/// reaches nsh through here, so the shell is still started in exactly one place.
fn run_session_full(
    cmds: &[&str],
    env: &[(&str, &str)],
    answer: Option<(&str, &str)>,
    delivery: InputDelivery,
    terminal: Terminal,
    gate: Option<(usize, &str)>,
) -> Result<SessionEnd, String> {
    let pty = openpty(None, None).map_err(|e| format!("openpty: {}", e))?;

    let s_in = pty.slave.try_clone().map_err(|e| e.to_string())?;
    let s_out = pty.slave.try_clone().map_err(|e| e.to_string())?;
    let s_err = pty.slave.try_clone().map_err(|e| e.to_string())?;

    // INT-290: a hang-up ending closes the pty master, which is what a closed window does. The
    // reader thread holds a second descriptor for it, so in this mode the thread polls with a
    // short deadline and drops its descriptor when told to stop; otherwise nothing would hang up.
    let hangup = HANGUP_ENDING.with(|c| c.get());
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_reader = stop.clone();
    if hangup {
        // INT-290: the shell must not inherit the master. openpty leaves it open across exec, so a
        // shell holding its own master keeps the pty alive and no close is ever a hang-up. Measured
        // 2026-10-17: without this, nsh ran to the harness deadline and was killed (signal 9).
        let master_fd = std::os::fd::AsRawFd::as_raw_fd(&pty.master);
        unsafe {
            nix::libc::fcntl(master_fd, nix::libc::F_SETFD, nix::libc::FD_CLOEXEC);
        }
    }
    let mut cmd = Command::new(fsh_bin());
    own_terminal(&mut cmd, terminal);
    let mut child = cmd
        // INT-206: the harness default goes FIRST so a case can override it. fsh starts in the
        // repo home deliberately (it once also restored its last directory), so the current_dir below
        // was silently ignored for months and conformance cases wrote their files into the
        // repository. NSH_KEEP_CWD suppresses that override.
        //
        // Set for every case rather than per-case, so a case added later cannot pollute the repo by
        // forgetting to opt in. One guardian case passes "0" and asserts the repo-home default,
        // so the behaviour daily use actually gets is still covered by a case that says so.
        .env("NSH_KEEP_CWD", "1")
        // INT-204: a FRESH DATABASE PER CASE, because the pollution this intent is about happens
        // WITHIN a run. repl_193 creates an alias and later cases in the same run see it -- a single
        // scratch database would only move that somewhere else. Measured at 20ms per case, 2.8s
        // across the suite, which is 3.4% of it; a template-copy scheme would save that and cost
        // more moving parts than it is worth.
        .env("ZERO_STATE_DB", case_db_path())
        .envs(env.iter().copied())
        .current_dir("/tmp")
        .stdin(Stdio::from(s_in))
        .stdout(Stdio::from(s_out))
        .stderr(Stdio::from(s_err))
        .spawn()
        .map_err(|e| format!("spawn nsh: {}", e))?;
    drop(pty.slave);

    let mut master = std::fs::File::from(pty.master);
    let mut reader = master.try_clone().map_err(|e| e.to_string())?;
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            if hangup {
                let mut pfd = nix::libc::pollfd {
                    fd: std::os::fd::AsRawFd::as_raw_fd(&reader),
                    events: nix::libc::POLLIN,
                    revents: 0,
                };
                let ready = unsafe { nix::libc::poll(&mut pfd, 1, 20) };
                if ready == 0 {
                    if stop_reader.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                    continue;
                }
            }
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

    // WAIT FOR MARKERS, DO NOT SLEEP. `\x1b[?2004h` is bracketed-paste-on: the line editor
    // announcing it is ready for input, which is exactly what every sleep here was
    // approximating. Measured over four runs 2026-08-05, the first prompt arrives in
    // 0.594-0.624s -- a thirty-millisecond spread -- so waiting for it is both faster than
    // 2500ms and DETERMINISTIC, which a sleep can never be. The determinism is the point and
    // the speed is a side effect.
    //
    // ⚠️ A TIMEOUT IS A FAILURE, not an empty capture.
    const READY: &[u8] = b"\x1b[?2004h";
    let mut raw: Vec<u8> = Vec::new();
    // INT-286: on a terminal it does not own nsh may end before any prompt, and that ending is
    // what such a case asserts on -- an error here would hide it. Inherited keeps the error.
    let prompted = wait_for(&rx, &mut raw, READY, 0, Duration::from_secs(20)).is_some();
    if !prompted && terminal == Terminal::Inherited {
        return Err("nsh never reached its first prompt".to_string());
    }
    let cmds: &[&str] = if prompted { cmds } else { &[] };
    // The banner belongs to no command, so it is in no line's capture. INT-262: it is returned
    // on its own instead of dropped, because nsh's startup reports print there.
    let banner = strip_ansi(&raw);
    raw.clear();

    // Each line waits for the prompt to return before the next is sent, because a later command
    // may depend on an earlier one having finished. The search starts at `mark` so the PREVIOUS
    // prompt cannot satisfy this one.
    //
    // The limit is generous deliberately: some cases run a literal `sleep 2`. The win is that a
    // fast command returns in milliseconds instead of always paying 1200ms -- not that slow
    // commands get cut short.
    for (idx, cmd) in cmds.iter().enumerate() {
        // INT-197: THE ANSWER BELONGS TO THE LAST LINE ONLY.
        //
        // It used to apply to every submitted line, which is why the answered door took exactly one
        // command and said so. That limit blocked two gates in two intents: INT-196 M8 needs several
        // lines and THEN an answer, and INT-197 gate 6 needs an alias defined on one line and
        // invoked on the next. A gap that blocks two intents is worth closing rather than working
        // around twice.
        //
        // SETUP LINES SUBMIT NORMALLY and wait for READY like any other line. Only the final line
        // synchronises on the guard prompt, because that is the line under test.
        // INT-286: in an owned mode a shell that has already ended is the result; stop sending.
        if terminal != Terminal::Inherited && matches!(child.try_wait(), Ok(Some(_))) {
            break;
        }
        if let Some((at, needle)) = gate {
            if idx == at {
                wait_for(&rx, &mut raw, needle.as_bytes(), 0, Duration::from_secs(20))
                    .ok_or_else(|| format!("never saw {needle:?} before line {at}"))?;
            }
        }
        let answer = if idx + 1 == cmds.len() { answer } else { None };
        let mark = raw.len();
        // PASTE THE LINE, DO NOT TYPE IT. fsh's highlighter repaints the whole line for every
        // byte it receives, so a harness writing 37 characters at once pays 37 full redraws back
        // to back. Measured 2026-08-05 on the debug binary: the same 37-character line takes
        // 1.392s to reach the submit marker when written raw and 0.040s inside a bracketed paste,
        // and pasted submit time is FLAT with length. A human typing pays those redraws one
        // keystroke apart and never notices; this is not a bug being worked around, it is the
        // difference between simulating a typist and delivering a line.
        //
        // The trailing newline sits OUTSIDE the paste-end marker, and rustyline submits on it.
        master.write_all(b"\x1b[200~").map_err(|e| e.to_string())?;
        master
            .write_all(cmd.as_bytes())
            .map_err(|e| e.to_string())?;
        master
            .write_all(b"\x1b[201~\n")
            .map_err(|e| e.to_string())?;
        if let Some((needle, reply)) = answer {
            wait_for(
                &rx,
                &mut raw,
                needle.as_bytes(),
                mark,
                Duration::from_secs(20),
            )
            .ok_or_else(|| format!("never saw the prompt {needle:?} after: {cmd}"))?;
            master
                .write_all(reply.as_bytes())
                .map_err(|e| e.to_string())?;
            master.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        // EVERY LINE SYNCHRONISES, INCLUDING A HELD ONE -- and the first version of this
        // skipped the wait, which was wrong in a way worth recording.
        //
        // The reasoning was that a continuation line completes no command so no prompt comes
        // back. Measured in a real pty, that is false: after `for i in alpha beta` fsh emits
        // ?2004h again for the CONTINUATION read. The marker arrives; it just means ready for
        // the next line rather than command finished.
        //
        // And skipping it broke the construct outright. Writing three lines without waiting
        // put all three in the pty before rustyline came back for them, so it read them as ONE
        // buffered submission, submitted `for i in alpha beta` alone, and consumed `do ...`
        // and `done` as part of that same read. They were gone. The shell then sat at `... `
        // waiting for a body that had already been swallowed -- which is why the capture held
        // the continuation prompt and nothing else.
        //
        // THE WAIT IS A HANDSHAKE HERE, not a command boundary: it is the acknowledgment that
        // fsh has consumed the previous line and is asking for the next. Do not batch these
        // writes as an optimisation; typing by hand works only because the pauses are there.
        if delivery == InputDelivery::Buffered && idx + 1 < cmds.len() {
            continue;
        }
        // INT-289 G1 c: the line ends at its own 133;D and the ready marker after it, unless the
        // case declared it held (wait_line_end).
        let held = matches!(delivery, InputDelivery::Held(h) if h.contains(&idx));
        match wait_line_end(&rx, &mut raw, READY, mark, held) {
            LineEnd::Done => {}
            LineEnd::HeldButCompleted => {
                return Err(format!(
                    "line {idx} was declared held but produced its own 133;D: {cmd}"
                ));
            }
            LineEnd::TimedOut => {
                // INT-286: a shell that ENDED mid-session is the result of an owned-mode case.
                // INT-289: owned is the default now, so a shell that is still alive and silent is
                // a hang, and a hang is a failure, as it always was. Inherited keeps its error.
                let ended = matches!(child.try_wait(), Ok(Some(_)));
                if terminal == Terminal::Inherited || !ended {
                    return Err(format!("timed out waiting for the prompt after: {cmd}"));
                }
                break;
            }
        }
    }

    // ASK, THEN WAIT FOR THE ANSWER. This was a flat 300ms sleep before killing the child,
    // which is a quarter of what a case costs now that nothing else sleeps. fsh exits in tens
    // of milliseconds when told to; the kill remains as the deadline for the case where it
    // does not. Teardown, not the per-line path -- so this is the speed gate's work rather
    // than a correction to the one about fixed sleeps.
    if hangup {
        // INT-290: the window closes. Nothing is typed; the master goes away and the kernel
        // hangs up the shell. The deadline is longer: an exit path that saves things takes time.
        stop.store(true, std::sync::atomic::Ordering::SeqCst);
        drop(master);
    } else {
        let _ = master.write_all(b"exit\n");
    }
    let deadline = Instant::now()
        + if hangup {
            Duration::from_secs(5)
        } else {
            Duration::from_millis(500)
        };
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(5));
            }
            _ => {
                let _ = child.kill();
                break;
            }
        }
    }
    let waited = child.wait().ok();
    let exit = waited.and_then(|s| s.code());
    let signal = waited.and_then(|s| std::os::unix::process::ExitStatusExt::signal(&s));
    // INT-286: AN OWNED-MODE CASE READS PAST THE LAST PROMPT. A shell that takes its terminal back
    // prompts again BEFORE it answers the line it was sent, so the prompt the loop above waited for
    // can arrive first and the answer after it, during teardown (measured 2026-10-12: ALIVE came
    // 1ms after that prompt). What nsh printed until it exited becomes `transcript`; the reader is
    // drained until it has been quiet for 50ms (INT-289 G1 d; it was 200ms), 2s at most. Inherited skips this, so its cost and
    // its lines are unchanged.
    let mut late: Vec<u8> = Vec::new();
    if terminal != Terminal::Inherited {
        let until = Instant::now() + Duration::from_secs(2);
        while let Ok(chunk) = rx.recv_timeout(Duration::from_millis(50)) {
            late.extend_from_slice(&chunk);
            if Instant::now() >= until {
                break;
            }
        }
    }

    // THE WINDOW IS BOUNDED BY TWO MARKERS, NOT BY A TIMEOUT. It runs from `?2004l`, which the
    // line editor emits when the line is submitted, to `133;A`, which fsh emits when it starts
    // drawing the next prompt. Between them is the command's output and nothing else.
    //
    // Taking everything after `?2004l` with no end boundary was the old behaviour, and the
    // capture then ran until drain's quiet timeout happened to fall -- which is why the same
    // case passed when the prompt was slow and failed when it was fast.
    //
    // ⚠️ `133;C`/`133;D` LOOK LIKE THE RIGHT BOUNDARY AND ARE NOT. OSC 133;C means "output
    // starts here", but fsh emits B, C and D in a cluster AFTER the command has finished for
    // every path that spawns a child -- the child inherits the pty and writes first. Measured
    // 2026-08-05: `echo ZZBUILTIN` puts its output between C and D, while `echo hi | grep h`
    // puts `hi` BEFORE B and leaves C..D empty. A C..D window broke 16 tests, all of them
    // pipelines, sequences or sh/bash/python3. That is an fsh shell-integration bug and it is
    // not this harness's to fix; the window is chosen so the harness does not depend on it.
    //
    // The prompt is excluded structurally because it begins AT `133;A`. The editor's
    // per-keystroke repaints and history autosuggestions are excluded because they precede
    // `?2004l`. The stray B/C/D tokens land inside the window and strip_ansi removes them,
    // since it handles OSC as well as CSI.
    //
    // `133;D;<status>` carries the exit code and lands INSIDE this window, so the status is read
    // from the same bytes rather than captured separately.
    //
    // THE FALLBACK IS THE OLD BEHAVIOUR, ON PURPOSE: a command that takes the screen or ends
    // the session may emit no `133;A`, and an empty capture would read as a passing assertion
    // about absence.
    const OSC_A: &[u8] = b"\x1b]133;A";
    let tail = match find_last(&raw, b"\x1b[?2004l") {
        Some(i) => {
            let start = i + 8;
            let end = find_from(&raw, OSC_A, start).unwrap_or(raw.len());
            &raw[start..end]
        }
        None => &raw[..],
    };
    const OSC_D: &[u8] = b"\x1b]133;D;";
    let status = find_last(tail, OSC_D).and_then(|i| {
        let from = i + OSC_D.len();
        let digits: Vec<u8> = tail[from..]
            .iter()
            .copied()
            .take_while(|b| b.is_ascii_digit())
            .collect();
        std::str::from_utf8(&digits).ok()?.parse::<i32>().ok()
    });
    let lines: Vec<String> = strip_ansi(tail)
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let transcript: Vec<String> = if terminal == Terminal::Inherited {
        Vec::new()
    } else {
        let mut all = raw.clone();
        all.extend_from_slice(&late);
        strip_ansi(&all)
            .split('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };
    Ok(SessionEnd {
        lines,
        status,
        banner,
        exit,
        signal,
        prompted,
        transcript,
    })
}

#[cfg(test)]
mod bin_tests {
    use super::resolve_bin;
    use std::path::PathBuf;

    /// INT-287 G4: both arms refuse a binary that is not on disk, and name it.
    #[test]
    fn a_missing_binary_is_refused_on_both_arms() {
        let here = env!("CARGO_MANIFEST_DIR").to_string();
        let gone = PathBuf::from(&here).join("no-such-binary-int287");
        let named = resolve_bin(
            "X_BIN",
            Some(gone.display().to_string()),
            PathBuf::from(&here),
        );
        assert!(named
            .unwrap_err()
            .contains("names a binary that does not exist"));
        let deployed = resolve_bin("X_BIN", None, gone.clone());
        assert!(deployed.unwrap_err().contains("is not set"));
        assert_eq!(
            resolve_bin("X_BIN", Some(here.clone()), gone.clone()),
            Ok(here.clone())
        );
        assert_eq!(resolve_bin("X_BIN", None, PathBuf::from(&here)), Ok(here));
    }

    /// F-0034: a relative NSH_BIN is made absolute before any case runs; an absolute one is kept.
    #[test]
    fn a_relative_shell_path_is_made_absolute() {
        let got = super::absolute_shell("target/debug/nsh");
        assert!(std::path::Path::new(&got).is_absolute(), "{got}");
        assert!(got.ends_with("target/debug/nsh"), "{got}");
        assert_eq!(super::absolute_shell("/usr/bin/nsh"), "/usr/bin/nsh");
    }
}

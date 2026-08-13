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
use std::time::{Duration, Instant};

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
    format!("/tmp/fsh-test-{}", std::process::id())
}

pub fn run_repl_lines_status(
    cmds: &[&str],
    env: &[(&str, &str)],
) -> Result<(Vec<String>, Option<i32>), String> {
    run_session(cmds, env, None)
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
    run_session(&[cmd], &[], Some((needle, reply))).map(|(lines, _)| lines)
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
    run_session(&all, &[], Some((needle, reply))).map(|(lines, _)| lines)
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
fn run_session(
    cmds: &[&str],
    env: &[(&str, &str)],
    answer: Option<(&str, &str)>,
) -> Result<(Vec<String>, Option<i32>), String> {
    let pty = openpty(None, None).map_err(|e| format!("openpty: {}", e))?;

    let s_in = pty.slave.try_clone().map_err(|e| e.to_string())?;
    let s_out = pty.slave.try_clone().map_err(|e| e.to_string())?;
    let s_err = pty.slave.try_clone().map_err(|e| e.to_string())?;

    let mut child = Command::new(fsh_bin())
        // INT-206: the harness default goes FIRST so a case can override it. fsh starts in the
        // forest home and restores its last directory, both deliberately, so the current_dir below
        // was silently ignored for months and conformance cases wrote their files into the
        // repository. FSH_KEEP_CWD suppresses both overrides.
        //
        // Set for every case rather than per-case, so a case added later cannot pollute the repo by
        // forgetting to opt in. One guardian case passes "0" and asserts the forest-home default,
        // so the behaviour daily use actually gets is still covered by a case that says so.
        .env("FSH_KEEP_CWD", "1")
        // INT-204: a FRESH DATABASE PER CASE, because the pollution this intent is about happens
        // WITHIN a run. repl_193 creates an alias and later cases in the same run see it -- a single
        // scratch database would only move that somewhere else. Measured at 20ms per case, 2.8s
        // across the suite, which is 3.4% of it; a template-copy scheme would save that and cost
        // more moving parts than it is worth.
        .env("FAELIGHT_STATE_DB", case_db_path())
        .envs(env.iter().copied())
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
    wait_for(&rx, &mut raw, READY, 0, Duration::from_secs(20))
        .ok_or_else(|| "fsh never reached its first prompt".to_string())?;
    raw.clear(); // the banner belongs to no command

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
        wait_for(&rx, &mut raw, READY, mark, Duration::from_secs(30))
            .ok_or_else(|| format!("timed out waiting for the prompt after: {cmd}"))?;
    }

    // ASK, THEN WAIT FOR THE ANSWER. This was a flat 300ms sleep before killing the child,
    // which is a quarter of what a case costs now that nothing else sleeps. fsh exits in tens
    // of milliseconds when told to; the kill remains as the deadline for the case where it
    // does not. Teardown, not the per-line path -- so this is the speed gate's work rather
    // than a correction to the one about fixed sleeps.
    let _ = master.write_all(b"exit\n");
    let deadline = Instant::now() + Duration::from_millis(500);
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
    let _ = child.wait();

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
    Ok((lines, status))
}

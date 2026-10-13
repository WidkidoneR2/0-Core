---
id: 286
date: 2026-10-08
type: future
title: "nsh ends on any readline error with a message the closing window never shows, so a lost terminal is a silent death"
status: complete
tags: [nsh, repl, readline, resilience, login, finding]
---

## START HERE

Begin at G0. Filed 2026-10-08 from F-0025 (found by INT-243), which carries the one live fact
F-0018 held when it closed not reproduced. Bears directly on INT-277.

## Vision

When nsh cannot read its terminal it recovers if it can, and if it cannot it leaves a reason that
outlives the window and an exit status that says it failed. A shell that is about to become the
login shell never dies without a word.

## The Problem

By reading (zero/shell/novashell/src/main.rs, 2026-10-08), the REPL matches rustyline 17.0.2:
- Err(ReadlineError::Interrupted) (Ctrl+C) clears the line and continues (:3071-3075).
- Err(ReadlineError::Eof) breaks (:3076-3078).
- Err(e), every other error, prints "Error: {e}" to stderr and breaks (:3079-3081).

After the break the shell writes its journal summary and exits (:3086). In a terminal that closes
with the shell, that stderr line is never seen, so any read error looks like the window dying.

tty.rs:69-74 ignores SIGTTIN, so a read from a terminal the shell does not own fails with an I/O
error instead of stopping the shell, and lands in that same arm. A foreground child that leaves
the terminal elsewhere is therefore a silent exit, not a recoverable state.

For INT-277: .bashrc execs nsh, so this arm ending the shell ends the terminal or the tty login.
Not yet measured: the exit status the shell returns after this arm, and a reproducible way to
reach it.

## The Solution

Ruled in G1 from G0's evidence. Candidates:
(a) Recover first: on an I/O error, take the terminal back (tty::take_terminal) and read again,
    a bounded number of times, before giving up.
(b) When giving up, write the reason to a durable log owned by zero-core paths before exiting,
    and exit non-zero, so 277's fallback and a person both have something to read.
(a) and (b) are not exclusive.

## Success Criteria

- [x] G0 census and trigger. Every readline call site and error arm in novashell; what follows
      the break; the exit status after the Err(e) arm, measured; and a reproducible way to drive
      the REPL into that arm (in the nsh-test pty harness or a sacrificial window), shown.
<!-- demonstrated 2026-10-12, read-only plus one probe:
CENSUS: one readline call site, novashell main.rs:2650 (match rl.readline(p)). Arms: :3071
Interrupted (newline, continue); :3076 Eof (break); :3079 Err(e) (eprintln "Error: {}", break).
rustyline 17.0.2 on unix has Io, Eof, Interrupted, Errno(nix) and Signal(Interrupt|Resize).
Signal never reaches nsh: enable_signals defaults false (config.rs:262), nsh never sets it, and
rustyline handles Resize itself (edit.rs:86-93). So Err(e) is reached by Io and Errno.
AFTER THE BREAK: core journal daily-summary (:3087), session save (:3091), session_patterns
(:3127), the summary print (:3162-3175), then main returns Ok(()) at :3177 -- exit status 0
whatever ended the loop.
SIGTTIN, SIGTTOU and SIGTSTP are ignored from startup (main.rs:814 -> tty.rs:64-75), so a read
of a terminal the shell does not own fails with EIO instead of stopping it.
TRIGGER (probe ~/.cache/zero/int286-probe-e446b341bf33.py, scratch ZERO_STATE_DB): nsh 5.3.1 on
a fresh pty whose foreground group is another process group. The first read fails, stderr gets
"Error: Input/output error (os error 5)", the exit path runs and prints its summary, exit status
0 after 0.2s. Reproducible on demand.
SEPARATE, not this arm: the same pty closed after the prompt -- nsh killed by signal 1 (SIGHUP)
after 0.3s, no exit path, no session save, stderr empty. Outside 286 (Non-goals); proposed as a
finding at the close.
-->
- [x] G1 rulings recorded here: which errors are retried and how often, where the durable record
      goes and which paths accessor owns it, and the exit status on giving up.
<!-- ruled 2026-10-12 by Christian, from G0's evidence:
RETRY: only an EIO from the read (ReadlineError::Io or Errno carrying os error 5), and only when
nsh owned its terminal at startup (tty::is_interactive() true then). nsh calls
tty::take_terminal() and reads again, at most 3 consecutive times; a good read resets the count.
A shell that never owned its terminal (started as another shell's background job) does not take
it from the rightful owner -- it gives up. Every other error gives up at once.
RECORD: on giving up, one line appended to a file under zero_state_dir(), through a new zero-core
paths accessor that is its one owner (its path read at G2 before use): time, pid, tty, the
error, the retries.
EXIT: 74 (EX_IOERR) on giving up, never 0. When the terminal itself is gone a fallback shell
cannot read it either, so INT-277's fallback should pass 74 through; the record explains it.
NOT CHOSEN: replacing rustyline (reedline; INT-168, INT-198). rustyline reports the error
correctly; the defect is nsh's arm. Revisited only with something rustyline demonstrably cannot
do.
-->
- [x] G2 red. A test driving the REPL into the Err(e) arm, shown FAILING on the current code
      against the G1 rulings, output quoted.
<!-- demonstrated 2026-10-12 on the code before the fix (debug nsh 5.3.1; novashell unchanged):
HARNESS, plan seal 182274beeac3b220: nsh-test repl.rs gains Terminal (Inherited, Owned, NotOwned),
SessionEnd (carrying nsh's own exit status, which no 133;D mark reports) and run_repl_gated (a line
held back until a marker prints). Every existing runner reaches nsh through run_session_full with
Inherited, so their spawn is unchanged: the other 236 cases stayed green.
FOUND ON THE WAY, measured: as the harness has always spawned it, nsh's pty is not its controlling
terminal (tcgetpgrp fails with ENOTTY), so tty::is_interactive() is false and no REPL case had run
with job control; as a session leader on the same pty nsh owns the foreground. And nsh gives a
background job /dev/null for stdin (fds 1 and 2 are the pty), so case 1's helper takes the
terminal through fd 1 (plan seal a7ff86683bab68af). The first red failed for that reason --
never saw STOLEN -- and was not counted.
RED: cargo fmt --check -p nsh-test exit 0; cargo build -p novashell -p nsh-test; then
NSH_BIN=/home/christian/0-core/target/debug/nsh ./target/debug/nsh-test --failed, exit 1, 238
results, the only failures:
  repl_286_lost_terminal_is_taken_back: nsh did not answer after losing its terminal: prompted
  true, exit Some(0), last lines ["Error: ...echo ALIVE...", "Input/output error (os error 5)",
  "Session complete", "2 commands ...", "Active: INT-286", "Project 0 remembers."]
  repl_286_terminal_never_owned_exits_74_with_a_record: want exit 74 and a record naming the I/O
  error; got exit Some(0), record ""
Case 1 took 30s while red: the last prompt wait runs to its limit once nsh has died.
-->
- [x] G3 fix. G2 green; Ctrl+C and Ctrl+D behave exactly as before, shown.
<!-- demonstrated 2026-10-12 on the debug nsh 5.3.1 built from this tree:
FIX, plan seal 6e6b87a04a32fea4. zero-core: paths::nsh_exit_log(), nsh-exit.log in the state dir.
novashell main.rs: is_eio() (rustyline's Io and Errno forms, by number) and record_gave_up(). The
inner read arm takes the terminal back and reads again when the shell owned it at REPL start
(tty::is_interactive), at most 3 EIOs in a row, a good read resetting the count. The outer Err arm
keeps its stderr line, appends one record line and exits 74 after the normal exit path. Every
other error, Interrupted and Eof are untouched.
FOUND ON THE WAY: the first green run failed case 1 alone, in 217ms, exit Some(0). A probe that
replays the case and keeps reading past the harness (int286-sync) showed nsh right and the case
wrong: after taking the terminal back nsh prompts again BEFORE it reads the waiting line, the
harness stopped at that prompt, and ALIVE came 1ms later. HARNESS, plan seal a999d5217e9aa412:
SessionEnd.transcript, owned modes only -- what nsh printed until it exited, the reader drained
until quiet for 200ms (2s cap); case 1 now asserts ALIVE as its own line in the transcript.
Inherited cases skip the drain, so their cost and their lines are unchanged.
GREEN: cargo fmt --check -p nsh-test -p novashell -p zero-core exit 0; cargo build -p novashell
-p nsh-test; NSH_BIN=/home/christian/0-core/target/debug/nsh ./target/debug/nsh-test --failed:
238 results stored, All tests passed, exit 0.
STILL RED WITHOUT THE FIX, same suite, NSH_BIN=/home/christian/.local/bin/nsh (deployed 5.3.1):
exit 1, the only failures the two repl_286 cases -- case 1 exit Some(0), lines "Error:
Input/output error (os error 5)", "Session complete", ...; case 2 got exit Some(0), record "".
KEYS, by hand in the debug nsh: Ctrl+C on a half-typed line cleared it to a fresh prompt; Ctrl+D
opened the health TUI (INT-258, main.rs:2518/2536/2716) exactly as the deployed 5.3.1 does.
CORRECTED: G2's RED line named NSH_BIN=target/debug/nsh; the run behind it passed the absolute
path. A relative NSH_BIN fails every REPL case with "spawn nsh: No such file or directory" (they
spawn from /tmp) -- measured today, 109 cases. A finding for INT-287.
-->
- [x] G4 doors. One ship at the end; the deployed nsh tested in a new shell; nsh-test all passing;
      d 0 failed.
<!-- demonstrated 2026-10-12:
BUMP, plan seal f932d890fece1809, the version call delegated to Claude: novashell 5.3.1 -> 5.3.2
and nsh-test 2.0.2 -> 2.0.3, both patch -- each crate's one Cargo.toml version line and its
tools.toml entry, then cargo update -p <pkg> --precise <new> (cicomplete's sync_lock command);
Cargo.lock read back at 5.3.2 and 2.0.3. zero-core stays 1.0.0: its versioning is unsettled by
Christian's ruling. Bumped BEFORE the ship so the doors test the numbered artifact; cicomplete's
offer for novashell is skipped at the close for that reason.
SHIP, once: ship nsh installed nsh 5.3.2, ship nsh-test installed nsh-test 2.0.3, each 1 shipped
0 failed.
SUITE: the deployed nsh-test with NSH_BIN unset -- nsh-test v2.0.3, under test 5.3.2, no
different-bytes warning; 238 results stored, All tests passed, exit 0.
d: Passed 26, Warnings 2 (Git Repository uncommitted changes; Update Readiness, commit or stash),
Failed 0.
NEW SHELL: a new terminal opened the shipped nsh; echo new shell ok answered; nsh-exit.log absent
-- no nsh has given up on this machine.
-->
- [x] G5 commit. Intent: INT-286, Fixes: F-0025, and the plan Seal with its note under
      refs/notes/seals.
<!-- demonstrated 2026-10-12: commit 57389885 (int286-g5, commit mode). Its checks ran first: note
digest 6e6b87a04a32fea4 == Seal, G0 to G4 ticked, the tree exactly the ten expected changes. Then
exactly those ten paths staged: 10 files changed, 501 insertions, 84 deletions. Trailers Intent:
INT-286, Fixes: F-0025, Seal: 6e6b87a04a32fea4. The fix plan's text, rebuilt byte for byte from
its show output, went to refs/notes/seals by hash-object -w and notes add -C (blob 3dfd7d4d971e);
read back, note matches plan text: True. core intent trace 57389885: intent INT-286, fixes F-0025,
seal 6e6b87a04a32fea4 intact. -->
- [x] G6 the push. The pre-push hook passes; main and refs/notes/seals reach GitHub; trace reads
      the seal intact.
<!-- demonstrated 2026-10-12: git push origin main refs/notes/seals, one push, so the pre-push gate
(zero-gate pre-push: builds novashell and nsh-test, runs the suite with NSH_BIN at the absolute
target/debug/nsh) ran once, and git sent both refs only after it exited 0 -- 50.6s; the gate's
own lines were not shown in the terminal. main bd936309..57389885 and refs/notes/seals
93389739..ea269852. git ls-remote then matched the local refs: main 573898856b70,
refs/notes/seals ea2698526ee9. core intent trace 57389885 after the push reads fixes F-0025 and
seal 6e6b87a04a32fea4 intact. -->
- [x] G7 each gate carries evidence per INT-158.
<!-- demonstrated 2026-10-12: G0 to G6 each carry an evidence comment (G1's is the ruling, the
others demonstrated) -- G0 to G4 checked by the plan that writes this one, G5's and G6's written
beside it. -->

## Non-goals

- No change to .bashrc or the bash-to-nsh handoff; INT-277 owns that and its crash fallback.
- Ctrl+C and end-of-input keep their current meaning.
- No new crates, no sudo, no chsh.
- Not a general terminal-recovery layer: only the read error that ends the REPL.

## Versions
- novashell skipped

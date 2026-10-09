---
id: 286
date: 2026-10-08
type: future
title: "nsh ends on any readline error with a message the closing window never shows, so a lost terminal is a silent death"
status: planned
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

- [ ] G0 census and trigger. Every readline call site and error arm in novashell; what follows
      the break; the exit status after the Err(e) arm, measured; and a reproducible way to drive
      the REPL into that arm (in the nsh-test pty harness or a sacrificial window), shown.
- [ ] G1 rulings recorded here: which errors are retried and how often, where the durable record
      goes and which paths accessor owns it, and the exit status on giving up.
- [ ] G2 red. A test driving the REPL into the Err(e) arm, shown FAILING on the current code
      against the G1 rulings, output quoted.
- [ ] G3 fix. G2 green; Ctrl+C and Ctrl+D behave exactly as before, shown.
- [ ] G4 doors. One ship at the end; the deployed nsh tested in a new shell; nsh-test all passing;
      d 0 failed.
- [ ] G5 commit. Intent: INT-286, Fixes: F-0025, and the plan Seal with its note under
      refs/notes/seals.
- [ ] G6 the push. The pre-push hook passes; main and refs/notes/seals reach GitHub; trace reads
      the seal intact.
- [ ] G7 each gate carries evidence per INT-158.

## Non-goals

- No change to .bashrc or the bash-to-nsh handoff; INT-277 owns that and its crash fallback.
- Ctrl+C and end-of-input keep their current meaning.
- No new crates, no sudo, no chsh.
- Not a general terminal-recovery layer: only the read error that ends the REPL.

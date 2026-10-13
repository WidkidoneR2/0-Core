---
id: 289
date: 2026-10-13
type: future
title: "nsh-test runs the REPL without job control, so a green case does not show how nsh behaves in a real window"
status: planned
tags: [nsh-test, repl, job-control, harness, pty, coverage, finding]
depends_on: [287]
---

## START HERE

Filed 2026-10-13 from what INT-286 measured about the harness. Begin at G0 after INT-287 is
complete. INT-290 (SIGHUP) and INT-277 (the login path) both need its tests to mean what they say.

## Vision

A green REPL case means nsh behaved as it does in a real window: it owns its terminal, job control
is on, and every prompt and answer it prints is seen before the case decides.

## The Problem (measured 2026-10-12 during INT-286)

1. No controlling terminal. repl.rs gives nsh the pty as stdin, stdout and stderr, with cwd /tmp,
   but in Terminal::Inherited the pty is not its controlling terminal: tcgetpgrp fails with
   ENOTTY, so tty::is_interactive() is false. In such a case these never run: job control, the
   hand-off of the terminal to a foreground job and back (tty::take_terminal), and INT-286's
   lost-terminal retry. Of the 109 cases that start the REPL, 107 run Inherited; only
   repl_286_lost_terminal_is_taken_back and repl_286_terminal_never_owned_exits_74_with_a_record
   do not.
2. One prompt per line. run_session waits for one READY marker (ESC[?2004h) after each line it
   sends, then reads the window from the last ?2004l to the next 133;A. A shell that prompts twice
   for one line is cut off before it answers: on 2026-10-12 nsh printed ALIVE 1ms after the prompt
   the harness had waited for. INT-286 gave owned sessions a transcript (the reader drained until
   200ms of quiet, 2s at most); Inherited sessions still end at the first prompt.
3. Unmeasured: which of today's REPL cases change when nsh owns its terminal. The first suspects
   are the job-control cases (repl_jobs_lists_a_running_job and the repl_background_job cases):
   they pass while nsh believes it is not interactive, so their green says nothing about jobs in
   a real window.
4. Cost unmeasured: the owned-mode drain adds up to 200ms after each case's exit. Across 107 cases
   that is up to about 21s more per suite run if Owned becomes the default.

## Success Criteria

- [ ] G0 census, read-only for the repo.
      a. A throwaway git worktree of HEAD in a mktemp dir, with one change made there only: the
         session default is Terminal::Owned, so every REPL case owns its terminal.
      b. The suite run twice against the debug nsh by absolute path (INT-287 G8): from the repo
         (Inherited) and from the worktree (Owned). The wall time of each recorded.
      c. A table written into this intent: every case whose verdict or output differs, with its
         name, Inherited verdict, Owned verdict, the first differing line, and one class:
         N (an nsh defect), H (a harness defect), R (the case relied on nsh being
         non-interactive). Each R row names what it asserted that only holds without a terminal.
      d. The worktree removed, shown; git status of the repo unchanged, shown.
- [ ] G1 rulings, written here before any code, each naming the G0 rows it rests on:
      a. The default: Owned for every REPL case, or chosen per case.
      b. What Inherited still proves (nsh on a terminal it does not control), and which cases
         keep it. Each kept case says why in a comment.
      c. How the end of a line's answer is recognised: (1) the next READY, as today; (2) that
         command's own 133;D status marker, then the READY after it; (3) the transcript to exit.
         The ruling says what a case may assert from its window and what only from its
         transcript.
      d. The drain: its quiet period and cap, and how many seconds a full suite run may grow,
         from G0's measured times.
- [ ] G2 findings. Every N row filed with core intent find "<what>" --at <file:line> --by
      INT-289, its F-number written beside the row. Fixed here only if G1 rules it in;
      otherwise it waits for its own intent. Every H row is fixed in G4.
- [ ] G3 red. One new case, written before the fix, whose answer arrives after a second prompt
      (INT-286's steal is one way to cause it) and which reads the per-line window only: shown
      failing on today's harness, with the line it missed.
- [ ] G4 the harness. The G1 rulings in repl.rs; run_session stays the one owner of the session
      protocol. G3's case green. Every R case rewritten to assert what nsh does with a terminal,
      or kept Inherited with its G1b reason. Every G0 row resolved (fixed, filed, or kept with a
      reason); none left blank. No REPL case deleted to reach green.
- [ ] G5 the class, not the example. In a second throwaway worktree, tty::is_interactive() forced
      to return false in the debug nsh: the job-control cases go red there, shown. Before 289
      they stayed green. The worktree removed.
- [ ] G6 doors: one ship at the end; nsh-test all passing in a new shell, with its wall time
      inside G1d's bound; d 0 failed; core intent validate clean. The close offers nsh-test its
      bump without a hand edit (the first real use of INT-287 G2).
- [ ] G7 commits: each carries INT-289 and its plan Seal, the note under refs/notes/seals; Fixes
      lines for the findings fixed here; pushed with the pre-push hook passing; core intent
      trace reads every seal intact.
- [ ] G8 each gate carries evidence per INT-158.

## Non-goals
- No change to how nsh behaves unless G1 rules a G2 finding in. No new crates: the owned mode
  uses only what repl.rs already depends on. No sudo. Omarchy untouched.
- The REPL case count does not drop: a case is rewritten or kept, never deleted to get green.

## Dependencies
depends_on: 287 (its G8 makes NSH_BIN absolute, which G0 needs). Feeds 290, whose red cases need
an owned terminal, and 277.

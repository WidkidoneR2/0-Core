---
id: 289
date: 2026-10-13
type: future
title: "nsh-test runs the REPL without job control, so a green case does not show how nsh behaves in a real window"
status: in-progress
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

- [x] G0 census, read-only for the repo.
      a. A throwaway git worktree of HEAD in a mktemp dir, with one change made there only: the
         session default is Terminal::Owned, so every REPL case owns its terminal.
      b. The suite run twice against the debug nsh by absolute path (INT-287 G8): from the repo
         (Inherited) and from the worktree (Owned). The wall time of each recorded.
      c. A table written into this intent: every case whose verdict or output differs, with its
         name, Inherited verdict, Owned verdict, the first differing line, and one class:
         N (an nsh defect), H (a harness defect), R (the case relied on nsh being
         non-interactive). Each R row names what it asserted that only holds without a terminal.
      d. The worktree removed, shown; git status of the repo unchanged, shown.
<!-- demonstrated 2026-10-16 at HEAD b0decf00. Debug nsh by absolute path (INT-287 G8); every run stored
into a scratch ZERO_STATE_DB, and the real nsh_test_results table stayed at 363196 rows.
a. Two throwaway worktrees of HEAD under mktemp: wt-owned with the run_session_banner default at repl.rs:522
   set to Terminal::Owned, wt-inherited untouched. Both carried the same throwaway session recorder (case,
   terminal, tcgetsid on the pty after the first prompt, nsh pid, commands, status, exit, lines, transcript).
   Each was built with cargo build -p nsh-test into its own target dir; the two binaries were hashed and the
   run refuses if they are equal.
b. Four interleaved suite runs, I O I O: Inherited 41.1s and 41.4s, Owned 68.6s and 68.4s, all 239/239.
   Owned costs about 27s per run, about 210ms per session: the 200ms drain at repl.rs:728-736.
   Each mode ran as named: the I runs had 129 default sessions with no controlling terminal (tcgetsid -1),
   the O runs 130 sessions with nsh holding it (tcgetsid = nsh pid) and none Inherited.
   INVALID RUNS, kept as the record: the first two censuses compared the Inherited harness with itself.
   Both worktrees shared one CARGO_TARGET_DIR, cargo kept the first build (the Owned one built in 0.1s),
   and every Owned-run session was Inherited. The recorder's tcgetsid count exposed it; per-worktree
   target dirs and the binary-hash guard now refuse it.
c. Cases whose verdict or per-line output differs between Inherited and Owned, the same in both runs of
   each mode: none (109 cases with sessions compared line by line, 239 verdicts).
   Unstable within one mode, two cases:
   - repl_jobs_lists_a_running_job | P | P | only the pgid in the jobs line changes run to run | not a row:
     the census compared a number the case does not assert.
   - repl_286_lost_terminal_is_taken_back (always Owned) | P | P | 1 of 4 runs ended exit None | class H,
     plus an N finding for G2.
     H: a byte timeline over 30 runs of the case alone (teardown 3000ms) shows that when nsh takes its
     terminal back it prints a second ready marker, and the per-line wait (repl.rs:691) takes that stale
     marker as the answer to the next line: echo ALIVE and then teardown's exit are each sent one prompt
     early, in 30 of 30 runs. This is the intent's problem 2, for G3 and G4.
     N: the exit typed ahead while nsh answers echo ALIVE is never echoed and nsh never exits, in 7 of 30
     runs (5 of 30 at 500ms and 12 of 30 at 3000ms in a separate run), although the bytes were written.
     A longer teardown does not change it. Cause not established: nsh does not flush input when it takes
     the terminal back (tty.rs:95-97), and rustyline 17.0.2 enters raw mode with TCSADRAIN
     (src/tty/unix.rs:1622). The case stays green because it asserts only ALIVE (main.rs:2908).
   No R rows: the job-control cases print the same lines with nsh owning its terminal.
d. Every worktree removed (git worktree list shows only the repo) and git status unchanged after each
   run. Record and scripts: ~/.cache/zero/int289-g0/ (table.md, run-*.txt, sessions-*.tsv,
   repl286-repeat.md, repl286-timeline.md), kept until 289 closes. -->
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

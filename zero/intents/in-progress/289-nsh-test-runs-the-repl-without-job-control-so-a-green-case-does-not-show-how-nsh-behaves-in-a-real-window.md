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

Written 2026-10-16 at a break; it supersedes the filing note kept below. Open a new chat with
`ints 289` and this section pasted.

STATE: G0 to G3 ticked with evidence. Commits: 488c0ebe G0, 03576147 G1, 23c7310e G2 (F-0046,
F-0047), and the commit that writes this. The tree is clean. The G3 red case is not in the tree:
the red and the fix commit once (AGENTS), so it was taken out for the break.

FIRST, A RULING, because G1 c says it comes back before any code. Measured at 23c7310e with Owned as
the default: 160 of 168 REPL lines get their own 133;D before their ready marker. The 8 without are
7 continuation lines in multiline_for_loop, multiline_if_block and multiline_while_loop, which end
at the "  ... " continuation prompt and never get a D, and repl_286's echo ALIVE, which is F-0046.
Proposed: a line the case submits as a continuation ends at its ready marker; every other line ends
at its own 133;D and the ready marker after it. Check first which runner those three cases use and
whether it already knows which lines are held. Census: ~/.cache/zero/int289-g0/d-census.md.
Also measured: for the line after a take-back, 133;D arrives in 30 of 30 runs and ALIVE comes before
or with it (repl286-d-marker.md), so ruling c holds for complete commands.

NEXT, in order:
1. The ruling above, written into G1 c.
2. Re-apply the red: python3 ~/.cache/zero/int289-g3red-25e510fd9dec.py shows it (expect Seal
   822c06f38105e2a1 on an unchanged main.rs); apply it and see it red again on the debug build.
3. G4 in repl.rs per G1 a to d and that ruling: Owned default, one explicit Inherited echo case, a
   line ends at its D and then the ready marker, drain 50ms with the 2s cap. The red goes green and
   every G0 row is resolved. One commit for red and fix: the fix plan's Seal and note, the red seal
   cited in the body, Fixes: F-0046.
4. G5 (is_interactive forced false in a worktree: the job-control cases go red), G6 (one ship, suite
   wall time at most 52s, d 0 failed, validate, the close offers nsh-test its bump), G7, G8.

LEARNED this session:
- Worktrees that share one CARGO_TARGET_DIR reuse each other's build (same relative paths): give
  each its own target dir and hash the binaries. That mistake made the first two G0 runs invalid.
- A paste line over about 2.5k characters can arrive cut: send parts with a length check first.
- Red runs use a scratch ZERO_STATE_DB, so the banner and the results history stay true.
- The int287 doors script is gone; ledger-only commits use the three-door line (nsh-test, core
  doctor run, core intent validate). The doors script never could run the d alias.
- Keep until 289 closes: ~/.cache/zero/int289-g0/, int289-g3red-25e510fd9dec.py and
  int289-g3-red.plan.txt.

Superseded 2026-10-16, kept as filed:
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
- [x] G1 rulings, written here before any code, each naming the G0 rows it rests on:
      a. The default: Owned for every REPL case, or chosen per case.
      b. What Inherited still proves (nsh on a terminal it does not control), and which cases
         keep it. Each kept case says why in a comment.
      c. How the end of a line's answer is recognised: (1) the next READY, as today; (2) that
         command's own 133;D status marker, then the READY after it; (3) the transcript to exit.
         The ruling says what a case may assert from its window and what only from its
         transcript.
      d. The drain: its quiet period and cap, and how many seconds a full suite run may grow,
         from G0's measured times.
<!-- ruled 2026-10-16. a, c, and d's quiet period and cap by Christian; b and d's bound decided by Claude
at Christian's instruction to make the best decisions.
a. Owned for every REPL case: the run_session_banner default (repl.rs:522) becomes Terminal::Owned.
   Rests on G0c: no case's verdict or per-line output changes with the mode (109 cases with sessions,
   both runs of each mode), so the switch hides nothing, and Owned is how nsh runs in a real window.
b. Inherited still proves one thing: nsh answering on a pty that is its stdio but not its controlling
   terminal, as when a script or another program starts it. One plain echo case keeps Terminal::Inherited
   explicitly, with a comment saying why, so that path and the Inherited branches of run_session_full
   stay exercised. repl_286_terminal_never_owned_exits_74_with_a_record (NotOwned) keeps covering a
   terminal nsh does not own. Rests on G0c: no R rows, so no other case needs Inherited.
c. The end of a line's answer is that command's own OSC 133;D status marker, then the ready marker
   (ESC[?2004h) after it; a ready marker alone no longer ends a line. A case asserts on a line's window
   only after its D; what nsh prints after the teardown exit is read only from the transcript. Rests on
   G0c's H row: a stale second ready marker after a take-back answered the next line in 30 of 30 runs.
   G3 first shows that nsh emits D for the line after a take-back; if it does not, c comes back for a
   ruling before any code.
c, amended 2026-10-17, approved by Christian. The ruling above stands except for held lines.
   A line the case declares held (it opens or continues a multi-line construct, so no command
   completes) ends at its ready marker, the shell's continuation prompt. A held line that produces a
   133;D is a harness error. A line not declared held that reaches a ready marker without its own
   133;D keeps waiting, and fails at 30s. The case declares; the runner never infers it from the
   prompt text, because taking a ready marker for an answer is the mistake F-0046 records.
   Rests on the G0 census: of 168 lines 160 have their own 133;D. The 8 without are 7 held lines
   (multiline_for_loop 2, multiline_if_block 2, multiline_while_loop 3) and echo ALIVE in
   repl_286_lost_terminal_is_taken_back (F-0046), which this ruling ends.
   Read 2026-10-17: run_repl_multiline (repl.rs:321) is Incremental, which waits for a ready marker
   after every line (repl.rs:688-691). Its doc comment (repl.rs:309-311) says every line but the
   last is sent without waiting; the code does not do that, and G4 corrects the comment. Held lines
   are named per case, not taken from position: in multiline_while_loop the first line `n=0`
   completes a command. Buffered delivery (quote, heredoc) writes the block with no wait per line
   (repl.rs:688-690); only its last line ends as ruled above.
d. The drain: quiet period 50ms (from 200ms), cap 2s unchanged. A full suite run may grow by at most 10s
   over the Inherited baseline of 41.1s to 41.4s (G0b), so 52s at most; G6 checks the measured wall time
   against it. Rests on G0b: Owned with the 200ms drain cost about 27s per run, about 210ms per session. -->
- [x] G2 findings. Every N row filed with core intent find "<what>" --at <file:line> --by
      INT-289, its F-number written beside the row. Fixed here only if G1 rules it in;
      otherwise it waits for its own intent. Every H row is fixed in G4.
<!-- evidence: 2026-10-16, filed with core intent find --by 289 (usage at engine domains/intent/findings.rs:163).
- H row, repl_286_lost_terminal_is_taken_back, the stale ready marker: F-0046, at
  zero/shell/nsh-test/src/repl.rs:691. Fixed in G4 under ruling c; that commit carries Fixes: F-0046.
- N row, input typed ahead after a take-back is lost: F-0047, at unknown (cause not established, filed
  the way F-0031 was). G1 did not rule it in, so it waits for its own intent.
No other rows: G0c found no mode difference and no R rows, and repl_jobs_lists_a_running_job was
noise in the comparison, not a row. -->
- [x] G3 red. One new case, written before the fix, whose answer arrives after a second prompt
      (INT-286's steal is one way to cause it) and which reads the per-line window only: shown
      failing on today's harness, with the line it missed.
<!-- evidence: 2026-10-16. Red plan Seal 822c06f38105e2a1 (text: ~/.cache/zero/int289-g3-red.plan.txt)
adds repl_289_answer_after_a_second_ready_marker_is_in_the_window to nsh-test main.rs after the
INT-286 cases. The command prints its own ESC[?2004h, sleeps 0.3s, then echoes ANSWER289; the case
reads the per-line window only (run_repl_lines). On today's harness, debug build and debug nsh:
239/240, the case failing with: ANSWER289 missing from ["3 commands" job line, "[1/3] printf ..."],
so the window ended at the marker the command printed, before its answer. Output kept in
~/.cache/zero/int289-g0/g3-red-suite.txt; results went to a scratch db. The steal was not used for the
red: its stale marker comes before the answer in only 23 of 30 runs (measured), so a red built on it
would be flaky. The case is re-applied by its Seal and commits with the G4 fix, per AGENTS. -->
- [x] G4 the harness. The G1 rulings in repl.rs; run_session stays the one owner of the session
      protocol. G3's case green. Every R case rewritten to assert what nsh does with a terminal,
      or kept Inherited with its G1b reason. Every G0 row resolved (fixed, filed, or kept with a
      reason); none left blank. No REPL case deleted to reach green.
<!-- evidence: 2026-10-17, plans: red 822c06f38105e2a1, harness 8df6819a6338efe3, fmt 861a2773eff3601d.
repl.rs: run_session_full is still the one owner; run_session_banner defaults to Terminal::Owned and
stays an Err when nsh never prompts; a line ends at its own 133;D then the ready marker; held lines
are declared per case through InputDelivery::Held and run_repl_multiline(cmds, held, env) (for [0,1],
if [0,1], while [1,2,3]); a held line that produces a 133;D is a harness error; drain 50ms, cap 2s.
G3's red case repl_289_answer_after_a_second_ready_marker_is_in_the_window was watched failing
(ANSWER289 missing) and is green after the harness. The one Inherited case is
repl_289_inherited_terminal_still_answers (G1b reason in its comment); no R rows existed (G0), so no
other case needed it. G0 rows: the H row (F-0046) is fixed here; the N rows were filed in G2 and the
case that shows N keeps asserting only ALIVE. Hang detection: green when the shell is alive and
silent, red when that error was swallowed (runs A and B, ~/.cache/zero/int289-g5/).
Suite on the debug build, scratch db: 241 of 241 passed in 51.4s after the harness; 243 results,
All tests passed, 49.9s after the G5 cases. 239 cases before 289, plus the red case, the Inherited
case and the two G5 cases is 243, so no case was deleted. cargo build clean, cargo fmt --check clean. -->
- [x] G5 the class, not the example. In a second throwaway worktree, tty::is_interactive() forced
      to return false in the debug nsh: a job-control case goes red there, shown. Before 289 no
      case did except repl_286_lost_terminal_is_taken_back. The worktree removed.
<!-- evidence: 2026-10-17, plan a4694d655eefc8cf. Amended wording, same day: the gate said "the
job-control cases go red"; the run showed the existing repl_jobs cases stay green on a nsh without
job control (run C, ~/.cache/zero/int289-g5/), so the gate now says what was measured. Two cases were
added that read what a foreground child is born with: repl_289_foreground_child_leads_its_own_process_group
and repl_289_foreground_child_gets_default_job_signals. Run d-N (normal nsh): 0 failed. Run d-F
(is_interactive forced false): 3 failed, both new cases and repl_286_lost_terminal_is_taken_back.
Outputs: run-d-N.txt, run-d-F.txt in ~/.cache/zero/int289-g5/. Built in a throwaway worktree with its
own CARGO_TARGET_DIR; the forced-false binary differed from the repo's debug nsh (hash guard); the
worktree was removed and git worktree list shows only the repo. -->
- [x] G6 doors: one ship at the end; nsh-test all passing in a new shell, with its wall time
      inside G1d's bound; d 0 failed; core intent validate clean. The close offers nsh-test its
      bump without a hand edit (the first real use of INT-287 G2).
<!-- evidence: 2026-10-17. One ship: nsh-test 2.0.4 shipped, 16 unchanged, 0 failed; the deployed
~/.local/bin/nsh and target/release/nsh have the same sha256 (78f8616e031065dc...). Reloaded with
exec /home/christian/.local/bin/nsh, then in that new shell: nsh-test 243 results, All tests passed,
wall 49.5s, inside the 52s bound (G1d set it on a 239-case baseline; this run has 243 cases).
d: Passed 26, Warnings 2, Failed 0; the two warnings are Git Repository (uncommitted work, until G7) and Update Readiness (INT-280).
core intent validate: All 326 intents valid. git worktree list: only /home/christian/0-core.
The last clause, the close offering nsh-test its bump, can only be shown by cicomplete; it is
recorded under G8 after the close. -->
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

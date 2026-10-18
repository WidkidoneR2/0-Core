---
id: 290
date: 2026-10-13
type: future
title: "nsh dies on SIGHUP: a closed window skips the exit path, saves nothing and leaves no record"
status: complete
tags: [nsh, sighup, signals, exit-path, jobs, login, resilience, finding]
depends_on: [289]
---

## START HERE

Filed 2026-10-13 from INT-286 G0's scenario B. Begin at G0 after INT-289 is complete: its cases
need nsh to own its terminal, which 289 makes the harness do. Bears directly on INT-277: once nsh
is the login shell, every closed window takes this path.

## Vision

A closed window ends nsh the way bash ends: its jobs are told, what must be saved is saved without
touching the terminal that has gone, and it ends reporting the hang-up, so whatever started it can
tell why it stopped.

## The Problem

Measured 2026-10-12 (INT-286 G0, scenario B): closing the window sends nsh SIGHUP. nsh has no
handler for it, so the default action kills the process. The exit path at the end of repl_main
never runs (the session save, the journal summary, INT-286's record in nsh-exit.log), and the
parent sees signal 1, not an exit code. Once INT-277 makes nsh the login shell, every closed
window and every logout takes this path.

Where it meets INT-286: once the window is gone, a read on the terminal fails with EIO. A handler
that only sets a flag and returns can let rustyline's read come back with that EIO and land in
INT-286's arm, which tries up to 3 times to take the terminal back and then exits 74 with a
gave_up record. A hang-up must not be reported as a lost terminal; G1 rules where the flag is
checked so the two endings stay distinct.

bash, the control (from its manual; G0 measures it): on SIGHUP an interactive bash resends SIGHUP
to every job, running or stopped (stopped ones get SIGCONT first), saves its history, and exits
by re-raising SIGHUP. A job marked with disown -h is spared. huponexit applies only to a login
shell's normal exit, not to a hang-up.

Not yet measured, for G0:
- which steps of nsh's exit path write to the terminal. Rust's println! panics when the write
  fails, so a hang-up path that prints would panic instead of exiting;
- whether nsh already saves history per command (INT-191 writes shell_history as commands run),
  and so what a hang-up actually loses;
- what ghostty sends when a window closes, to which processes, and whether SIGKILL follows;
- whether nsh's background jobs (their own process groups, stdin /dev/null) outlive the window.

## Success Criteria

- [x] G0 census, read-only.
      a. bash as the control, in a ghostty window: a HUP trap logging to a file in a mktemp dir,
         a background sleep 300 & with its pid written down, then the window closed. Recorded:
         the signals bash received, the job's fate, whether the session's history was saved.
      b. nsh 5.3.2 on the same steps: the job's fate, whether nsh-exit.log gained a line (it
         should not today), and what shell_history holds for the session's last command.
      c. Who gets what: a small python3 signal logger, started once in the foreground and once
         as a background job from each shell, shows which signals reach each and when.
      d. The exit path read: every step after repl_main's loop, in order, with file:line, each
         marked T (writes to the terminal) or F (files and database only).
      e. The signal owner: how nsh already handles SIGINT, SIGTSTP and SIGCHLD, named with
         file:line, so the hang-up handler joins it without a new crate.
<!-- evidence: demonstrated 2026-10-17. pty census, bash control vs deployed nsh 5.3.2, closing the master: bash died by signal 1, HUPed its jobs and saved history; nsh died by signal 1 in the bg, fg and live-kill modes, nsh-exit.log grew 0 bytes, the bg job survived (python3 pg6951 alive 1s later), the fg job got SIGHUP from the kernel, shell_history held the fg command. Exit path read at main.rs 3154-3251 and session.rs 81-123; signal owner at commands/mod.rs 6938 and tty.rs 69-74, libc already a dependency. Not measured here: whether ghostty follows with SIGKILL (G4). -->
- [x] G1 rulings, written here before any code:
      a. Which G0d steps run on a hang-up. A T step is skipped, or made to tolerate a failed
         write; none may panic.
      b. Jobs: sent SIGHUP (SIGCONT first to stopped ones), as bash does, or left alone; and
         whether nsh's disown, if it has one, spares a job.
      c. The ending: the exit record gains a hang-up line (same file and format as INT-286's),
         then SIGHUP is re-raised with its default action so the parent sees signal 1; or exit
         129 instead.
      d. The handler sets an atomic flag and nothing else. The flag is checked when readline
         returns, before INT-286's EIO arm, so a hang-up never counts as a lost terminal; and
         when a foreground job ends.
      e. A hang-up while a foreground job runs: the job gets its own SIGHUP from the kernel; nsh
         waits for it to end, then takes the hang-up path.
      RULED 2026-10-17 (Christian approved), from the G0 census and the exit-path read:
      a. Runs on a hang-up: the journal daily-summary, SessionMemory::save and the
         session_patterns insert (main.rs 3155-3198, all F). Skipped: the session summary and
         the Project 0 remembers line (main.rs 3200-3243, T). No println! or eprintln! on this
         path; the record function never writes to stderr; mark() is not called on it.
      b. Jobs: a new JobTable method sends SIGCONT then SIGHUP to each live job group, as bash
         does. It prints nothing and waits for nothing (kill_job prints and waits, so it is
         not reused). nsh has no disown, so none is spared and none is added.
      c. Ending: one line in nsh-exit.log, same format as INT-286 with error=SIGHUP; then
         SIGHUP is re-raised with its default action so the parent sees signal 1. Not 129.
      d. libc::signal plus a static AtomicBool, no new crate. The flag is checked inside the
         inner readline loop (the EIO arm gains a not-hung-up guard), before match read_result,
         and at the top of the 'repl loop: readline returning Ok, Eof or Err, and a
         foreground job ending, all reach the hang-up path and never the lost-terminal arm.
      e. A hang-up during a foreground job: the job gets its own SIGHUP from the kernel; nsh
         waits for it, then takes the hang-up path (the loop-top check).
<!-- evidence: Christian approved these rulings 2026-10-17 before any novashell code; written from the G0 census recorded above. -->
- [x] G2 red. New nsh-test cases with an owned terminal (INT-289), each shown failing on 5.3.2:
      repl_290_hangup_runs_the_exit_path: nsh at its prompt, the harness closes the pty master;
      asserts the ending per G1c and the hang-up line in nsh-exit.log.
      repl_290_hangup_reaches_jobs: sleep 300 & first, its pid captured; after the close the job
      is gone, or still running, per G1b.
      repl_290_hangup_is_not_a_lost_terminal: the record says hang-up, never gave_up=74.
      SessionEnd reports a terminating signal as well as an exit code, if it does not already.
<!-- evidence: demonstrated 2026-10-17 on the deployed nsh 5.3.2 (NSH_BIN=/home/christian/.local/bin/nsh), /tmp/red290.txt: repl_290_hangup_runs_the_exit_path 167ms (exit None, signal 1, record empty), repl_290_hangup_reaches_jobs 2182ms (the sleep survived), repl_290_hangup_is_not_a_lost_terminal 169ms (record empty); 243 of 246 pass, only these three red. The first run was red for the wrong reason (5s, signal 9: the shell inherited the pty master); fixed with FD_CLOEXEC on the master in hang-up mode, plan seals 88f664962e4aba90 and 06b8c25914e23951. SessionEnd.signal added in nsh-test repl.rs. Not committed alone: a red suite fails the pre-push gate, so the commit rides with G3. -->
- [x] G3 fix. G2 green. The handler and flag per G1d; no step on the hang-up path writes to the
      terminal. Unchanged, each shown by its existing case or a run: Ctrl+C at the prompt and in
      a foreground job; Ctrl+D (the INT-258 health TUI); exit; INT-286's two cases.
<!-- evidence: demonstrated 2026-10-17. Red seals 88f664962e4aba90 and 06b8c25914e23951 (red on 5.3.2); fix seals 5e94ff4450936fd4 (JobTable::hangup_all, the flag, handler, record_hangup, raise_hangup, the three flag checks, the exit path), e1a37367be85de29 (the main thread blocks SIGHUP; the EXIT: start mark guarded), 5285e9df5c083cf2 (the watcher thread and IN_READLINE). Debug build: nsh-test 246/246 with repl_290_hangup_runs_the_exit_path 276ms, repl_290_hangup_reaches_jobs 284ms, repl_290_hangup_is_not_a_lost_terminal 283ms. Deployed nsh 5.3.3: nsh-test 246/246 (output kept in ~/.cache/zero/suite-290.txt; the /tmp copies of the earlier runs were cleared), including repl_286_lost_terminal_is_taken_back, repl_286_terminal_never_owned_exits_74_with_a_record, repl_289_foreground_child_gets_default_job_signals and repl_289_foreground_child_leads_its_own_process_group. The unchanged paths, each shown by a pty run on the deployed nsh at 19:19: Ctrl+C at the prompt (nsh alive, next command ran, exit 0, summary shown); Ctrl+C in a foreground job (job gone, nsh alive, next command ran, exit 0); Ctrl+D at an empty prompt (health TUI drawn, 2868 bytes, q leaves it, nsh alive, next command ran, exit 0); exit (code 0, Project 0 remembers shown). AMENDS G1d, found by a live pty test on the debug build before any ship: a bare kill -HUP at an idle prompt did NOT end nsh (alive after 8s, no record) because rustyline blocks in read(2), the SIGHUP handler is SA_RESTART, and a SIGINT raised inside the handler never interrupted the read. Now the handler only sets the flag and writes one byte to a CLOEXEC pipe; a watcher thread with every signal blocked sends SIGINT to the REPL thread, as its own event, only while that thread is inside readline and only while a SIGINT handler exists; the main thread blocks SIGHUP so the flag is set on the REPL thread before a read can fail with EIO. The same test after the fix: A idle prompt with a bg job: signal 1 in 0.04s, record written, job gone; B foreground sleep 2: signal 1 in 1.51s (waits for the job, G1e), record written, job gone; C nsh -c: signal 1 at once, default action kept. AMENDS G1a: the record is written before the F steps, not after them, so a hard kill that follows the hang-up still leaves it. -->
- [x] G4 live. The deployed nsh in ghostty, G0a's steps again: the job and the record behave as
      G1 rules, and nsh-exit.log shows the hang-up line.
<!-- evidence: demonstrated 2026-10-17 on the deployed nsh 5.3.3 in real ghostty windows. Baseline: nsh-exit.log 3 lines (all old EIO), state.db session_patterns 9129 rows, newest 16:50:17. After four window closes: four new lines ending error=SIGHUP (pids 3557, 40863, 42152, 41630, each with its own /dev/pts/N, 19:06:02 to 19:07:39), session_patterns 9133 rows with the newest recorded_at 19:07:39 (the exit path ran to its last save step on a closed terminal, so nothing on it panicked), both sleep 300.290 jobs gone, the one live nsh is the current window. On 5.3.2 the same close left no record and saved nothing. -->
- [x] G5 doors: one ship at the end; the deployed nsh tested in a new shell; nsh-test all
      passing; d 0 failed; core intent validate clean. The close offers nsh and nsh-test their
      bumps.
<!-- evidence: demonstrated 2026-10-17. One ship (18.2s): nsh 5.3.3, nsh-test 2.0.5. The deployed nsh-test against the deployed nsh in a new shell: 246/246 including the three repl_290 cases at 261, 268 and 262 ms (output kept in ~/.cache/zero/suite-290.txt and /tmp/doors290.txt). d: 26 passed, 0 failed, 2 warnings (Git Repository uncommitted changes, Update Readiness hold off on tracked changes), both of which the commit clears. core intent validate: All 326 intents valid. Filed F-0048 (the owned-terminal pty master leak in the nsh-test harness, found by INT-290). The version bumps are left to cicomplete 290 as ruled. -->
- [x] G6 commits: Intent INT-290 and the plan Seal with its note under refs/notes/seals; pushed
      with the pre-push hook passing; main and refs/notes/seals reach GitHub; core intent trace
      reads every seal intact.
<!-- evidence: demonstrated 2026-10-17. Commit 0396ce35 carries Intent: INT-290 and Seal: 5e94ff4450936fd4 and no other trailer. gp pushed main 124fade8..0396ce35 with the pre-push hook passing (55.0s); refs/notes/seals e7170134..24ca6619 pushed. core intent trace on the commit, on the seal and on INT-290: seal 5e94ff4450936fd4 intact (the plan text in refs/notes/seals hashes to it); INT-290 has one finding, F-0048, open, filed by core intent find at nsh-test repl.rs:488, named in the commit body and closed by no commit. The other plan seals are cited in the body (red 88f664962e4aba90 and 06b8c25914e23951; e1a37367be85de29, 5285e9df5c083cf2, effc23dd1bc0b983, fee0fde582c6f73a, ffbd98604c269a78); the saved texts of the first seven hash to their seals (checked 19:30). git status after the push: main level with origin/main, only the two INT-280 entries left. -->
- [x] G7 each gate carries evidence per INT-158.
<!-- evidence: demonstrated 2026-10-17. Checked by script when this plan was made: G0 to G6 each carry an evidence comment with the date, the commands or seals and the measured result. -->

## Non-goals
- No new crates. No sudo. Omarchy untouched, ghostty's config included.
- No disown, huponexit or nohup features unless G1b rules one in.
- No change to bash or to the .bashrc handoff: INT-277 owns that.
- SIGTERM and other signals are out of scope; G0 notes them if it sees them.

## Dependencies
depends_on: 289 (its owned terminal is what lets nsh-test deliver a real hang-up). Bears on 277,
which waits for this one.

## Versions
- nsh 5.3.3 -> 5.3.4 (patch)
- nsh-test 2.0.5 -> 2.0.6 (patch)

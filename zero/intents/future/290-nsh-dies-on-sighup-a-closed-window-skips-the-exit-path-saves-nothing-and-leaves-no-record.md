---
id: 290
date: 2026-10-13
type: future
title: "nsh dies on SIGHUP: a closed window skips the exit path, saves nothing and leaves no record"
status: planned
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

- [ ] G0 census, read-only.
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
- [ ] G1 rulings, written here before any code:
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
- [ ] G2 red. New nsh-test cases with an owned terminal (INT-289), each shown failing on 5.3.2:
      repl_290_hangup_runs_the_exit_path: nsh at its prompt, the harness closes the pty master;
      asserts the ending per G1c and the hang-up line in nsh-exit.log.
      repl_290_hangup_reaches_jobs: sleep 300 & first, its pid captured; after the close the job
      is gone, or still running, per G1b.
      repl_290_hangup_is_not_a_lost_terminal: the record says hang-up, never gave_up=74.
      SessionEnd reports a terminating signal as well as an exit code, if it does not already.
- [ ] G3 fix. G2 green. The handler and flag per G1d; no step on the hang-up path writes to the
      terminal. Unchanged, each shown by its existing case or a run: Ctrl+C at the prompt and in
      a foreground job; Ctrl+D (the INT-258 health TUI); exit; INT-286's two cases.
- [ ] G4 live. The deployed nsh in ghostty, G0a's steps again: the job and the record behave as
      G1 rules, and nsh-exit.log shows the hang-up line.
- [ ] G5 doors: one ship at the end; the deployed nsh tested in a new shell; nsh-test all
      passing; d 0 failed; core intent validate clean. The close offers nsh and nsh-test their
      bumps.
- [ ] G6 commits: Intent INT-290 and the plan Seal with its note under refs/notes/seals; pushed
      with the pre-push hook passing; main and refs/notes/seals reach GitHub; core intent trace
      reads every seal intact.
- [ ] G7 each gate carries evidence per INT-158.

## Non-goals
- No new crates. No sudo. Omarchy untouched, ghostty's config included.
- No disown, huponexit or nohup features unless G1b rules one in.
- No change to bash or to the .bashrc handoff: INT-277 owns that.
- SIGTERM and other signals are out of scope; G0 notes them if it sees them.

## Dependencies
depends_on: 289 (its owned terminal is what lets nsh-test deliver a real hang-up). Bears on 277,
which waits for this one.

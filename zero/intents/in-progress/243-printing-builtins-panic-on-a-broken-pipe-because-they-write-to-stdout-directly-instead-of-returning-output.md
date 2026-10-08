---
id: 243
date: 2026-09-05
type: fix
title: "printing builtins panic on a broken pipe because they write to stdout directly instead of returning Output"
status: in-progress
tags: [fix, bugfix, nsh, pipeline, builtins, sigpipe]
depends_on: []
---

## Vision
Piping any command into `head` ends the pipeline. It does not kill the shell and
it does not panic.

## The Problem
Found 2026-09-05 while proving an unrelated fix:

```
nsh -c "dash forest" | head -4
->  thread 'main' panicked at library/std/src/io/stdio.rs:1166:9:
    failed printing to stdout: Broken pipe (os error 32)
```

`head` exits after four lines and closes the pipe; the next `println!` panics.

### ⚠️ THIS IS INT-299's SYMPTOM, RETURNING BY A DIFFERENT ROUTE
INT-299's comment records the original: *"`ls ~/path | head -5` would previously
panic with 'failed printing to stdout'"*. Its arch-era fix was a process-wide
`signal(SIGPIPE, SIG_DFL)`, which traded a visible panic for a SILENT FATAL
SIGNAL -- and that was corrected on 2026-08-21 by removing the process-wide
reset and restoring `SIG_DFL` per child in `spawn_pipeline`'s `pre_exec`.

⭐ **THAT FIX ASSUMED THE SHELL NEVER WRITES INTO A CLOSED PIPE**, because both
pipeline stages are spawned as real children with real pipes. **A PRINTING
BUILTIN BREAKS THAT ASSUMPTION**: it writes to stdout directly with `println!`,
inside the shell process, so there is no child to take the signal.

### The connection to the structured pipeline
`peel_builtin_first_stage` handles a builtin at the head of a pipeline by taking
its `CommandResult::Output(text)` and feeding it in on a thread with
`let _ = write_all(...)` -- EPIPE-safe by construction. But a builtin that
PRINTS rather than returning `Output` never reaches that path; it falls into
`Peeled::Finished` and its bytes go straight to stdout.

⭐ So this is the **same root cause as `history | head` dropping its pipe**
(fixed 2026-08-21 by giving tables a `to_pipe_text`): builtins split into those
that RETURN text and those that PRINT it, and the printing ones are outside the
pipeline machinery. That fix made table commands return `Value`. This one is
about the commands that still print.

### Scope
Any builtin that writes with `println!` is a candidate. `dash forest` is
confirmed. The census is the first gate because the count decides whether the
answer is per-command or structural.

## The Solution
Two candidate shapes, and the choice is the intent's real content:

**(a) Make printing builtins return text.** Consistent with the `to_pipe_text`
work and makes them pipeable as a side effect. ⚠️ Large: every printing builtin
changes shape, and some print incrementally by design.

**(b) Handle EPIPE at the write boundary.** Smaller, but it is a guard rather
than a fix, and it leaves the two classes of builtin permanently different.

⚠️ **DO NOT REINTRODUCE A PROCESS-WIDE SIGPIPE RESET.** The 2026-08-21 work
established that the shell must IGNORE SIGPIPE (so writes return EPIPE and can
be handled) while children get `SIG_DFL` restored in `pre_exec`. Both halves are
load-bearing: without the per-child restore, `yes | head -3` spins forever.

## Success Criteria
- [x] G1 RED FIRST, BOTH DOORS: `nsh -c "dashboard" | head -4` is run and its panic
      captured verbatim, and the control is re-run -- `yes | head -3` stops,
      `ls ~ | head -5` does not panic. Then `dashboard | head -4` is run in the
      interactive REPL (the door fsh-test drives, INT-173) and the result recorded as
      observed: by reading, a printing builtin returns Empty, which
      peel_builtin_first_stage pipes on as empty text (commands/mod.rs:9843), so the
      expected symptom there is a dropped pipe rather than a panic -- recorded, not
      assumed
      <!-- DISCHARGED BY EXPLANATION 2026-10-07 (the INT-213 G1 precedent). The panic did not
      reproduce: nsh -c "dashboard" | head -4 exited 141 with nothing on stderr, and so did the
      deterministic variant (nsh -c "dashboard" with its stdout already closed). Cause: commit
      3e4ecfbe (2026-09-15) added a panic hook that turns the EPIPE print panic into a silent
      exit(141) (novashell main.rs:840-849), ten days after this intent was filed. Controls:
      yes | head -3 printed three lines and exited 0; ls ~ | head -5 printed five, no panic.
      REPL door, typed one line at a time at a top-level prompt: dashboard | wc -l printed the
      whole dashboard to the terminal and then 0 -- the pipe is dropped, not crashed. That
      defect moved to INT-281. -->
- G2 EVERY builtin that writes to stdout directly instead of returning Output
      is ENUMERATED -- println!, print!, and any write to an io::stdout() handle -- by
      fsearch across builtin code paths. The count decides (a) versus (b)
      MOVED to INT-281 under the 2026-10-07 re-scope -- not a gate here.
- G3 THE RULING between (a) and (b) is recorded here with its reason
      MOVED to INT-281 under the 2026-10-07 re-scope -- not a gate here.
- [x] G4 `nsh -c "dashboard" | head -4` completes without panic and without killing
      the shell, on the DEPLOYED binary
      <!-- DEMONSTRATED 2026-10-07 on the deployed binary (~/.local/bin/nsh): exit 141, no
      stderr, and the calling shell kept running. The fix (3e4ecfbe, 2026-09-15) predates this
      gate, so its failure could not be watched first; G7's regression test keeps it fixed. -->
- [x] G5 THE CONTROLS STILL HOLD: `yes | head -3` still terminates its child,
      and no process-wide SIGPIPE reset has returned. `grep` proves the second
      <!-- DEMONSTRATED 2026-10-07: nsh -c "yes | head -3" printed y three times and exited 0,
      inside a 10-second timeout that never fired. Read across zero/shell: the only SIGPIPE
      signal call that is code rather than comment is libc::signal(libc::SIGPIPE,
      libc::SIG_DFL) inside the per-child pre_exec at commands/mod.rs:9585. The search was
      python over the source, standing in for grep. -->
- [x] G6 A NESTED-SHELL test: the failure originally took the PARENT shell down
      too, so the fix is verified from inside a child nsh
      <!-- DEMONSTRATED 2026-10-07 in a sacrificial window, typed one line at a time: nsh
      started a child shell (echo level $SHLVL printed level 2); inside it
      nsh -c "dashboard" | head -4 printed four lines and the child's prompt returned;
      echo child still here answered; exit printed the session summary and returned to the
      parent, where echo level $SHLVL printed level 1. The grandchild's exit 141 took
      neither shell down. -->
- [ ] G7 Regression tests in nsh-test, beside regression_sigpipe_no_crash, for at
      least one printing builtin piped into head, through BOTH doors: run_fsh
      (nsh -c) and a Category::Repl case
- [ ] G8 each gate carries evidence per INT-158

## Non-goals
- Making every builtin pipeable. That is the structured-pipeline work; this is
  about not crashing.
- Revisiting INT-299's original decision. It is already corrected.

## Revision 2026-10-07

Recon before cistart. `dash forest` no longer reproduces as written: forest appears
nowhere in novashell after the rename, and dash would collide with any dash binary on
PATH, which peel_builtin_first_stage spawns instead of the builtin
(commands/mod.rs:9788). The gates now use dashboard, the canonical name
(builtin_names.rs:68, dispatched at commands/mod.rs:1383).

The Problem section's Peeled::Finished is stale since the history fix: a printing
builtin returns Empty, which is piped on as empty text (commands/mod.rs:9843), so
inside a pipeline its bytes bypass the reader rather than crash. The panic needs nsh's
own stdout to be the closed pipe -- the nsh -c case. G1 and G7 now cover both doors
(INT-173).

By reading, the 2026-08-21 SIGPIPE arrangement holds: the only signal call is SIG_DFL
in the per-child pre_exec (commands/mod.rs:9585). G5 still demonstrates it after the fix.

## Re-scope 2026-10-07 -- scope (a), ruled by Christian

G1 found the panic already gone: commit 3e4ecfbe (2026-09-15) made nsh exit 141 silently
on a broken pipe. What remains is keeping it gone, so this intent now locks that behaviour
in: the nested-shell check (G6) and regression tests through both doors (G7).

The dropped pipe G1 observed in the REPL (dashboard | wc -l prints 0) is a different
defect. It moved to INT-281 together with the census and the return-text-versus-guard
ruling (old G2 and G3).

Separately, a terminal window closed instantly while a child nsh received three entered
lines. It did not reproduce typed singly at a top-level prompt, its cause is unknown, and
it is recorded as a finding (core intent find) rather than folded in here.

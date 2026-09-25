---
id: 243
date: 2026-09-05
type: fix
title: "printing builtins panic on a broken pipe because they write to stdout directly instead of returning Output"
status: planned
tags: [fix, bugfix]
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
- [ ] G1 RED FIRST: the panic is captured verbatim, and the three-case control
      is re-run -- `yes | head -3` stops, `ls ~ | head -5` does not panic,
      `dash forest | head -4` panics
- [ ] G2 EVERY builtin that writes to stdout with `println!` instead of
      returning `Output` is ENUMERATED. The count decides (a) versus (b)
- [ ] G3 THE RULING between (a) and (b) is recorded here with its reason
- [ ] G4 `dash forest | head -4` completes without panic and without killing the
      shell, on the DEPLOYED binary
- [ ] G5 THE CONTROLS STILL HOLD: `yes | head -3` still terminates its child,
      and no process-wide SIGPIPE reset has returned. `grep` proves the second
- [ ] G6 A NESTED-SHELL test: the failure originally took the PARENT shell down
      too, so the fix is verified from inside a child nsh
- [ ] G7 Regression tests in nsh-test for at least one printing builtin piped
      into `head`
- [ ] G8 each gate carries evidence per INT-158

## Non-goals
- Making every builtin pipeable. That is the structured-pipeline work; this is
  about not crashing.
- Revisiting INT-299's original decision. It is already corrected.

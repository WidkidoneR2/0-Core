---
id: 188
date: 2026-07-21
type: feature
title: "fsh job control: Ctrl+Z / bg / fg via the nix crate -- verify what fsh already has first"
status: planned
tags: [fsh, job-control, signals, nix, tokio, process-groups]
---

## Vision
Real job control in fsh: suspend a running command with Ctrl+Z, resume it in the background (bg) or
foreground (fg), and track a job table. The nix crate provides the syscall primitives real job control
needs -- signals (SIGTSTP/SIGCONT), process groups (setpgid), terminal control (tcsetpgrp).

## VERIFY-FIRST -- do NOT assume fsh lacks this (golden rule)
The FIRST gate is to check what fsh already does. fsh may already handle SOME of this (Ctrl+C is handled;
job control may be partial or absent). Filing "add job control" without checking risks rebuilding what
exists (the redundant-intent trap that killed 3 would-be intents in the 2026-07-20 session). Read the
signal/process handling in exec.rs + main.rs BEFORE scoping any build.

## Why it matters
Job control is a core interactive-shell capability. Without it, a long-running command can't be backgrounded
mid-run, and Ctrl+Z either does nothing useful or misbehaves. For a daily-driver shell this is a real gap
IF it's missing. But it is INTERACTIVE-PATH work -- like the line editor, a bug here degrades the terminal,
so it wants care (relates to INT-168 reedline's keystroke handling; sequence sanely with it).

## Sequencing note
Job control touches signal handling and the terminal foreground process group -- the same territory the line
editor (INT-168 reedline) owns. Coordinate: don't build job control on top of rustyline right before swapping
to reedline. Likely AFTER 168 stabilizes, or carefully alongside.

## Success Criteria
- [ ] VERIFY-FIRST: document what fsh's current signal/job handling actually does (Ctrl+C, Ctrl+Z, any
      backgrounding). Read exec.rs + main.rs signal paths. Scope only what's genuinely missing.
- [ ] Gate zero: is job control actually absent/broken, or partially present? If already adequate, CANCEL.
- [ ] If it proceeds: the nix-crate primitives it needs (SIGTSTP/SIGCONT, setpgid, tcsetpgrp) named + a job
      table design.
- [ ] Ctrl+Z suspends, bg resumes in background, fg brings to foreground -- demonstrated on a real command.
- [ ] No regression to the line editor or Ctrl+C. fsh still boots, logs in, deploys.
- [ ] Each gate carries evidence per INT-158.

## The Rule
"A daily-driver shell should background a job. But first: read what fsh already does -- the golden rule
killed three redundant intents last week, and this is exactly the kind that needs the check." 🌲

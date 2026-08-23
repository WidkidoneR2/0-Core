---
id: 188
date: 2026-07-21
type: feature
title: "fsh has no job control layer: no process groups, no terminal foreground ownership, and one signal handler"
status: planned
tags: [fsh, job-control, signals, process-groups, terminal, int-207]
---

## Vision
fsh owns its signals, its process groups and the terminal foreground -- so a job can be suspended,
resumed, backgrounded and reported on, and the shell's idea of what its children are doing matches
what they are actually doing.

## ✅ VERIFY-FIRST: DONE 2026-08-23, and it CHANGED THE SCOPE
The intent's own first gate demanded this before any build, and it was right to. Measured across
`faelight-shell/src/`, non-comment lines. **THREE signal touchpoints in the entire shell:**

    libc::signal(SIGPIPE, SIG_DFL)   mod.rs:9253    one reset at startup
    ctrlc::set_handler               mod.rs:6708    ONE handler, ONE signal
    status.signal()                  main.rs:729    READING a dead child's signal as 128 + s

**ZERO references to SIGCHLD, SIGTSTP, SIGCONT, SIGWINCH, SIGTTIN, SIGTTOU, SIGHUP.
ZERO setpgid. ZERO tcsetpgrp. ZERO tcgetpgrp. ZERO waitpid.**

★ SO GATE ZERO IS ANSWERED, AND NOT THE WAY THE INTENT EXPECTED. Job control is not partial. There
is no process-group handling and no terminal-foreground control anywhere in fsh. Ctrl+Z cannot work,
a stopped job cannot be resumed, and a terminal resize reaches nothing.

⚠️ THIS IS NOT A DEPENDENCY TASK. The original title says "via the nix crate" and the tags name
tokio -- a crate chosen before the shape. **THE MISSING ABSTRACTION IS THE DEFECT, NOT THE MISSING
DEPENDENCY.** Adding a crate that exposes the right signals would leave every hard question
unanswered.

## The Problem
A shell is a process manager. fsh currently spawns children and reads their exit status, and that is
the whole of it. It does not know when a child stops, does not place children in process groups, and
never negotiates which process group owns the terminal. Every capability below follows from those
three absences, not from a missing library.

## The Solution: ONE dispatcher, then everything else
    OS signals
        |
    signal dispatcher            <- ONE owner. ctrlc is SUBSUMED, not kept beside it.
        |
    +---------+---------+
    |         |         |
  SIGCHLD  SIGWINCH  SIGINT/SIGTSTP
    |         |         |
  job      terminal   foreground
  table     state      control

★ SIGCHLD DRIVES RECONCILIATION, NOT NOTIFICATION. A signal that flips a boolean or prints a line is
the shape this ledger deleted four times on 2026-08-22 -- consumers that reported confidently from a
source that could not support it. SIGCHLD must cause the job table to ASK what actually happened and
record it.

    RUNNING --(SIGTSTP/SIGSTOP)--> STOPPED --(SIGCONT)--> RUNNING
    RUNNING --(SIGCHLD + exit)---> EXITED
    RUNNING --(SIGCHLD + signal)-> SIGNALED

⚠️ SIGWINCH IS NOT JOB STATE. It belongs to terminal and editor state, and mixing it into the job
table would be the same category error as `TIMING:` rows living in shell_history.

## Success Criteria
- [x] VERIFY-FIRST: document what fsh's current signal/job handling actually does. Scope only what is
      genuinely missing
<!-- DONE 2026-08-23, measured not assumed: three touchpoints (SIGPIPE reset, one ctrlc handler,
     one read of a dead child's signal), and zero of every primitive job control needs. The gate
     earned its place -- it changed the intent from "add a crate" to "build the layer". -->
- [x] Gate zero: is job control absent, partial, or adequate? If adequate, CANCEL
<!-- ABSENT. Not partial. No process groups, no terminal foreground control, no SIGCHLD. -->
- [ ] G1 THE DISPATCHER SHAPE IS DECIDED BEFORE ANY DEPENDENCY IS CHOSEN, and the decision is
      written here: who receives signals, how a signal becomes a shell event, how SIGCHLD drives
      AUTHORITATIVE job-state reconciliation, and how process groups and terminal foreground
      ownership interact.
      ⚠️ A DEPENDENCY IS NOT SELECTED BECAUSE IT EXPOSES THE REQUIRED SIGNALS. signal-hook, rustix,
      nix and raw libc are all capable; capability is not the criterion
- [ ] G2 ONE SIGNAL OWNER. `ctrlc` is removed or subsumed -- two mechanisms answering one question
      is the shape INT-207 and INT-221 both existed to end
- [ ] G3 THE REQUIRED SET IS HANDLED AND EACH ONE'S PURPOSE IS STATED: SIGCHLD, SIGTSTP, SIGCONT,
      SIGINT, SIGWINCH, SIGTTIN, SIGTTOU, and SIGHUP
- [ ] G4 PROCESS GROUPS: children are placed in groups deliberately, with the requirement stated --
      what gets its own group, what shares one, and what happens to a pipeline
- [ ] G5 TERMINAL FOREGROUND OWNERSHIP: the shell-versus-job foreground model is defined and
      implemented, including what happens when a background job reads from the terminal
- [ ] G6 ONLY THEN: the implementation is chosen -- signal-hook, rustix, nix or libc -- with the
      reason recorded and what it gives up
- [ ] G7 DEMONSTRATED ON A REAL COMMAND: Ctrl+Z suspends, `bg` resumes in background, `fg` returns
      it to the foreground, and `jobs` reports state that matches reality
- [ ] G8 NO REGRESSION: the line editor, Ctrl+C, login, and deploy all still work. fsh-test green
- [ ] G9 each gate carries evidence per INT-158

## Sequencing
- INT-168 (reedline) owns keystroke handling and the same terminal territory. Do not build job
  control on rustyline immediately before swapping the editor -- coordinate or sequence after.
- ⭐ THIS INTENT IS THE PREREQUISITE FOR THE IDENTIFIER SCHEME (Crockford Base32 job ids): job
  control is what makes an identifier visible and gives it a first real consumer.
- INT-207's observability has a `Jobs` target already defined and unused. Every state transition
  above should emit through it rather than growing a private log.

## Non-goals
- Choosing a crate before G1. That is the failure this rewrite exists to prevent.
- Reimplementing what the OS provides. The question is which layer owns the call, not whether to
  write a scheduler.
- SIGWINCH-driven redraw. It belongs to the editor; this intent only routes the signal.

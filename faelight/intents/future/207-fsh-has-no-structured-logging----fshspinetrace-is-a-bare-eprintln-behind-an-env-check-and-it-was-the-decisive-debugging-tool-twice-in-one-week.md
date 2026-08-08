---
id: 207
date: 2026-08-08
type: arch
title: "fsh has no structured logging -- FSH_SPINE_TRACE is a bare eprintln behind an env check, and it was the decisive debugging tool twice in one week"
status: planned
tags: [architecture, rust, design]
---

## Vision
fsh is a process manager. Spawning, process groups, job state, signals and terminal foreground
control are the things that become undebuggable without a record, and job control is the work coming
next. This intent gives the shell structured logging so a question about its own behaviour has an
answer that does not depend on adding a println and rebuilding.

## The Problem
THE CASE IS ALREADY MADE, BY USE RATHER THAN BY ARGUMENT. FSH_SPINE_TRACE is a bare eprintln behind
an env-var check -- four lines -- and it was the decisive tool twice in one week. It proved the router
claims a redirected background line, and it proved that jobs and kill are excluded as REPL-state
commands. Before it, three separate probes had been spent trying to DEDUCE which engine ran a command
and every one was ambiguous. Four lines turned "did the behaviour differ?" into "which engine owned
it?".

What it cannot do is everything else. There are no levels, so a trace is on or off. There are no
targets, so the router, the lexer, expansion, the executor and job control cannot be enabled
separately. There is no structure, so a line cannot be filtered, correlated with an execution_id, or
read by anything but a human. And each new question has meant adding another env var: FSH_SPINE_TRACE
exists, and this session added two temporary file-writing instruments beside it for the legacy
executor and the sh fallback.

⚠️ THOSE TWO INSTRUMENTS ARE THE ARGUMENT IN MINIATURE. Both were written by hand, both needed a field
added after the fact to make their rows interpretable -- a spine-state field on one, a build field on
both -- and both are marked for deletion. That is three hand-rolled observability mechanisms in one
codebase, each learning the same lessons separately.

## The Solution
Structured logging with targets and levels, so the question being asked selects what is recorded.
The crate is an implementation detail; INT-198 ruled tracing as the mechanism and ADD NOW as the
priority, but this intent owns the shape rather than the dependency.

WHAT IT MUST SUPPORT, drawn from what has actually been needed:
  which engine owned a line, and why the other declined
  spawning: program, argv, pgid, and the resulting pid
  job state transitions, which INT-188 will need at every step
  signals sent and received, and terminal foreground changes
  correlation with execution_id, so a trace line and a lifecycle row describe the same event

## Explicitly out of scope
Replacing the shell's USER-FACING output. A trace is for the person debugging the shell; a diagnostic
is for the person using it, and that is INT-208's subject. Mixing them would make the shell chatty
for everyone in order to be legible for one.

## Success Criteria
- [ ] RED FIRST: a question that cannot be answered today is written down, and answered after
- [ ] Targets exist and are independently selectable -- router, lexer, expansion, executor, jobs
- [ ] A trace line can be correlated with a command_execution row by execution_id
- [ ] FSH_SPINE_TRACE is either migrated or explicitly kept with a stated reason. Two mechanisms
      for one question is the thing this intent exists to end
- [ ] The two temporary instruments (legacy-exec.log, sh-fallback.log) are deleted or migrated,
      not left beside the new mechanism
- [ ] Nothing is chattier by default. An interactive session with no target selected looks exactly
      as it does today
- [ ] Each gate carries evidence per INT-158
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->

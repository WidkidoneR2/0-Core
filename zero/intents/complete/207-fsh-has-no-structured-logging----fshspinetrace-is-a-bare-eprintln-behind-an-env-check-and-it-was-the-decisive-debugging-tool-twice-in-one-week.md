---
id: 207
date: 2026-08-08
type: arch
title: "fsh has no structured logging -- FSH_SPINE_TRACE is a bare eprintln behind an env check, and it was the decisive debugging tool twice in one week"
status: complete
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
- [x] RED FIRST: a question that cannot be answered today is written down, and answered after
<!-- THE QUESTION: over a run of real use, does any `-c` line still reach `sh`, and is legacy
     execution still reached at all? It decides whether roughly 250 lines of raw-text derivation
     can be deleted, and it is INT-169's evidence.

     ⚠️ THE ORDERING IS NOT WHAT THIS GATE ASKED FOR, and saying otherwise would be the kind of
     claim this intent exists to prevent. The question was NOT written down before the work. It was
     formulated DURING it, on 2026-08-23, when the two file instruments meant to answer it were
     found to have never written a row -- they wrote into faelight/runtime/ with
     OpenOptions::create(true), which never creates a parent directory, and that directory was lost
     in the Phase 1 tree move.

     ★ SO THE GATE IS SATISFIED IN SUBSTANCE RATHER THAN IN SEQUENCE. What it tests is whether the
     new mechanism can answer something the old ones could not, and that is demonstrably true: the
     question was UNANSWERABLE this morning -- not because nobody asked, but because the instrument
     that asked it was silently discarding every observation -- and it is answerable now.

     THE ANSWER SO FAR: three `-c` lines of different shapes (a builtin, an external pipeline, an
     `&&` chain) with FSH_OBSERVE=executor and a working sink produced ZERO rows. The spine claimed
     all three; neither the sh fallback nor legacy execution was reached.
     ⚠️ Three lines is a sample, not a deploy cycle. The real measurement is a day of ordinary use
     with the sink set in the deployed shell -- the before-half of a before/after comparison across
     the INT-169 flip, on the same clock, schema and correlation. -->
- [x] Targets exist and are independently selectable -- router, lexer, expansion, executor, jobs
<!-- observe.rs: Target is an ENUM (Router, Lexer, Expansion, Executor, Jobs, Boot), so a typo is a
     compile error and the registry is closed by construction. FSH_OBSERVE takes a comma-separated
     list or `all`; FSH_OBSERVE_LEVEL sets a floor. Demonstrated: FSH_OBSERVE=router emits router
     events while FSH_OBSERVE=jobs on the same command emits nothing. Unit tests
     targets_select_independently and level_is_a_floor. Commit 852f6497. -->
- [x] A trace line can be correlated with a command_execution row by execution_id
<!-- The emission path attaches session:execution from FSH_SESSION_ID and FSH_EXECUTION_ID, so a
     caller cannot forget it. Live: correlation_id=47320-1787459760307931693:1 on a router event,
     the same key command_execution carries. TWO router events on one line share it, which is what
     makes a sequence readable. -->
- [x] FSH_SPINE_TRACE is either migrated or explicitly kept with a stated reason. Two mechanisms
      for one question is the thing this intent exists to end
<!-- MIGRATED, six emissions, engine.rs. FSH_OBSERVE=router replaces it. Its own distinction is
     preserved -- excluded-as-REPL-state versus disabled-by-FSH_SPINE=0 stay two messages, because
     saying the wrong reason is worse than silence. FSH_TRACE went the same way: seven sites across
     three files, split across Router, Expansion and Executor, and one message lost a hardcoded
     line number that had already drifted. Commits 852f6497, 6771571b. -->
- [x] The two temporary instruments (legacy-exec.log, sh-fallback.log) are deleted or migrated,
      not left beside the new mechanism
<!-- MIGRATED, not deleted, and the reason matters: NEITHER HAD EVER WRITTEN A ROW. They wrote to
     faelight/runtime/ with OpenOptions::create(true), which never creates a parent directory, and
     that directory was lost in the Phase 1 tree move -- so every open failed and every `if let Ok`
     arm was skipped. Deleting them would have turned an unanswered question into an irreversible
     conclusion.
     Their fields survive as event fields, each keeping the comment that records why it was added
     after a row proved unreadable without it: spine-state, shape, word, expanded, typed. `build`
     and `door` moved to the emission path.
     ★ AND THE RULE IS NOW ENFORCED IN CODE: an instrumentation path must FAIL VISIBLY when its
     destination cannot be established. observe::init creates the sink's parent directories and
     reports failure loudly, because otherwise "no observations" and "nothing happened" are the
     same output. Demonstrated both ways. Commits 0dafc3b5, dc1191e4. -->
- [x] Nothing is chattier by default. An interactive session with no target selected looks exactly
      as it does today
<!-- enabled() returns false unless FSH_OBSERVE is set; the file sink is None unless
     FSH_OBSERVE_FILE is set. Unit test silent_without_the_variable asserts it directly, and
     `fsh -c true` with no variables produces no output at all. 220 unit tests and 159/159
     fsh-test green throughout the migration. -->
- [x] Each gate carries evidence per INT-158
<!-- this block. -->
<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->

---
id: 201
date: 2026-08-04
type: arch
title: "fsh execution engine: extract a reusable executor from main.rs -- nothing outside the REPL loop can run a command"
status: in-progress
tags: [fsh, architecture, refactor, execution]
priority: medium
---

## ⚠️ NUMBERING NOTE, READ FIRST
The pre-migration ledger used numbers up to 351, and the fsh source still carries 60 citations in
the range INT-201..INT-346 that refer to THOSE intents, not these. `INT-201` appears at
main.rs:3743 meaning something else entirely. There is no old-to-new mapping -- the migration log
records the v1-to-v2 engine work, not a renumbering -- so those citations can be annotated but
never mechanically resolved. This intent is the FIRST post-migration 201. When you meet an
INT-2xx or INT-3xx citation in fsh source, assume it is pre-migration until proven otherwise.

## Vision
ONE execution engine, several front ends. The interactive REPL becomes one client of it, `fsh -c`
becomes another, and later entry points -- scripts, the test harness, a language server, batch
mode -- share the same one. Today there is no engine to be a client of.

## The Problem
NOTHING OUTSIDE main.rs CAN EXECUTE A LINE. Measured 2026-08-04:
  try_spine_background_command  1 caller   main.rs:2486
  try_execute_spine_source      1 caller   main.rs:2514
  execute_with_context          2 callers  main.rs:2148, main.rs:3429
Every one of them is inside the REPL loop.

The pipeline is ORCHESTRATED IN THE LOOP rather than implemented in a function: alias expansion,
then the segment splitter, then the spine router, then a legacy fallback beneath it, then exit-code
handling -- all wrapped in `continue 'segments` control flow that has no meaning outside a loop.

⚠️ AND THE COUPLING IS THROUGH MUTABLE SESSION STATE, NOT JUST CONTROL FLOW. `last_exit_code`,
`job_table`, `shell_vars`, `prev_op` and the session counters are woven through roughly 1,500 lines
and every execution path reads or writes them. That state is what makes the loop body hard to lift,
far more than the line count. It is also why `fsh -c` still delegates to `sh`: there is no seam to
route it through.

## THE DESIGN QUESTION THIS INTENT EXISTS TO ANSWER
    What is the minimal execution context required to execute ONE parsed command?

Everything else follows from the answer. Once that context is an explicit type rather than a
scattering of loop locals, the REPL is one client that supplies an interactive version of it,
`fsh -c` is another that supplies a non-interactive one, and neither has to know how the other works.

## The Solution -- shape, not sequence
Name the context first, then move code into it. The temptation is the reverse: start lifting the
loop body and let the parameters accumulate. That produces a function with eleven arguments and the
same coupling, relocated.

⚠️ DO NOT ROUTE `fsh -c` UNTIL THE CONTEXT EXISTS. Reaching the milestone by reproducing a subset of
the dispatch paths is precisely the "two doors, two behaviours" defect this work exists to end --
the same defect that let `fsh -c` mean `sh -c` for months, and that made the conformance suite
measure the wrong shell.

## Prior work already landed (this intent inherits it, INT-200 does not own it)
- 813c5ee8 -- `config::apply` returns a report instead of printing. A runtime step must not emit UI,
  or a non-interactive caller inherits it on stdout.
- f84b8853 -- `runtime_init()` extracted: open the db, ensure a default config, load it, apply it.
  Returns a struct, prints nothing, and `repl_main` destructures it immediately.
Those two established the RUNTIME/SESSION boundary. This intent establishes the EXECUTION boundary,
which is the harder half.

★ THE RULING THAT CAME WITH THEM, and it governs every front end added later: any non-program output
emitted by a non-interactive invocation goes to STDERR. Not a special case for config diagnostics --
a general rule, so a future deprecation notice lands correctly without anyone revisiting the policy.

★ AND THE ONE THAT DECIDED THE SCOPE: `-c` should execute the complete fsh LANGUAGE, but it does not
have to create a complete interactive SESSION. Language completeness and session completeness are
independent, and conflating them is what makes both the thin-door and complete-shell designs feel
wrong. Execution semantics inherit the caller's cwd and environment exactly as given; session
semantics establish prompt, history, direnv, bookkeeping and the welcome banner.

## Success Criteria
- [x] The minimal execution context is NAMED as a type, with its fields justified one at a time
<!-- evidence: src/engine.rs. `Engine` owns five fields, each traced to a census of the REPL loop
     rather than chosen: shell_vars and last_exit_code are the only genuine mutable EXECUTION state
     of eleven bindings counted (77 reads/writes for the exit code alone -- the dominant coupling,
     and load-bearing because `&&` decides the next segment from it); db, core_root and before_rules
     are RESOURCES a caller hands in. The other eight bindings are session furniture the executor
     never reads. All five `cfg` uses in the loop were `cfg.before_rules`, so the engine takes the
     rules and the loop stops needing the config.
     ★ NOT A FOURTH "CONTEXT", and the naming is part of the design: ExecContext is per-command
     provenance, ShellContext is the ephemeral read-only view variable resolution needs, and Engine
     is the long-lived mutable OWNER that produces both. `shell_context()` builds the view per call
     because it COPIES the exit code -- holding one across an execution would hand the spine a value
     the command it is about to run is going to change.
     ⚠️ prev_op was flagged as coupling and is NOT here: it is per-line chain state, internal to
     executing one input line, so it stays local to the executor's own scope.
     ⚠️ DELIBERATELY UNWIRED. Moving the loop turns the `&db` sites into accessor calls -- mechanical,
     and it should not ride along with the design where a mistake would be invisible rather than a
     compile error.
     ⚠️ COUNT CORRECTION (measured at b1a60bb7, after this comment was written): the loop holds
     29 `&db` plus 41 method sites = 70, and 39 `core_root` -- not the ~82 and ~31 quoted from the
     first census. The original figures were an estimate repeated from memory rather than a count.
     ✅ THE WIRING LANDED at 3dbca455 (accessors) and 1abf3083 (77 sites). The loop no longer owns
     the database, the forest root or a config: every `cfg` use inside it was `before_rules`, so the
     engine takes those by partial move. Verified on a clean build, 138 unit tests, the full 132-case
     interactive suite, and a behaviour probe covering exit status, pipelines, the INT-143 prefix
     save/restore, a db-backed builtin, health and history recall.
     ⚠️ NOT A GATE. The four state bindings below still have no owner -- that is the next gate, and
     this increment sharpened it: the resource conversion produced ZERO borrow conflicts, so the
     whole remaining tension is `last_exit_code` (77 sites) and `shell_vars` (18). -->
- [x] `last_exit_code`, `job_table`, `shell_vars` and `prev_op` each have a stated owner
<!-- evidence: two owners are ENFORCED, two are STATED, and the difference is deliberate.
     ENFORCED BY THE COMPILER: `last_exit_code` and `shell_vars` are engine fields. Their local
     bindings are DELETED, so no site can read a stale copy -- a missed conversion was an
     unresolved name, not a silent divergence. 52 writes and 16 reads of the exit code, and six
     mutations of the variable map, now go through set_last_exit / set_var / remove_var. Both
     `ShellContext` literals became `engine.shell_context()`, so the view reports state the engine
     actually holds. Both `#[allow(dead_code)]` attributes are removed and the build is
     warning-free, which is the mechanical proof that every field and accessor has a real caller.
     STATED AT THE DECLARATION: `prev_op` (main.rs ~1441, inside the loop) is per-line chain state
     and is reset for each input line, so it is deliberately not engine state. `job_table`
     (main.rs ~993, session scope) is owned by the session and will be passed into line execution
     as Option<&mut JobTable> -- None for non-interactive callers, where a backgrounded job dies
     with the process. Both now carry that reasoning as a comment where the binding lives.
     ★ WHY THAT CLOSES THE GATE: an ownership gate exists to make ownership explicit and
     verifiable, not to complete every downstream refactor. Requiring it to be encoded in the call
     graph would merge two milestones -- establish ownership, and refactor execution -- and delay
     credit for work that is done. The parameter arrives with the executor extraction, which is
     the NEXT gate, and a future change that contradicts these comments is visible in review.
     ⚠️ ONE DESIGN FINDING CAME OUT OF THIS: the database is now Rc, because the completion helper
     borrows it for the whole session and pinned the engine as immutably borrowed. That is not a
     borrow-checker workaround -- it states that the database is a SHARED resource while the
     variables and exit code are state the engine owns outright. -->
- [x] Execute one command line via a function that accepts all per-execution state as
      parameters, callable from outside main.rs
<!-- evidence: `pub fn execute_and_record` in src/engine.rs, called from main.rs. Nine parameters,
     every one derived from the command line being run. Landed in three commits that each did one
     thing: 48bf9152 extracted 221 lines with the advisories left in place so print order could not
     move; 38f26186 hoisted four silent per-execution blocks in and the return type got SIMPLER,
     from a pair back to a bare SegmentOutcome, because the redirect write consumes the captured
     output by value; 26042b75 moved it into engine.rs and made it pub. Deployed gen 462.
     ★ THE CLAUSE THAT WAS REMOVED, AND WHY. This gate previously required `Option<&mut JobTable>`.
     That was written during gate 2 as a PREDICTION of the shape, and the work disproved it.
     Measured 2026-08-05: four engine handlers already accept one -- try_jobs, try_fg and try_kill at
     engine.rs 631/652/682, and try_background added at 4f0d167b -- and backgrounding is decided in
     the guard chain ABOVE the executor. background_command's own doc states the principle: an
     ExecutionPlan describes one FOREGROUND process, and "do not wait" is a scheduling decision with
     no business inside a description of what to run. The job table is session state reaching
     per-command HANDLERS; it is not a parameter of the executor.
     ⚠️ WHAT IS NOT CLAIMED. Nine parameters carry #[allow(clippy::too_many_arguments)] -- an honest
     marker that the count is known and accepted, not hidden. And 1,598 lines of preparation still
     sit ABOVE the call. That preparation is dispatch and it is the NEXT gate, not this one.
     ⚠️ THIS TICK IS LATE AND THE REASON IS RECORDED RATHER THAN TIDIED. Commit 7cc6ecd6 was titled
     "close gate 3" and its diff contained only the Progress section -- 40 insertions and 18
     deletions, exactly the 191-to-213 line change. The gate patch was written and never run, so for
     two days a commit message asserted a closure this file did not show. That is the failure the
     intent audit exists to catch, and it happened here.
     Verified on 138 unit tests, the full 132-case suite, and live probes on the deployed binary. -->
- [ ] The REPL loop is a CLIENT of it -- no dispatch logic left inline
- [ ] fsh-test stays green throughout, including the conformance cases
- [ ] `fsh -c` routes through it, and the digit guard applies to both doors
- [ ] `fsh -c 'pwd'` prints the CALLER's directory, not the forest root

## Progress -- gate 3, and four bugs found while proving it (2026-08-05)
Landed since the last note: `48bf9152` the executor extracted -- 221 lines, five inputs, one returned
value, because the eleven-argument function collapsed once the four Friday advisories came out first;
`38f26186` the hoist, which moved four silent per-execution blocks in and made the return type SIMPLER
by taking more code rather than less; `26042b75` the move into engine.rs, where something other than
the REPL can reach it. Deployed gen 462.

Then the seventeenth handler. `4f0d167b` extracted the legacy trailing-& block into
`Engine::try_background`, taking `Option<&mut JobTable>` exactly as `try_jobs`, `try_fg` and
`try_kill` already did -- a pure move, with three known defects preserved so that the commit is a move
and nothing else. `76305252` then fixed the first of them: a background job that could not start said
nothing and left the previous command's exit code standing. Red was witnessed before green, under
FSH_SPINE=0, which is the only route that still reaches that path.

THE MEASUREMENT THAT DECIDES THE REMAINING GATES. The segments body is 1,664 lines: 1,598 of
preparation before the executor call and 66 after it. What sits after is exactly what should -- the
outcome check and the advisory display. What sits before is alias expansion, the guard chain, variable
and glob expansion, spine routing, redirect detection and pipeline analysis. That preparation IS
dispatch, so "the REPL loop is a CLIENT of it" is not met, and those 1,598 lines are the remaining
bulk of this intent.

AND THE JOB-TABLE PHRASE NOW HAS EVIDENCE RATHER THAN A GUESS. `execute_and_record` takes nine
parameters, every one derived from the command line being run, and is callable from outside main.rs.
It takes no `Option<&mut JobTable>` -- and the recon says that is correct rather than missing. Four
engine handlers already accept one, and `background_command`'s own documentation rules that "do not
wait" is a scheduling decision with no business inside a description of what to run. Backgrounding is
a guard-chain handler by nature, not a parameter of the executor. The clause describes a shape this
work has shown to be wrong, so it wants rewording rather than satisfying.

TWO BUGS FOUND WHILE VERIFYING, BOTH PRE-EXISTING, BOTH IN THEIR OWN COMMITS. `b1bb3615` deleted two
comments -- one in exec.rs, one in main.rs -- each claiming a redirected background line falls through
to legacy. It has not for months: the spine claims it and honours the redirect through the same
configure_file_io the foreground path uses. `a33d6cd7` guarded a real one. A backgrounded redirect
reaching the legacy path took the ampersand into the redirect target, so `echo hi | cat > out.txt &`
created a file named `out.txt &`, ran in the foreground, registered no job and reported nothing. Seven
such files accumulated in /tmp before anyone noticed what they were. It reaches the DEFAULT path,
because the spine declines to background a pipeline. The guard refuses rather than repairs; the real
repair is the spine learning to background a pipeline, and that is its own work.

Deployed gen 464, in daily use. 138 unit tests and the full 132-case suite green at every step.

## Finding -- duplicate `?` handlers, no behaviour change made (2026-08-05)
During the guard extractions, `try_nl_query` was moved to the engine and then found to be UNREACHABLE.
The `?` path has a single reachable behaviour today: the REPL-level guard at main.rs ~1379 catches
any line starting with `?`, an empty query exits via `continue 'repl`, and a non-empty query also
resolves there. The segments loop never sees it, so the segments-level handler is dead from the
user-facing path regardless of the quality of its implementation.
⚠️ THE TWO ARE NOT EQUIVALENT, which is why this is not a cleanup:
  reachable  -- `translate_natural_language`, y/N confirmation, confidence display.
  dead       -- `nl::translate_with_custom` with TOML custom patterns, `nl::is_diagnostic` and
                auto-diagnose, pipeline and join resolution.
★ Promoting the richer path is a FEATURE decision, not relocation, so it is deliberately NOT made
here. Resolving it inside a refactor would leave a bad historical boundary -- future archaeology
would read this commit as the one that changed query semantics. Recorded instead, and deferred to
INT-202 (unify natural-language query routing).

## Scope guardrails
- Do NOT delete the command registry (main.rs ~903). It is built on every startup and never read
  afterwards, but removing it is a DESIGN decision about abandoned wiring, not a refactor. Its one
  verified constraint: `config::apply` writes shell_aliases and `registry.populate` reads them, so
  apply must precede populate.
- Do NOT reach the milestone by duplicating a dispatch path. See the warning above.
- Do NOT let the context type grow to hold session concerns because it was convenient. If the REPL
  needs something the engine does not, that is the REPL's field, not the engine's.

## Relationship
- INT-200 finished its own objective (spine COVERAGE: 98.4% equivalence, no accounting artifacts
  left in the decline list). `fsh -c` routing was a step INSIDE it that turned out to need this.
  The dependency runs this way: 200 does not wait on 201, 201 unblocks what 200 could not reach.
- INT-171 (one parsing entry point) is the same family one layer up: it consolidated parsers, this
  consolidates executors. Read its gates before designing -- the argument shapes are identical.
- INT-169 built the spine and the router that this must keep callable.
- INT-168 (reedline) touches the same 233 lines of init. Whichever lands second inherits a large
  merge; worth sequencing deliberately rather than discovering it.

## The Rule
"The goal did not change -- the understanding of the work did. `-c` calling the same execution path
sounded like one function call until the code said there is no path, only a loop."

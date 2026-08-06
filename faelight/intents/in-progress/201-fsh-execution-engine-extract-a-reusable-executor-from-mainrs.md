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
- [ ] The REPL performs no execution dispatch. Every entered command is delegated to the execution
      engine, and the ENGINE selects the executor -- shell or query -- and runs it. Language routing
      is an engine concern, not a REPL concern.
<!-- ★ REWORDED 2026-08-05 ON A DESIGN RULING, not on difficulty. The original asked for "no dispatch
     logic left inline", which reads as "there is exactly one executor" -- and the reconnaissance
     above shows that is not reachable by deletion, because the spine cannot parse fsh's own query
     language and the forest query executor has to survive.
     ★★ THE DISTINCTION THAT DECIDED IT (Christian): the redirect and pipeline executors exist
     because the shell parser could not yet absorb those paths -- HISTORICAL boundaries, so they go.
     The query executor exists because it runs a DIFFERENT LANGUAGE -- a REAL boundary, so it stays,
     as a first-class executor owned by the engine. Forcing two genuinely different grammars through
     one parser adds more complexity than it removes.
     ★ The abstraction boundary survives intact and is arguably cleaner: the REPL performs I/O, the
     engine decides how execution happens, executors implement languages. Responsibilities now line
     up with language boundaries instead of with the order things were built.
     ⏭ WHAT THIS GATE NOW REQUIRES, in order:
       1. The redirect executor goes. Nothing needs folding -- the spine already claims all nine
          redirect forms probed. What remains is its diagnostic: `echo a >` is DECLINED and legacy
          prints the caret. Make MissingRedirectTarget SURFACE rather than decline, per "refusals
          fall back; defects surface" -- a missing target is the user's mistake, not a construct the
          spine declines to own.
       2. The pipeline executor goes AFTER the spine can background a pipeline. It is reachable and
          wrong today (532880e1 guards it), so deleting it first moves that line from wrong to
          unhandled.
       3. The query executor MOVES into the engine as a named executor. It is not deleted and not
          apologised for.
     ⚠️ A PREDICTION THIS RULING MAKES, worth testing separately: the digit guard exists because one
     parser had to serve two grammars. Under clean two-language routing the shell parser never sees
     `cpu > 0.5`, so `echo test > 0.5` could become an ordinary redirect -- retiring BOTH declared
     divergences in the conformance corpus. Its own change, its own evidence. -->
- [ ] fsh-test stays green throughout, including the conformance cases
- [ ] `fsh -c` routes through it, and the digit guard applies to both doors
- [ ] `fsh -c 'pwd'` prints the CALLER's directory, not the forest root

## The design gate 4 will be built to (decided 2026-08-06, before any code moved)
THE BOUNDARY. The REPL owns terminal interaction and session lifetime. The engine owns language
semantics. Everything in the segment loop is measured against that one sentence.

TWO ENTRY POINTS, NOT ONE.

    run_input(&mut self, input: &str, jobs: &mut JobTable) -> InputOutcome
    run_segment(&mut self, segment: &str, jobs: &mut JobTable) -> SegmentOutcome

run_input takes one complete command line, splits the boolean chain, and dispatches each segment.
run_segment is the internal primitive: everything the loop body does today.

WHY THE SPLIT BELONGS TO run_input. `&&`, `||` and `;` are language. If the REPL performs that split
it still contains part of the language implementation, and the gate would be satisfied on paper by a
layer that still parses.

JobTable DOES NOT MOVE INTO Engine. It is passed as `&mut`, exactly as try_jobs, try_fg, try_kill and
try_background already take it. Gate 2 ruled it session state that lives in the loop, and that ruling
stands: this change is about moving RESPONSIBILITIES, not about changing state ownership.
Relitigating ownership inside a refactor is how a refactor becomes a rewrite.

THE ADVISORIES ARE DELIBERATELY OUT OF SCOPE. Predictions, consecutive-failure detection, trigger
evaluation and the Friday daemon event sit below the executor and are logically adjacent to it -- but
they affect observable output ORDERING, and gate 3's extraction already learned to leave them in
place for exactly that reason. Relocating them is an independent refactor afterwards, so any
behavioural change is isolated and easy to validate.

WHAT THE LOOP ACTUALLY HOLDS, measured at 1,238 lines after the three executor deletions:
  LANGUAGE    the flow/skip decision, let/export inline assignment (~260 lines), heredoc, alias
              expansion, expand_vars, expand_subshells, expand_globs, detect_redirect, the split
  ROUTING     the spine background attempt, the router, the two legacy refusals
  DISPATCH    eleven engine.try_* handlers, the query executor, four job-control handlers,
              execute_and_record
  SESSION     job_table
  ADVISORIES  the tail
Only the last two are the REPL's. Everything above them is the engine's by the gate's own wording.

THE OUTCOME TYPE NEEDS THREE VARIANTS, and the reason is not the advisories themselves. The control
flow already has three distinct semantic outcomes, and today they are encoded by WHERE a
`continue 'segments` happens to sit rather than by anything a type says:

    Executed    execution completed and the post-execution phase should run
    Handled     execution was fully handled and that phase should be skipped
    ExitShell   the shell should exit

Compressing those into a boolean loses information. Thirty-three `continue 'segments` sites in this
loop skip the advisory tail today; only the fall-through past execute_and_record reaches it. A
run_segment returning a two-state outcome would either run the advisories thirty-three times where
they do not run now, or skip them where they do.

SO SegmentOutcome IS WIDENED RATHER THAN JOINED BY A SECOND RESULT TYPE. Widening preserves the
existing abstraction and turns every match site into a compile error, which is exactly the situation
where exhaustive matching earns its keep -- the same argument CommandResult made when it chose to
widen instead of adding a variant that eighteen catch-all arms would have swallowed.

WHERE THE BOUNDARY FALLS, AND WHY. run_segment ends where execution semantics are complete and the
advisory phase begins. That is a clean architectural seam on its own terms: everything above it
decides what to run and runs it, everything below it reacts to what happened. It is NOT chosen to
keep the parameter list short -- that three advisory bindings (the session command count, the shown
suggestions, the last Friday intent) then stay out of the signature is VALIDATION of the seam, not
the reason for it. Pulling the advisories across merely to make the extraction larger would be
choosing a boundary for the wrong reason.

SEQUENCE. First run_segment as a straight extraction, with the REPL still driving the loop -- the
same "move code, not dispatch" pattern that worked for the query executor, and provable by the suite
alone because nothing about behaviour is supposed to change. run_input comes later and is NOT a
small follow-on: roughly four hundred and fifteen lines sit between the outer loop and the segment
loop -- comment stripping, history expansion, normalisation, brace expansion, the `?` guard, the
pty_exec paths and the segment split -- and that is a refactor of its own, once the lower layer has
settled.

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

## Progress -- gate 4 reconnaissance: the loop holds FOUR executors, not one (2026-08-05)
Reading the region between the spine router and the executor call answered a question nobody had
asked plainly. The router is about twenty lines. The other six hundred and sixty are three more
executors, each of which runs a command and then continues the segment loop:

  a  redirect executor          main.rs ~2372-2572   ~200 lines
  b  external pipeline executor main.rs ~2613-2859   ~247 lines
  c  forest query executor      main.rs ~2222-2319   ~97 lines

So execute_and_record is the FOURTH executor, not the only one, and the remaining preparation is
three parallel executors plus roughly a hundred and fifty lines of genuine preparation.

THE FIRST MOVE WAS REACHABILITY, NOT EQUIVALENCE, and it was the cheap question. With the router
trace on, nine forms were probed under default routing -- simple redirect, append, `2>`, `2>&1`,
combined, stdin, two- and three-stage pipelines, and a pipeline into a file. All nine were CLAIMED by
the spine. Legacy's `__stderr__` sh-delegation is therefore dead for the ordinary `2>` case: INT-172
restored it when the spine had no stderr model, and the spine has IoPlan and StderrTarget now. The
three thousand history rows the migration audit skips as stderr-delegated describe the legacy PARSER,
not today's routing.

⚠️ SO DELETION IS A REACHABILITY ARGUMENT RATHER THAN AN EQUIVALENCE ONE -- but it is not free, and
two costs are now named rather than waiting to be discovered.

FIRST, EXECUTOR (a) OWNS THE BEST ERROR MESSAGE IN THE REDIRECT PATH. `echo a >` is declined by the
spine and lands on the `__redirect_error_no_target__` branch, which prints a source-span diagnostic
with a caret under the offending `>` and exits 2. Deleting the executor takes that with it. Any
deletion plan must say where it moves to, or accept losing it.

SECOND, EXECUTOR (b) IS REACHABLE AND WRONG, WHICH BLOCKS ITS OWN DELETION. `echo hi | cat &` was
declined and fell through to it, where the pipeline split handed the last stage the ampersand as an
argument: `cat: '&': No such file or directory`, foreground, no job. `sleep 4 &` was claimed and
registered correctly, so the boundary is exactly "the spine will not background a pipeline".
Commit 532880e1 guards it -- refuse with a message rather than half-run -- on the same reasoning as
this morning's redirect guard: repairing legacy would mean building a second pipeline executor beside
the spine's.

    THE SPINE BACKGROUNDING A PIPELINE IS A PREREQUISITE FOR DELETING EXECUTOR (b), not a parallel
    improvement. Deleting it today moves that line from wrong to unhandled.

AND EXECUTOR (c) IS A KEEP, WHICH RAISES THE QUESTION THIS GATE CANNOT ANSWER ALONE. The spine cannot
parse fsh's own query language -- the migration audit counts roughly four hundred and twenty rows of
`tt | where deployed == true`, `ps | where cpu > 0.5 | sort cpu desc`, `select * from ps where cpu >
1` -- nor the trigger DSL. `ps | where cpu > 0.5` is declined by the router and served by (c), which
works correctly. So "the REPL is a client of one executor" is not reachable by deleting things.

    THE FORK, AND IT IS A DESIGN DECISION RATHER THAN A REFACTOR: either the spine LEARNS the query
    language, which makes "one AST every path routes through" literally true and is its own intent,
    or the engine hosts TWO executors BY DESIGN -- the spine for shell syntax, the query executor for
    typed pipes -- because they are genuinely two languages, and pretending otherwise is what made
    `where cpu > 0.5` need a digit guard in the first place. Neither is chosen yet, and this gate
    cannot close until one is.

## Progress -- the query executor moves, and the redirect diagnostic changes doors (2026-08-05)
Two commits against gate 4, each proving one thing.

`23a6e306` gave the router a type that can describe what it did. Option<CommandResult> could say "ran
it" and "not mine" and had no way to say "owned it, reported it, nothing to run" -- which is what a
parser diagnostic is. Every route to preserving the existing caret box ran aground on that, because
absorb_result prints an Error with a red x and miette's box already opens with its own marker. So
SpineOutcome names the three states after OWNERSHIP rather than after today's case: Executed carries
a result, Handled means the spine claimed the input and established the status, Declined means legacy
may try. MissingRedirectTarget now surfaces instead of declining, and render_redirect_error_at takes
the parser's span instead of rescanning the line for the offending operator. Refusals fall back;
defects surface -- a missing redirect target is a mistake the parser already located, not a construct
the spine declines to own. The proof was three things at once: byte-identical diagnostic, exit code
still 2, and the trace printing "handled" where it printed "declined".

`96b6ea94` moved the query executor into the engine as try_query_executor. Ninety-three lines leave
the loop; the call site stays exactly where the block was, so this commit moves code and not
dispatch. It is named for the question -- does the query language own this line -- rather than for
the forest-pipeline check that answers it today.

TWO DEFECTS TRAVELLED WITH IT, RECORDED RATHER THAN FIXED, so the move has no semantic delta. The
source list holds "deploys" twice. And its has_pipe is a bare contains rather than the quote-aware
form used later in the same loop, so a forest-source command with a quoted pipe in an argument routes
to the query executor wrongly. The second matters more than it looks: under the two-languages ruling
that predicate IS the language router, so it is a boundary correctness issue rather than parser
polish, and it earns its own change with its own evidence.

WHAT THE RECONNAISSANCE SETTLED ABOUT THE OTHER TWO EXECUTORS. Nine redirect and pipeline forms were
probed with the router trace on and all nine were claimed by the spine, so deletion is a reachability
argument rather than an equivalence one. But the redirect executor still owns the diagnostic path for
whatever the spine declines, and the pipeline executor is reachable AND wrong -- `echo hi | cat &`
reached it and ran cat with the ampersand as an argument until 532880e1 guarded it. So the spine
backgrounding a pipeline is a prerequisite for deleting the pipeline executor, and the redirect
executor's remaining branches need their own reachability argument.

⚠️ AND A QUESTION THAT DELETION RAISES, NOT YET ANSWERED: FSH_SPINE=0 is the only remaining route
into either executor. Delete them and the escape hatch stops handling redirects and pipelines at all
-- it becomes a diagnostic aid rather than a working shell. That is worth deciding on purpose, since
a half-working escape hatch is worse than an honest one.

## Progress -- layer 2: the spine backgrounds a pipeline (2026-08-05)
Three commits, and the last one removes a prerequisite rather than a line count.

`2e6cf27e` split spawning a pipeline from waiting on it. execute_pipeline built its chain in one
block and waited in another; the two were already separate, so making the seam a function cost
nothing and changed nothing. The doc carries the obligation the split creates: the returned children
are alive and unwaited, and dropping them leaks zombies.

`bb4adb88` used that seam. A configured Command cannot express a chain -- every stage must already
be running before the last can be handed over -- so the background door's return type grew a second
shape: Single carries a command the job table will start, Chain carries children already running with
the last one holding the status. The boundary the door exists to protect is unchanged: exec.rs spawns
and still never learns what a JobTable is; main.rs registers and still never learns how to lower.

THE PART THAT WOULD HAVE LEAKED QUIETLY. Register only the tail and every upstream stage becomes a
zombie, once per backgrounded pipeline, invisibly. The Job now holds the upstream stages beside the
status-bearing child and check_completed reaps them. Verified by counting defunct processes
afterwards -- the single one on this machine is a core subprocess with a shell parent, present hours
earlier and unrelated, which is itself a finding filed elsewhere.

AND THE GUARD FROM 532880e1 WAS KEPT RATHER THAN DELETED, which reverses the plan written when it was
added. Under FSH_SPINE=0 the line never reaches the spine, so that guard is now the legacy path's
honest answer instead of a temporary measure. It follows the contract FSH_SPINE was given: a
migration aid that says "legacy does not implement this", not a fallback shell that pretends
otherwise.

⏭ EXECUTOR (b) IS NOW DELETABLE, AND ITS DELETION IS NOT JUST A REMOVAL. Under default routing the
spine claims every pipeline, so the inline executor is dead there. Under FSH_SPINE=0 it is still
live, and deleting it without a guard would send `echo x | cat` to the single-command executor, which
would hand `echo` the arguments `x | cat` -- the same leak-into-argv failure in a new place. So the
commit is a deletion plus a legacy pipeline refusal, and FSH_SPINE=0 stops running pipelines and says
so.

## Progress -- all three legacy executors are gone (2026-08-05)
`f849fdba` deleted the pipeline executor and `257d3e92` deleted the redirect executor. Together with
the query executor's move into the engine, the four-executor finding above is answered: one of them
moved because it runs a different language, two were deleted because they existed only while the
shell parser could not absorb their constructs, and execute_and_record remains.

UNREACHABLE WAS PROVED, NOT ARGUED. Each deletion left behind a refusal that prints a distinctive
line, and the line never appeared under default routing. For pipelines: a two-stage pipe, a
three-stage pipe, a backgrounded pipe and a query pipeline all behaved exactly as before. For
redirects the suite is the proof, because its REPL cases exercise write, append, stderr, and
redirect-from-an-alias through the interactive door -- if two hundred lines of redirect handling had
still been load-bearing, they would have gone red.

THE REDIRECT EXECUTOR WAS HIDING A DEAD PARAMETER. It ended in an unconditional continue, so
execute_and_record could only ever be called with redirect: None, and the write hoisted into it at
38f26186 never ran once. A parameter that cannot vary is not a capability; it is wiring that outlived
its source, and deleting the block is what made that visible.

AND THE PROBE IT REQUIRED WENT WITH IT. try_builtin existed because the redirect path called execute()
to ask whether a line was a builtin, got Empty, concluded it was not, and spawned the command a second
time -- mkdir ran twice, a POST posted twice. It was the answer to a question only that block asked.
With the block gone it had no callers and ExecutionMode::Probe had no constructor. CommandResult::
NotBuiltin stays: the spine produces and consumes it, where it means "not a builtin, so run argv",
and the comment claiming only try_builtin could return it was already false.

main.rs went from 3,913 lines to 3,419 in one day.

⚠️ WHAT THIS COSTS, STATED RATHER THAN DISCOVERED: FSH_SPINE=0 no longer runs redirects or pipelines.
That is the contract the variable was given -- a migration aid for comparing routing, not a fallback
shell -- and the alternative was keeping two implementations alive to honour a promise nobody made,
which is how an ampersand ended up in a filename that morning. The router comment and INT-169's status
section were corrected to match, rather than being left to mislead the next reader.

## Design result -- the direct run_segment lift was explored and rejected (2026-08-06)
The extraction was attempted and abandoned. That is a result, not a failure: it answered a question
the design could not answer on paper.

WHAT STOPPED IT. A census of the loop body's control flow, taken from the file rather than from
memory, found eighteen labelled breaks, thirty-one labelled continues, and -- decisively -- seven
bare continues, three bare breaks and one bare break in expression position. A bare break or continue
binds to the NEAREST ENCLOSING LOOP, and the bare breaks prove the body contains inner loops. So some
of those bare continues belong to an inner loop and must not become a return, while others belong to
the segment loop and must. Nothing in the text distinguishes them.

THE TRANSFORMATION WAS SYNTACTIC AND THE SEMANTICS ARE LEXICAL. That is the whole finding. A
text-driven rewrite cannot prove it preserves behaviour here, however careful the substitution table,
because the meaning of the statement depends on the scope it sits in and not on the characters it is
made of. Four attempts failed for four different surface reasons; the underlying reason was the same
each time.

AND THE HISTORY SAYS THE SAME THING. Every bounded extraction landed with a reviewable diff and a
clear proof: the executor at two hundred and twenty-one lines, the query executor at ninety-three,
three legacy executors deleted with their unreachability demonstrated. The one attempt to move the
whole remaining body never reached a state where it could be validated at all. That is feedback that
the abstraction boundary is not ready, not that the move was performed badly.

WHAT THE LOOP ACTUALLY IS. Not one cohesive unit awaiting relocation -- a collection of distinct
responsibilities that happen to share a scope. The flow decision, inline assignment, the expansions,
the router, the two refusals, the derivations. Each is a responsibility; none of them is the loop.

SO THE ORDER INVERTS. Carve off the remaining inline responsibilities into handlers, one at a time,
the way the seventeen already in the engine got there. When the loop is a dispatcher over try_ calls
plus a little orchestration, run_segment stops being a twelve-hundred-line relocation and becomes
almost mechanical -- and the SegmentOutcome widening becomes meaningful at that moment, because there
is finally a boundary for it to describe. The widening was written, proved inert, and reverted for
exactly that reason: it describes a seam that does not exist yet.

## Progress -- the first inline extraction, done the way the rejected lift proved necessary (2026-08-06)
The flow decision moves to the engine and the loop keeps its continue.

MOVE LOGIC, NEVER CONTROL FLOW. A bare break or continue means whatever the nearest enclosing loop
says, so carrying one across a function boundary changes its meaning silently. Returning a bool
cannot. That is the rule the failed lift produced, applied for the first time.

THE PREDICATE IS A FREE FUNCTION AND THE METHOD DELEGATES, so its tests need no Engine and no
database. Four of them: and-runs-after-success, or-runs-after-failure, no-operator-always-runs, and
the one nobody would think to write -- a missing exit code counts as success, which is what makes a
chain work as the first command of a session. That last case is why this block was chosen first: it
is the only remaining one provable without driving a shell.

Success is still read from the exit code and nowhere else. Re-deriving it from a result variant at
another call site is the bug INT-171 gate 5 put the rule in one place to prevent.

⏭ FIVE REMAIN AND THEY ARE LARGER: let/export inline assignment at roughly two hundred and sixty
lines, the expansions, the router block, the two legacy refusals, and the base_cmd derivations. Then
run_segment is a wrapper rather than a relocation. Gate 4 is still real work; it is now predictable
work.

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

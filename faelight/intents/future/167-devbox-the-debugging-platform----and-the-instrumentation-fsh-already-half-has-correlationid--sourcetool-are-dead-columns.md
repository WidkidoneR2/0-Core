---
id: 167
date: 2026-07-16
type: arch
title: "DevBox: the debugging platform -- and the instrumentation fsh already half-has (correlation_id + source_tool are dead columns)"
status: planned
tags: [devbox, debugging, instrumentation, events, sqlite, tui, fsh, architecture]
---

## Vision
An IDE for fsh's runtime. Not a pile of debug logs -- a single interface showing parser state, AST,
active jobs, environment changes, syscalls, performance timeline, memory allocations, and recent
events side by side. When a command misbehaves, inspect the whole execution path -- raw input ->
tokenization -> parsing -> expansion -> execution -> exit status -- without switching tools.

A COLLECTION OF SMALL TOOLS, not one massive application. That is the design constraint, and it is
the right one.

## Architecture (Christian's design, 2026-07-16 -- preserved verbatim in shape)

                      +----------------------+
                      |     Your Shell       |
                      |----------------------|
                      | Parser               |
                      | Lexer                |
                      | Executor             |
                      | Job Control          |
                      | Plugin Manager       |
                      +----------+-----------+
                                 |
                    Instrumentation API
                                 |
         ------------------------------------------------
         |        |         |          |          |
     Event Log  Trace    Metrics    Profiler   Recorder
         |        |         |          |          |
         ------------------------------------------------
                                 |
                       SQLite / JSONL Storage
                                 |
                  +------------------------------+
                  | Debug UI / CLI / TUI / Web   |
                  +------------------------------+

EVERYTHING EMITS EVENTS. That is the whole idea.

### Core principles
1. EVERY SUBSYSTEM PRODUCES STRUCTURED LOGS. Not "Parser error" but
   {"time":17200001,"module":"parser","function":"parse_pipeline","line":415,"token":"|",
    "state":"ExpectCommand","message":"Unexpected pipe"}
   Structured logs become invaluable over time.
2. EVERY FUNCTION GETS TRACING. ENTER parser.parse() / ENTER lexer.next() / EXIT lexer.next() /
   EXIT parser.parse(). Eventually a tree: shell_start -> parser -> lexer -> tokenize; executor ->
   fork -> exec -> wait. Makes flow understandable.
3. EVENT RECORDING -- record absolutely everything: token created, AST node built, builtin executed,
   variable changed, alias expanded, history updated, signal received, fork(), exec(), pipe(),
   waitpid(), malloc(), panic, thread spawned. Then REPLAY the execution.

### libdebug -- one instrumentation library everything links against
trace() log() metric() event() assert() panic() record() profile_scope() snapshot()
NEVER call printf() directly. Every subsystem uses this.

Levels: ERROR WARN INFO DEBUG TRACE VERBOSE -- changeable AT RUNTIME.
Categories: PARSER LEXER EXECUTOR JOBS BUILTINS PLUGINS NETWORK FILESYSTEM MEMORY PERFORMANCE NIX
CONFIG. Then `debug --category parser` shows only parser logs.

### SQLite instead of plain logs
Events, stacks, metrics, timings, memory, crashes, warnings -- all in SQLite.
    SELECT * FROM events WHERE module='parser' ORDER BY timestamp;
is far more powerful than searching text files.

### The TUI -- htop for your shell
ratatui / crossterm. Panels: live logs, parser, executor, variables, memory, threads, performance,
status bar, input.
TIMELINE VIEW: 0ms shell startup / 2ms load config / 10ms lexer / 15ms parser / 18ms executor /
20ms builtin / 23ms prompt. Very useful for startup optimisation.

### Visualizers
AST VIEWER -- `ls -la | grep txt` renders as Pipeline -> Command(ls, -la) + Command(grep, txt).
  Save every AST; DIFF ASTs between parser versions.
LEXER VISUALIZER -- `echo $HOME` -> WORD echo / SPACE / VARIABLE HOME. Diagnoses tokenizer issues.
MEMORY TRACKER -- wrap malloc/calloc/free/realloc; track allocation, stack, owner, lifetime, module.
  Per-module allocation counts make leaks obvious.
SYSCALL RECORDER -- wrap fork/execve/pipe/dup2/waitpid/kill/open/close/read/write. Output the real
  sequence: fork() -> PID 4212 -> execve("/usr/bin/ls") -> pipe() -> dup2(5,1) -> waitpid(...).
  Invaluable for shell development.
ENVIRONMENT SNAPSHOTS -- after each command save cwd/PATH/HOME/variables/functions/aliases/jobs/
  history/options. Then COMPARE snapshots.
COMMAND RECORDER -- input, tokens, AST, expansions, execution, output, exit code, time. Then replay.
PLUGIN DEBUGGING -- per plugin: execution time, memory, variables changed, functions added, hooks
  registered, errors.
CRASH REPORTER -- on crash save stack trace, logs, memory, registers, loaded modules, environment,
  config, recent commands, recent events. One compressed archive.
PERFORMANCE PROFILER -- parser, lexer, executor, prompt, autocomplete, history, config, startup.
  Flame graphs or timelines.

### Build configuration (NixOS -- separate debug features into build profiles)
    { shell = { debug = true; tracing = true; profiling = true; sanitizers = true;
                logging = "trace"; }; }

### Integrate existing tools -- do not reinvent
gdb/lldb (interactive), strace (syscalls), ltrace (library calls), perf (CPU), valgrind or compiler
sanitizers (memory), rr (deterministic record-and-replay -- excellent for hard-to-reproduce bugs),
bpftrace/eBPF (low-overhead kernel + syscall observability).
The framework should LAUNCH fsh under these tools and collect the artifacts.

### Suggested repository layout
    debug-platform/
      libdebug/ { logging tracing events metrics profiling assertions serialization }
      tui/  web/  recorder/  replay/  sqlite/
      visualizers/ { ast lexer memory timeline }
      plugins/  crash-reporter/
      integrations/ { gdb rr perf bpftrace }

## THE MEASURED REALITY -- half of this already exists, and part of it is DEAD (2026-07-16)
Do not start from zero. Recon before building, per the forest's own rule.

ALREADY BUILT AND WORKING:
  - faelight/runtime/state.db `events` table: 112,643 rows. Columns: id, domain, action, payload,
    timestamp, source_tool, correlation_id. FOUR indexes (domain, timestamp, domain+ts, action).
    That IS "SQLite instead of plain logs" and it IS the storage layer of the diagram.
  - Domains already emitting: compositor 63,015 / shell 43,081 / doctor 2,186 / git 1,870 /
    external 1,674 / intent 299 / security 262 / deploy 125. That is principle 1's "categories",
    already live, already queryable.
  - commands/mod.rs emit_command(db, &cmd, result_str) fires on EVERY command -- the "Security
    layer -- log every command". Principle 3's command recording, partially.
  - Friday already READS this: 13 patterns, 680 facts, 0.92 avg confidence. A consumer exists.
  - engine/src/domains/friday/events.rs is the emit API -- the seed of libdebug.

ALREADY BUILT AND DEAD -- fix this FIRST:
  - correlation_id: 112,643 rows total, 475 non-null, and ALL 475 HOLD THE SAME VALUE -- the EMPTY
    STRING. events.rs:33 reads `let corr = correlation_id.unwrap_or("");` and every caller passes
    None. The column's own doc comment says "optional session/workflow id for TRACING CAUSALITY".
    It has traced nothing since 2026-05-22. A correlation id that never varies correlates nothing.
    THIS IS PRINCIPLE 2. It is not missing -- it is present and inert.
  - source_tool: 112,173 rows EMPTY vs 473 "core", 2 "friday-chat", 2 "faelight-git". 99.6% blank.
  - Payload format is INCONSISTENT: `sandbox` parses its payload with serde_json::from_str (real
    JSON), while `external` emits the flat string `cmd:awk exit:0`. Principle 1 is half-true.
    A structured-log platform whose own storage is half-unstructured cannot deliver principle 1.

THIS IS THE INT-119 SHAPE, EXACTLY. A confident comment above a column that does nothing; a thing
that looks wired and is not. Found the same day INT-119's "unskippable" hook turned out never to
have been installed. The lesson transfers: BUILDING MORE ON TOP OF AN INERT FOUNDATION MAKES THE
LIE BIGGER, NOT THE SYSTEM BETTER.

## Phases -- P0 is startable tomorrow; P4 may never be worth it
P0 -- MAKE THE EXISTING INSTRUMENTATION HONEST. No new tools. Populate correlation_id with a real
  per-command id so one typed line's events can be SELECTed as a unit. Populate source_tool. Pick
  ONE payload format (JSON) and converge the emitters. Outcome: `SELECT * FROM events WHERE
  correlation_id=? ORDER BY timestamp` returns the true story of one command. THAT ALONE would have
  found INT-143's double-execution bug instantly -- two `exec` events for one typed line.
P1 -- libdebug: extract events.rs into a real instrumentation crate. trace/log/metric/event/
  record/profile_scope. Levels + categories, runtime-changeable.
P2 -- spans + timeline: ENTER/EXIT with parent ids. Startup timeline. Command recorder (input ->
  tokens -> AST -> expansion -> exec -> exit).
P3 -- the TUI: ratatui, htop-for-fsh. Live logs, timeline, panels. There is precedent in-repo --
  health_tui.rs, git_tui.rs, history_tui.rs, intent_tui.rs, cheatsheet_tui.rs all exist.
P4 -- MAYBE, AND ONLY IF EARNED: AST viewer, lexer visualizer, memory tracker, syscall recorder,
  crash reporter, replay, web UI, integrations (gdb/rr/perf/bpftrace).

## Scope guardrails -- read this before starting
THE HONEST RISK: libdebug + TUI + AST viewer + replay + crash reporter + memory tracker + syscall
recorder + web UI IS A PRODUCT, NOT A FEATURE. It is bigger than fsh. October is ~10 weeks out
(targets: Faelight Forest 80%, Friday 50%+conversation, fsh 98%) and DevBox competes with all three
for the same hours. The dashboard flags "CONTRADICTION: values declare focus>speed" already.

DO NOT BUILD A SEPARATE debug-platform/ REPO. The suggested layout above is the VISION's shape, not
this repo's. INT-112 measured the equivalent trap: a metal/ tree that would have been six
directories holding one host's files under a different name. libdebug belongs at
faelight/rust-tools/ or faelight/engine/ with the rest of the platform; the dependency seam (nix/
depends on faelight/, never the reverse) already exists and works.

DO NOT WRAP malloc IN RUST. The memory-tracker section is C-shaped thinking. Rust does not call
malloc the way the doc assumes; the equivalent is a custom GlobalAlloc, and the honest first
question is whether fsh has ever had a memory bug worth the machinery. Answer that before building.

DO NOT REINVENT strace/perf/rr. The doc says this itself and is right. Launch fsh under them and
collect artifacts; do not reimplement them.

P0 IS WORTH DOING NEXT. P1-P2 are worth doing when fsh's complexity demands it. P3 when the queries
get tedious. P4 when a specific bug proves the need -- not before.

## ============================================================
## REVISED 2026-08-14 -- supersedes the shape above, not the ownership
## ============================================================
A rewritten DevBox document arrived. Everything above is KEPT per INT-027; these six changes are
what materially improved, and each is recorded because the reason matters more than the edit.

1. THE CAUSAL MODEL IS FOUR CONCEPTS, NOT ONE ID.
   correlation_id was too ambiguous to trace with. It becomes:
     session_id       lifetime of one fsh process/session
     command_id       one typed command
     span_id          one meaningful nested operation
     parent_span_id   causal parent of that operation
   P0 needs only the first two. The schema must be shaped so P2 can add spans WITHOUT another
   migration of the fundamental causal model.
   *** AND THIS INDEPENDENTLY ARRIVES AT WHAT INT-191 MEASURED: identity is the PAIR, because
   execution_id restarts at 1 in every shell process. command_execution already keys on both. The
   revision reasoned to it; 191 proved it. They agree.

2. "EVERY FUNCTION GETS TRACING" IS WITHDRAWN.
   Principle 2 above says instrument everything. The revision says instrument every meaningful
   SUBSYSTEM BOUNDARY, and internals only selectively while investigating something specific.
   Reason: tracing every function distorts the timings it exists to measure, and makes the trace
   harder to read rather than easier.

3. REDACTION -- ENTIRELY ABSENT ABOVE, AND NOT HYPOTHETICAL.
   "Record everything" is not a security policy. Environment, arguments, stdin/stdout/stderr, paths,
   plugin state and config can all carry secrets. Redaction belongs AT THE INSTRUMENTATION BOUNDARY,
   never left to each downstream consumer.
   *** The events table ALREADY holds 43,081 shell rows including command text. This is a live
   property of the existing store, not a future concern.
   The rule: record enough to explain behaviour without turning the debugging database into a
   credential database.

4. THE EVENT CONTRACT IS domain + action + schema_version, NOT MERELY "JSON".
   JSON is the storage format; it does not define meaning. Versioning the contract is what lets a
   consumer read historical events after a producer evolves. Schema changes must be VERSIONED rather
   than silently changing what an existing event means.

5. REPLAY IS THREE DIFFERENT PROBLEMS -- the biggest conceptual fix in the revision.
     Level 1  HISTORY                     show what happened            (P0/P2)
     Level 2  SEMANTIC REPLAY             re-run shell stages against recorded input/state
     Level 3  DETERMINISTIC PROCESS REPLAY reproduce machine execution   DO NOT BUILD -- use rr
   The text above says "then REPLAY the execution", which conflates all three. An event log is a
   history of OBSERVATIONS; it is not automatically a deterministic execution recording.

6. THE TUI SPEC BELOW DESCRIBES A DIFFERENT TOOL.
   Config Test / Edit Config / Start Rebuild is a NIXOS CONFIGURATION workflow. An fsh debugger needs
   commands, sessions, timeline, events, AST, environment, processes, state.
   They can SHARE the event infrastructure without being one application. If the configuration TUI is
   still wanted, it is a separate small consumer of the platform.
   *** This also answers the open question recorded with that spec ("what goes in the LEFT pane vs
   the RIGHT pane -- TBD"): the question had no good answer because two tools were being drawn as one.


## Success Criteria
- [ ] P0: correlation_id carries a REAL per-command id -- prove it with a query returning one
      command's full event story, multiple distinct ids in the table
      <!-- UNBLOCKED 2026-07-29 by INT-169 and INT-191: THE ID NOW EXISTS. ExecContext carries
      `execution_id`, a process-local monotonic AtomicU64 shared by all three constructors, so
      every hook and event for one typed line can already name the same execution.
      ⚠️ AND IT IS NOT SUFFICIENT ALONE -- INT-191 proved that. It restarts at 1 in EVERY shell
      process, so persisting it by itself would let two concurrent sessions both claim 1, 2, 3: a
      key that looks unique and is not. The lifecycle identity is the PAIR, (session_id,
      execution_id), and `command_execution` already keys on both. Whatever P0 writes into
      correlation_id must carry the pair, not the counter.
      So P0 is now WIRING, not design: the id exists, the pairing is settled, and nothing writes
      it. INT-169's rides-with gate stays OPEN until this lands, deliberately, so the dependency
      keeps its reason. -->
- [ ] P0: source_tool populated by every emitter -- the 99.6%-empty number goes to ~0
- [ ] P0: ONE payload format, and every domain emits it. Written down, and old rows either migrated
      or explicitly declared legacy with a cutoff timestamp
- [ ] ⚠️ CHECK BEFORE ACCEPTING THIS GATE (flagged 2026-08-14): INT-143 was proven 2026-07-16, and
      INT-169, 196, 197, 203 and 220 have reworked execution since. IF 143 IS FIXED, a fresh event
      log CANNOT show it, and historical rows carry no command_id -- so this gate would be
      unachievable as written rather than merely hard. Check 143 status first. If fixed, the dogfood
      needs either a different live case or an explicit reframe: reconstruct a KNOWN double
      execution from a synthetic reproduction. The gate is right in spirit; its subject may have
      moved.
- [ ] P0 dogfood: replay INT-143's double-execution bug from the event log alone. If DevBox cannot
      show the SAME command exec'ing twice for one typed line, P0 is not done
- [ ] Every phase after P0 names the SPECIFIC bug or friction that justified it. "It would be cool"
      is not a gate. This intent's own guardrails say so
- [ ] ⚠️ AND NOTHING HERE BECOMES A SECOND OWNER EITHER (added 2026-08-14). The revision proposes
      `devbox events / trace / ast / timeline`. fsh ALREADY HAS AN `events` BUILTIN -- it is listed
      in `help` as "events -- recent events [today|domain]". Extend the consumer that exists before
      minting a new binary. INT-134 cut TEN roadmap items on 2026-08-14 for exactly this shape: a
      second owner of an idea that already has one.
- [ ] ⚠️ AND MEASURE P0 BEFORE SCOPING IT (added 2026-08-14). The revision reads as though P0 is
      unbuilt. INT-191 already built session identity, execution_id, and the command_execution
      lifecycle table -- and this intent's own gate note above already says "P0 is now WIRING, not
      design". On 2026-08-14 INT-134 found NINE of 27 roadmap items already built. The same recon
      discipline applies: run it before estimating.
      RECON LIST: is INT-143 still live · what does the existing `events` builtin already do · how
      much of session_id/execution_id does INT-191 already persist · does a crate boundary already
      exist (faelight-insightd is in the tools list, and engine/src/domains/friday/events.rs is
      named as the seed, so P1 may be a MOVE rather than an extraction).
- [ ] Nothing here becomes a separate repo. It lives in faelight/, per INT-061's domain seam
- [ ] Each gate carries evidence per INT-158 (docs/CONVENTIONS.md)

## Relationship
- Origin: Christian's own design, completed 2026-07-16 after a week of debugging work (INT-027/059/
  159 and the 143 session). Not filed earlier because the idea only finished that day.
- INT-119 is the cautionary twin: instrumentation that LOOKS wired and is not. correlation_id is
  that bug, sitting in DevBox's own foundation.
- INT-143 is P0's proving case: fsh runs every external `cmd > file` TWICE (proven 2026-07-16 --
  `rm -rf /tmp/dirtest; mkdir /tmp/dirtest > /tmp/mk.txt` -> "File exists"). A working event log
  makes that a one-line query. A missing one made it four minutes of manual bisection.
- INT-157 (fsh VM testing) is the complement: 157 PROVES correctness from outside, DevBox EXPLAINS
  behaviour from inside. Both are deferred until after Friday.
- INT-039 (friday-daemon) will want the same event spine.
- INT-171 gate 4 (tracing spans through the parser: lexer/parse/expansion/dispatch/execution, one
  command traced end to end) was ASSIGNED HERE by ownership decision 2026-07-20. It is this intent's
  principle 2, sequenced at P2 on the P0 correlation_id substrate. 171 de-scoped it rather than build
  spans on P0's not-yet-real foundation. When 167 reaches P2, "trace one typed command end to end"
  is a P2 deliverable inherited from 171 gate 4.

## The Rule
"Everything emits events. But an event that never varies is not instrumentation -- it is a comment
that compiles. Make the foundation honest before building the cathedral." 🌲


## DEVBOX TUI DESIGN (captured 2026-07-20) -- Christian's spec
Reference aesthetic: ArchTUI (https://gitlab.com/live4thamuzik/ArchTUI) -- a Rust TUI frontend
(ratatui-style) + modular backend for installing/administering Arch. GPLv3. DevBox borrows the
split-panel look, not the code.

LAYOUT (top to bottom):
- TOP: a SPLIT PANEL, two panes side by side.
  - LEFT pane:  ~33.3% of width.
  - RIGHT pane: the remaining ~66.6%.
- MIDDLE: a ROW OF THREE RECTANGLES (buttons/panels) below the split panel:
  - LEFT rectangle:   "Config Test"
  - MIDDLE rectangle: "Edit Config"
  - RIGHT rectangle:  "Start Rebuild"
- BOTTOM: a HELPERS row underneath the three rectangles (keybind hints / actions).

ASCII sketch of the intent:
+-------------------+-------------------------------------+
|                   |                                     |
|  LEFT pane 33.3%  |  RIGHT pane (~66.6%)                |
|                   |                                     |
+-------------------+------------------+------------------+
|  Config Test      |  Edit Config     |  Start Rebuild   |
+-------------------+------------------+------------------+
|  helpers row (keybinds / hints)                        |
+--------------------------------------------------------+

NOTES / OPEN QUESTIONS for when this is built:
- What goes in the LEFT (33.3%) pane vs the RIGHT (~66.6%) pane? (Likely: left = a list/nav/status,
  right = main content/output -- TBD, Christian to confirm.)
- The three rectangles map to DevBox's core actions: test the config, edit it, trigger a rebuild.
- Ties to 167's existing scope: DevBox as the debugging platform, and fsh's dead instrumentation
  columns (correlation_id + source_tool). The TUI is the FRONT of that platform.
- Framework: ratatui (matches ArchTUI's approach and the forest's Rust-TUI tools like faelight-fm).

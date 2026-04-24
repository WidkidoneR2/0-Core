**Scope:** How fsh executes commands today, what v9 needs to change, and what the friction items reveal about architectural direction.
**Audit session:** 2026-04-24
**Source at audit:** faelight-shell v0.7.0, commit 61b7e783
**Author:** Christian (sole developer)
**Reviewer during audit:** Claude (Anthropic)
**What this document is:**
- A real read of the shell as it exists, not as it should exist
- A map from current architecture to v9's pillars
- Honest about what v9 can extend vs what v9 must rebuild
- A reference that survives session breaks
**What this document is not:**
- An implementation plan (follows from this audit, separately)
- A rewrite justification (v9 is evolution on v8 foundation)
- A gate-by-gate schedule (gates depend on what we find)
---
*Writing in progress.*
---
*Queued.*
---
*Queued.*
---
*Queued.*
---
*Queued.*
---
*Queued.*
---
*Queued -- will be written last, after sections 1-6 establish the architectural baseline.*
---
*Populated as I read.*
---
*Written at end of session -- what's complete, what's remaining, recommended next audit pass.*
**Source: `exec.rs` (474 lines)**
The execution layer is built on `ExecContext` -- a typed struct that captures
everything about a command before it runs: raw string, expanded (post-alias)
string, command name, arg vector, cwd, active intent, timestamp, and a
`in_pipeline` flag.
Header comment at `exec.rs:7-8`:
**Source: `exec.rs` (474 lines)**
The execution layer is built on `ExecContext` -- a typed struct that captures
everything about a command before it runs: raw string, expanded (post-alias)
string, command name, arg vector, cwd, active intent, timestamp, and a
`in_pipeline` flag.
Header comment at `exec.rs:7-8`:
line → build_context() → preexec() → dispatch() → postexec() → result
Implemented in `execute_with_context()` at `exec.rs:458`:
```rust
let ctx = ExecContext::from_line(line, db);
if let Some(block_reason) = preexec(&ctx, db, core_root, rules) {
    return CommandResult::Error(block_reason);
}
let result = commands::execute(line, db, core_root);
postexec(&ctx, &result, db);
result
```
Three hooks (`preexec`, `dispatch`, `postexec`), typed context at two of them
(pre and post), raw string at the third (dispatch).
Nine fields. All present. All populated. Most notable for v9:
- `intent: Option<String>` -- already pulled from `db.get_focus_intent()` at
  context build time. Every command already knows which INT-NNN is active.
  v9's Friday deep integration can use this without new wiring.
- `in_pipeline: bool` -- marked pub, has a builder method (`with_pipeline`),
  but **currently never set to true** at any call site. Pipeline commands
  go through a different path (main.rs, TBD in Section 3). This field is
  aspirational; Pillar 1 needs to actually set it.
- `timestamp: u64` -- unix seconds. No sub-second precision, which matters
  for any v9 feature that wants to measure command duration or ordering
  at high frequency.
Runs before every command. Returns `None` to allow, `Some(message)` to block.
Four safety rules live here today:
1. **Catastrophic rm -rf protection** -- blocks `rm -rf /`, `/home`, etc.,
   and forest source dirs. Hardcoded list at line 113.
2. **Core lock enforcement** -- blocks git commit/push/etc. and fg ops when
   core is immutable-flagged. Shells out to `lsattr` for the check.
3. **Self-overwrite protection** -- blocks `cp`/`mv` to `scripts/core` unless
   the invocation is a deploy.
4. **Config rules** -- reads `BeforeRunRule` list from `config.fsh` and
   applies Block/Warn/Suggest actions.
Plus a fifth rule added by INT-194:
5. **DELETE confirmation** -- interactive "type DELETE to confirm" prompt
   for rm -rf on existing paths. Reads stdin mid-preexec at line 239-243.
**Architectural observation for v9:** The preexec hook is where CHALLENGE
level Friday interrupts plug in (Pillar 5). The existing structure is
well-suited for this: return `Some(message)` already means "block execution
and tell the user why." Friday's CHALLENGE level is exactly this pattern
driven by a confidence threshold + knowledge query rather than hardcoded rules.
Runs after every command. Has four distinct subsystems, each wrapping
around a different intelligence concern:
**(a) History recording (line 273-276).** Writes the command to shell_history
if status != "exit". Uses `db.save_history_entry(&ctx.raw)` -- the method
with the `.ok()` silent-drop bug discovered during INT-234 gate 8. Already
captured in INT-245 Pillar 6.
**(b) Failure memory (line 278-297).** On error status, writes
`last_failed_command` to shell_state and appends a `failure_log_<ts>` entry.
Enables `last_command retry/explain/fix` builtins. INT-176 heritage.
**(c) Knowledge engine query on failure (line 299-351, INT-233).** Tokenizes
error + command, filters noise words, queries `knowledge_entries` table for
matches with confidence ≥ 0.85. Surfaces inline. This IS Pillar 2's "Error
pattern recognition" capability -- partially built. What's missing for v9:
- It only fires on error status -- no success-side knowledge surfacing
- It does a single query, single result -- no multi-lesson presentation
- No signal back to knowledge engine about whether the suggestion helped
- The 0.85 threshold is hardcoded -- v9's interrupt-level system needs
  different thresholds for different behaviors
**(d) Suggestion system (line 352-452, INT-171 Phase 4 + Phase 28/INT-186).**
Two sub-paths:
- **Hardcoded suggestions** (line 353-367): match on ctx.cmd for known
  follow-ups. `fg commit` → "run d", `deploy` → "run d", etc. Six rules.
- **Predictive suggestions** (line 369-451): reads shell_history, finds
  most common next command after this one, gates on ALL of {accuracy ≥ 80%,
  occurrences ≥ 30, confidence ≥ 0.7, 3-minute cooldown} before firing.
  Full credibility output: confidence, occurrences, accuracy, causality
  claim, counterfactual ("might be wrong if:").
**Architectural observation for v9:** This is already Pillar 2 "Command
prediction" in a form. But it's passive -- shown AFTER the command runs. v9's
Tab-to-accept pattern needs a different surface: suggestions shown DURING
typing, not after execution. That's a different subsystem -- input-layer
completion, not output-layer suggestion. We'll find that code in
`completion.rs` (TBD Section 5).
Line 468: `let result = commands::execute(line, db, core_root);`
`ExecContext` does not reach the dispatcher. The dispatcher receives the
raw `line` string and re-parses. This means:
- **Pillar 1 (parallel):** The dispatcher cannot know it was called from
  a `parallel { }` block without re-parsing or without a new signal channel
- **Pillar 2 (intelligent):** Commands that want to inspect the active
  intent, timestamp, or pipeline context have to either re-fetch from db
  or receive context through a different path
- **Pillar 5 (Friday):** The same context ExecContext holds is needed
  inside commands, but has to be rebuilt per-command
**This is the first major v9 architectural change.** Either:
- (a) Thread `&ExecContext` through `commands::execute` -- requires signature
  changes across `commands/mod.rs` (~7000 lines, many match arms)
- (b) Store the active ExecContext in a thread-local or passed-through
  handle that commands can read when they need it
- (c) Keep the current raw-string dispatch and build a parallel context-aware
  dispatch path for v9 features that need it, leaving v8 commands unchanged
I do not know which Christian prefers. Recorded in Section 8 (Open Questions).
- Hook structure exists and is clean
- Typed context exists and is already populated with what v9 needs
- Context is built but discarded at dispatch -- the key gap
- Pre/post hooks are the natural attachment points for Friday interrupt levels
- Knowledge engine integration already exists in postexec but is single-query, single-surface, failure-only today
---
v9 will thread `&ExecContext` through `commands::execute` as a mandatory
parameter. Signature change propagates through all call sites in
`commands/mod.rs`, `scripting.rs`, `triggers.rs`, and the pipeline code in
`main.rs`.
**Rationale:** Options (b) thread-local and (c) dual-path were rejected as
quick fixes that carry long-term debt. Option (a) is the complete fix. It
means ~7000 lines of mechanical parameter addition plus ~15-20 places where
raw-string re-parsing can be replaced with structured ctx reads.
**Cost:** 2-3 focused implementation sessions before any user-visible v9
feature ships. Christian explicitly accepted this in audit session 2026-04-24:
"yes i want a complete job."
**Implication:** The first v9 commits will look like pure refactoring from
the outside. They are the foundation on which every subsequent pillar
attaches.
---
**Sources:** `main.rs:1285-1410` (native pipe loop), `main.rs:1033` and
`main.rs:1616` (execute_with_context call sites), and the many `Stdio::*`
sites across `commands/mod.rs`.
Three stdio dispositions appear in the pipeline and command dispatch code:
- **`Stdio::inherit()`** -- the child process writes directly to fsh's own
  stdout/stderr. fsh never sees the content. The user sees it in real time.
- **`Stdio::piped()`** -- fsh captures the child's stdout as a pipe fd and
  either connects it to the next stage's stdin, or reads it into a String
  for processing.
- **`Stdio::from(prev_stdout)`** -- the child's stdin is connected directly
  to the previous stage's captured stdout. This is how native pipe stages
  hand off.
When user input contains ` | ` and has external commands, fsh enters a
native pipe execution path:
- First stage stdin: `Stdio::inherit()` (line 1324) -- reads from terminal
- Last stage stdout: `Stdio::inherit()` (line 1327) -- writes to terminal
- Intermediate stage stdout: `Stdio::piped()` (line 1329) -- captured for
  next stage's stdin
- Every stage's stderr: `Stdio::inherit()` (line 1379) -- direct to terminal
The loop correctly chains `prev_stdout → next stdin`. Builtin-first logic
at line 1338 tries the fsh command table before falling back to external
process spawn. When a builtin fires mid-pipeline (line 1344-1372), fsh
takes the builtin's captured output and pipes it into `sh -c "<remaining>"`
for the rest of the chain -- a sh-shaped seam inside a supposedly-native path.
Any failure in the native pipe path -- tokenize failure, spawn failure, or
the catch-all else -- dumps to `sh -c "<original_line>"` with all three
streams inherited. The user types into fsh; sh runs the command; fsh never
sees what happened.
`execute_with_context(&base_cmd, ...)` fires here. `base_cmd` is the user
input with pipeline ops stripped out. If the command returns a `Value` and
there are pipeline ops that don't need external processes, fsh applies
them in-memory (line 1618-1620) -- this is the fsh-native value pipeline
(table transforms, sorts, filters). If external ops are present, fsh
again bails to `sh -c` (line 1624).
A second `execute_with_context` call site lives inside a shell-variable
handling block. After `expand_vars(rest, &shell_vars)` at line 1032, the
expanded command is dispatched through the canonical path. This means
variable-using commands ARE observed by preexec/postexec. Good.
`exec::execute_with_context` is the gateway to preexec (safety rules,
CHALLENGE-level Friday interrupts) and postexec (history recording,
knowledge engine queries, suggestion system).
**Paths that go through execute_with_context (OBSERVED):**
- Single commands without external pipes (main.rs:1616)
- Commands with pipeline ops applied to structured Values (main.rs:1616 + 1618-1620)
- Commands after variable expansion (main.rs:1033)
**Paths that bypass execute_with_context (UNOBSERVED):**
- Every pipeline containing external commands (main.rs:1285-1399)
- Every sh fallback (main.rs:1403-1410)
- Every fallback from the external-op path at line 1622-1624
Every `fsearch "x" | grep y` -- unobserved.
Every `ps -ef | grep foo` -- unobserved.
Every `cargo build 2>&1 | tail -30` -- unobserved.
Every multi-stage pipeline with external tools -- unobserved.
These are a substantial fraction of real shell activity. The preexec
safety rules (rm -rf protection, core-lock enforcement) do NOT fire
on pipeline commands. The knowledge engine does NOT query on pipeline
failures. The predictive suggestion system does NOT learn from
pipeline sequences. shell_history does NOT record pipeline commands
via the postexec path -- which means the history table has holes.
FSH-PHILOSOPHY.md Invariant 4: "Data that looks stored is actually stored."
In the persistence layer this refers to db writes. The `.ok()` silent-drop
bug was a violation. But there is a sibling violation in the observation
layer: activity that looks observed is not actually observed. The user sees
their pipeline command echoed in the prompt, sees its output in the
terminal -- but Friday never saw it, knowledge engine never queried on its
failure, shell_history never recorded it through the ctx path.
This is a v9 prerequisite. Before any Pillar 1-5 feature ships, the
pipeline and fallback paths must be routed through the same observation
layer as single commands -- otherwise Pillar 2 (Intelligent Execution) and
Pillar 5 (Friday Deep Integration) will have the same gap they have today,
just dressed up.
- Native pipe execution exists and is well-structured for what it does
- stdio modes are used correctly for pipe chaining
- stderr is always inherited -- this must change for labeled parallel output
- Two execute_with_context call sites exist, neither covers pipelines
- Roughly half of real shell activity runs unobserved by the intelligence layer
- This is not a bug -- it is a gap that is not visible from a feature checklist
- v9's first implementation target: wrap pipeline and fallback paths with
  the same preexec/postexec structure as single commands
---
**Sources:** `main.rs:1150-1410` (redirect + pipe dispatch), plus
`value::parse_pipeline` and `detect_redirect` (implementations not yet
read, flagged in section 3.5).
When fsh receives a command line, main.rs branches between five execution
paths:
1. **Redirect detected** (`redirect_info` is Some at line 1161) →
   redirect handling block (lines 1161-1267)
2. **No redirect + has external pipeline op** (line 1281) →
   native pipe loop (lines 1285-1399)
3. **No redirect + pipeline op but all internal** (implicit, falls through) →
   `execute_with_context` at line 1616 with pipeline_ops applied in memory
4. **No redirect, no pipe, plain command** →
   `execute_with_context` at line 1616
5. **Anything that fails path 2** → sh fallback at line 1403
Handles `> file`, `>> file`, `2> file`, `2>&1`, `2> /dev/null`.
Three sub-cases:
- **Stderr-only (`2>` or `2>&1` with no stdout redirect, line 1182-1197):**
  routes directly to `sh -c` with `Stdio::inherit()` on stdout. fsh never
  observes the command or its output.
- **Builtin result + redirect (line 1214-1226):** runs the builtin through
  `commands::execute` (NOT `execute_with_context`), captures the Output or
  Value result, writes to the target file. Respects `is_append`. Adds a
  trailing newline after each write.
- **External command + redirect (line 1227-1261):** spawns the command
  with stdout pointed at the target file. Handles `2>&1` (duplicates fd
  for both streams), `2>file` (separate file for stderr), and plain `>`/`>>`.
The builtin case IS observed at the commands layer but NOT at the
execute_with_context layer -- meaning preexec safety rules don't fire,
postexec doesn't record history, knowledge engine doesn't query on failure.
Already documented in Section 2. Key addition here: the branching at
line 1270-1278 uses `value::parse_pipeline` and `has_external_op` to
decide between native pipe execution (external ops present) and in-memory
value pipeline (all ops are fsh-native value transforms like sort/filter/
column).
This is a clean design. The fsh-native value pipeline is structured data
flowing through typed transforms. The external pipe loop is OS processes
flowing through fds. They're cleanly separated at the dispatch point.
Any failure in the native pipe loop (tokenization, spawn, or the generic
else) dumps to `sh -c "<original_line>"` with all three streams inherited.
Complete observation loss.
Two functions are called but not yet read in this audit pass:
- **`detect_redirect(line)`** at line 1160 -- returns `(line_stripped,
  redirect_info)`. Friction item 7 (`>>` append failing) and friction
  item 10 (junk files on parse failure) likely live inside this function's
  implementation. The current audit has not read it. **v9 implementation
  cannot fix friction items 7 and 10 without auditing this function.**
- **`value::parse_pipeline(line)`** at line 1271 -- returns `Vec<PipeOp>`.
  The classification between `External` and internal pipe ops determines
  which path the pipeline takes. If this parser misclassifies, commands
  route to the wrong execution path.
Both live in modules not yet read: `value.rs` or a submodule. Adding
to Section 8 open questions.
Section 2 claimed "roughly half of real shell activity runs unobserved."
Section 3 sharpens this:
**Paths that observe via execute_with_context (OBSERVED):**
- Plain single commands, no pipes, no redirects (main.rs:1616)
- Single commands with fsh-native value pipeline ops only (main.rs:1616 + value transforms)
- Commands after shell variable expansion (main.rs:1033)
**Paths that bypass execute_with_context (UNOBSERVED):**
- Redirects with external commands
- Redirects with builtins (partially -- commands::execute is called but not via ctx)
- Stderr-only redirects
- Pipelines with any external op
- Any sh fallback
The observed set is narrower than Section 2 implied. Most non-trivial
shell use (anything with a redirect, anything piping to head/grep/tail,
anything with `2>`) runs without preexec safety, postexec recording,
or knowledge surfacing.
v9 Pillar 1 has two explicit parallel constructs:
**Block form: `parallel { a; b; c }`** -- this is a new parse target. The
lexer/parser must recognize `parallel {` as a block start and collect
statements until `}`. Each statement then spawns concurrently. Most
naturally lives as a new branch in the dispatch tree at Section 3.1
-- a sixth path, evaluated before the redirect check (because `parallel`
wraps commands that may themselves contain redirects or pipes).
**Operator form: `a ||| b`** -- this splits on `|||` similar to how pipe
splits on `|`. Can plug into the same dispatch tree as a pre-check: if
the line contains `|||` outside quotes, route to parallel execution
instead of sequential.
Auto-parallel (the intent's aspirational pillar) requires dependency
analysis across a sequence of commands. This is a separate subsystem that
sits at the REPL loop level -- before dispatch branches -- and would
rewrite sequences into parallel blocks before they reach dispatch.
- Dispatch tree has five paths; observation gap applies to four of them
- Redirect handling is a substantial code block (~100 lines) with three
  sub-cases; friction items 7 and 10 live inside `detect_redirect` which
  has not been audited yet
- The native pipe vs value pipe split is clean and worth preserving
- sh fallback is an escape hatch used on any unexpected failure; every
  use of it is an observation loss
- Parallel execution needs a new dispatch-tree branch plus operator-level
  splitting, evaluated before redirect detection so parallel can wrap
  mixed content
- Auto-parallel detection is a separate subsystem at the REPL level, not
  a pipeline feature
---
33 lines. Three branches: stderr patterns (line 153-158), append redirect
(line 160-167), plain redirect (line 168-181). Default fallthrough returns
no redirect (line 182).
**Code reads correctly for the obvious cases.** Stderr patterns route to
`__stderr__` sentinel. `>>` and `>` are matched with `rfind(" >> ")` and
`rfind(" > ")` respectively. Guards against comparison operators (leading
digit, leading `=`) prevent false positives.
**Reported friction items cannot be pinned from reading alone:**
**Friction item 7 (`cat file >> target` fails "No such file or directory"):**
The logic at line 160-167 should handle `cat file >> target` correctly:
- `rfind(" >> ")` finds the redirect operator
- path = "target" (doesn't start with digit)
- returns `(cmd="cat file", Some(("target", true)))`
- main.rs:1198-1199 opens target with append+create
- cmd_part "cat file" is spawned with stdout redirected to target
From reading, this should work. The reported failure indicates either
(a) the reporter's actual command had different whitespace or quoting
than recorded, (b) a downstream bug after detect_redirect returns
correct values, or (c) an interaction we cannot trace from detect_redirect
alone.
**Friction item 10 (junk files like `=68`, `=69` in repo root):**
The guards on lines 163 and 176-177 prevent detect_redirect from treating
comparison-like strings as redirect targets. The reported junk files
(`=68`, `=69`, `=257`, and one containing SQL fragments) cannot originate
from detect_redirect under this reading -- something is creating these
files but it's not this function returning a bogus redirect target.
**Conclusion:** v9 redirect fixes require live reproducers. The next time
friction item 7 or 10 fires, capture the exact input string and trace
through detect_redirect with real values. Fixing these from architecture
alone is guesswork.
**Implication for v9 implementation order:** Friction items 7 and 10
move from "fix in detect_redirect" to "trace with live reproducer first,
then fix." This does NOT block other v9 work. Adding to Section 8.
---
**Source:** `main.rs` (23 `Command::new` sites), analyzed from fsearch at
2026-04-24.
fsh launches external processes through three distinct patterns, each
with different observability properties:
**Pattern 1 -- sh delegation (12 sites):**
- main.rs:63 (early startup)
- main.rs:382 (REPL execution variant)
- main.rs:400, 408 (two more sh launches near startup)
- main.rs:628, 665 (likely input processing paths)
- main.rs:1188 (stderr-only redirect handler -- already seen)
- main.rs:1356 (builtin-output-into-external-pipe seam -- already seen)
- main.rs:1403 (native pipe sh fallback -- already seen)
- main.rs:1605 (likely another execute_with_context fallback path)
- main.rs:1624, 1641 (pipeline-with-external-op fallback paths -- already seen)
Every sh delegation hides execution from fsh. Twelve sites means twelve
places where intelligence, safety, history, knowledge-surfacing, and
suggestion systems are bypassed.
**Pattern 2 -- direct external launches (9 sites):**
- main.rs:322 (`lsattr` for core-lock check)
- main.rs:437, 807, 1943 (three separate `core` invocations)
- main.rs:441 (`faelight-export`)
- main.rs:1230 (`parts[0]` -- user external command inside redirect path)
- main.rs:1375 (`cmd_name` -- user external command inside native pipe loop)
- main.rs:1700 (`faelight-notify`)
- main.rs:2032 (`git` for commits-today count in prompt)
These include forest internal tools (core, git, faelight-notify,
faelight-export) and two user-command launch points (1230, 1375).
Internal-tool launches are plumbing and are acceptable as-is for v9.
The user-command launches at 1230 and 1375 are v9-critical.
**Pattern 3 -- exec replacement (2 sites):**
- main.rs:542, 548 (`.exec()` calls -- fsh replaces itself with target)
Used by the `exec` builtin. Single-purpose. Not a v9 concern.
Two sites are where fsh actually spawns a user's external command:
**main.rs:1230 (redirect external branch):** `Command::new(parts[0])`
inside the redirect handler at line 1227-1260 (Section 3.2). This is
where `echo foo > file` runs its external command. The redirect logic
controls stdout, stderr handling for `2>&1` and `2>file`. This site
receives no ExecContext.
**main.rs:1375 (native pipe stage):** `Command::new(cmd_name)` inside
the pipe loop at lines 1285-1399 (Section 2.2). Every external command
inside a native pipe chain is launched here. Stdio is inherited for
first/last stages, piped for intermediate stages, stderr always inherited.
This site receives no ExecContext.
Both sites bypass the observation layer. Both sites are the targets
for v9's pipeline observability fix named in Section 2.9.
For `parallel { a; b; c }` and `a ||| b` to ship with the properties
the INT-245 intent names (labeled output, clean non-interleaving, jobs
control), v9 needs a new launch mechanism with five capabilities:
1. **Concurrent spawn of N children** -- trivial with `Command::spawn`
2. **Per-child output labeling** -- requires capturing stdout/stderr
   (NOT inherit) and prepending labels before emitting to terminal
3. **Observation integration** -- each parallel child runs through
   preexec/postexec so safety rules fire and history records
4. **Jobs table** -- a structured record of each running parallel child
   with its command, pid, start time, label, status. Required for the
   `jobs` / `wait` / `cancel` / `job-log` builtins
5. **Graceful failure handling** -- one child failing does not abort
   the others; the `parallel` block reports per-child success/failure
   at completion
None of the 23 existing launch sites provide all five. The native pipe
loop (site 1375) is closest but uses `Stdio::inherit()` and does not
label or record per-stage state.
The observation layer refactor (Section 2.9) and the parallel launch
subsystem are the same foundational work viewed from two angles:
- Section 2.9 asks "how do all execution paths observe?"
- Section 4.3 asks "how does concurrent execution label and track?"
Both require the same thing: a central process-launch function that
takes an ExecContext, captures stdio when needed, records to a jobs
or observation table, and routes through preexec/postexec.
**Proposed v9 first implementation target:**
Build a single `spawn_observed(ctx, command_spec, stdio_policy) -> JobHandle`
function that:
- Takes full ExecContext
- Takes a command spec (argv, env, cwd overrides)
- Takes a stdio policy (inherit vs capture vs pipe-to-next)
- Runs preexec hook (blocks can veto)
- Spawns the child
- Records a row in a jobs table with pid, label, timestamps
- Captures output according to policy
- Runs postexec hook on completion
- Returns a JobHandle the caller can wait on, label, cancel
Then migrate the 12 sh delegation sites and 2 user-command launch sites
to call `spawn_observed` with appropriate policies. Parallel execution
becomes `N × spawn_observed` called concurrently. Pipe execution becomes
`spawn_observed` with stdout piped to the next stage's input.
This unifies Sections 2, 3, and 4 into one architectural move. It also
gives v9 the `jobs` / `wait` / `cancel` / `job-log` builtins essentially
for free -- they become queries against the jobs table.
- 23 total process-launch sites in main.rs
- 12 are sh delegations (observation loss)
- 9 are direct external launches (7 internal plumbing, 2 user-command)
- 2 user-command sites (1230 redirect external, 1375 native pipe stage)
  are the primary v9 observation targets alongside the sh delegations
- No existing launch site supports labeling, jobs tracking, or observation
- v9 parallel execution needs a new `spawn_observed` function rather than
  extending existing launch sites
- Migrating existing sites to spawn_observed unifies Section 2's
  observation fix with Section 4's parallel enablement into one change
The central launch API for v9 is a builder-pattern Job type with hybrid
observation and hybrid jobs storage, plus policy-per-block failure handling
for parallel execution.
**Axis A -- API shape: Builder pattern.**
```rust
Job::new(&ctx, "cargo")
    .arg("build")
    .arg("--release")
    .stdout(StdoutMode::Capture)
    .stderr(StderrMode::Capture)
    .label("[core]")
    .observed(true)
    .spawn()
```
Rationale: Rust-idiomatic, scales with new policies, reads cleanly at
call sites. Rejected: single-function with policy argument (grows messy),
trait-based strategy (heavier abstraction without clear payoff today),
async-first (changes the whole concurrency model, biggest leap).
**Axis B -- Jobs table: Hybrid in-memory + state.db.**
Active jobs live in an in-process HashMap or similar for fast access.
Completed jobs write a row to a `shell_jobs` table in state.db for
history and queries. The `jobs` / `wait` / `cancel` / `job-log` builtins
read from the in-memory active set plus recent state.db rows.
Rationale: matches the forest's "structured data in SQL" philosophy
while keeping hot-path access fast. Rejected: in-memory only (loses
history across sessions), state.db only (unnecessary latency for every
running job's status check).
**Axis C -- Observation scope: Hybrid wrap-only by default, mid-flight opt-in.**
Every spawn runs through preexec before and postexec after by default.
Regular commands stay wrap-only. Parallel blocks opt into mid-flight
capture because labeling requires capturing output anyway. A future
`--observe` flag or builder method (`Job::new(&ctx, cmd).observed_stream(true)`)
lets specific commands request mid-flight observation.
Rationale: (i) wrap-only was tempting but silently breaks Pillar 1's
labeled parallel output requirement. (ii) mid-flight always adds capture
overhead to every command unnecessarily. (iii) gets most of the capability
at most of the cost -- capture where labeling demands it, wrap where it
doesn't, opt-in for future CHALLENGE-during-execution features.
**Axis D -- Parallel failure policy: Policy-per-block, both supported.**
`parallel { a; b; c }` -- best-effort (all run, failures reported at end).
`parallel! { a; b; c }` -- fail-fast (first failure sends SIGTERM to siblings).
Rationale: both patterns have legitimate use cases (exploration wants
best-effort, CI wants fail-fast). Supporting both costs little once the
spawn subsystem exists.
**Commitment:** all four axes apply to every v9 process launch through
the new spawn subsystem. No exceptions baked in without updating this
record.
---
The original 4.4 proposed a single `spawn_observed(ctx, spec, policy) -> JobHandle`
function. Section D2 decides the real design: builder-pattern API,
hybrid jobs storage, hybrid observation, policy-per-block failure.
```rust
pub struct Job<'a> {
    ctx: &'a ExecContext,
    program: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    cwd: Option<PathBuf>,
    stdin_mode: StdinMode,
    stdout_mode: StdoutMode,
    stderr_mode: StderrMode,
    label: Option<String>,
    observed: bool,
    observed_stream: bool,  // mid-flight capture opt-in
}
pub enum StdinMode {
    Inherit,
    FromPipe(ChildStdout),  // chain from previous stage
    FromString(String),     // pipe a string in
    Null,
}
pub enum StdoutMode {
    Inherit,
    Capture,                // read into String after completion
    Piped,                  // for chaining to next stage
    LabeledStream(String),  // capture, prepend label, emit live
    ToFile(PathBuf, bool),  // path, append
}
pub enum StderrMode {
    Inherit,
    Capture,
    MergeStdout,            // 2>&1
    ToFile(PathBuf),
    LabeledStream(String),
}
```
```rust
impl<'a> Job<'a> {
    pub fn new(ctx: &'a ExecContext, program: impl Into<String>) -> Self { ... }
    pub fn arg(mut self, a: impl Into<String>) -> Self { ... }
    pub fn args<I: IntoIterator<Item = String>>(mut self, a: I) -> Self { ... }
    pub fn env(mut self, key: &str, val: &str) -> Self { ... }
    pub fn cwd(mut self, path: PathBuf) -> Self { ... }
    pub fn stdin(mut self, mode: StdinMode) -> Self { ... }
    pub fn stdout(mut self, mode: StdoutMode) -> Self { ... }
    pub fn stderr(mut self, mode: StderrMode) -> Self { ... }
    pub fn label(mut self, label: impl Into<String>) -> Self { ... }
    pub fn observed(mut self, b: bool) -> Self { ... }
    pub fn observed_stream(mut self, b: bool) -> Self { ... }
    pub fn spawn(self) -> Result<JobHandle, JobError> { ... }
}
```
```rust
pub struct JobHandle {
    pub id: u64,            // from jobs table
    pub pid: u32,
    pub label: Option<String>,
    pub started_at: u64,
    inner: JobInner,        // opaque: Child + capture threads if any
}
impl JobHandle {
    pub fn wait(self) -> JobResult { ... }
    pub fn wait_timeout(self, d: Duration) -> Result<JobResult, JobHandle> { ... }
    pub fn kill(self) -> Result<JobResult, JobError> { ... }
    pub fn status(&self) -> JobStatus { ... }  // Running | Exited(code) | Signaled | Killed
    pub fn id(&self) -> u64 { self.id }
}
pub struct JobResult {
    pub id: u64,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub stdout: Option<String>,  // if StdoutMode::Capture
    pub stderr: Option<String>,  // if StderrMode::Capture
    pub duration_ms: u64,
    pub label: Option<String>,
}
```
```sql
CREATE TABLE shell_jobs (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    program TEXT NOT NULL,
    args TEXT NOT NULL,               -- JSON
    cwd TEXT NOT NULL,
    env_keys TEXT,                    -- JSON array of modified env keys
    label TEXT,
    parent_ctx_raw TEXT,              -- ExecContext.raw at spawn time
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    exit_code INTEGER,
    signal INTEGER,
    stdout TEXT,                      -- if captured
    stderr TEXT,                      -- if captured
    duration_ms INTEGER,
    observed INTEGER NOT NULL,
    observed_stream INTEGER NOT NULL,
    parallel_block_id INTEGER         -- NULL unless part of parallel { }
);
CREATE INDEX idx_shell_jobs_session ON shell_jobs(session_id);
CREATE INDEX idx_shell_jobs_started ON shell_jobs(started_at);
CREATE INDEX idx_shell_jobs_label ON shell_jobs(label);
```
Active jobs are held in a `HashMap<u64, JobHandle>` inside a process-level
`JobTable`. On completion the handle moves to the state.db row (stdout
and stderr persisted if captured).
Inside `Job::spawn()`:
1. If `observed == true`: run preexec hook via ctx. Block may veto
   (returns `JobError::Blocked(message)`).
2. Insert row into shell_jobs with `completed_at = NULL`.
3. Spawn the child with appropriate `Stdio::*` based on modes.
4. If `observed_stream == true`: spawn one or two reader threads that
   prepend the label to each line and forward to terminal while also
   capturing into the stdout/stderr buffers.
5. Return `JobHandle`.
On `wait()` or `status()` transition to Exited:
1. Collect stdout/stderr buffers if captured.
2. Update shell_jobs row with completed_at, exit_code, signal, stdout,
   stderr, duration_ms.
3. If `observed == true`: run postexec hook via ctx with synthesized
   CommandResult from exit code + captured output.
4. Return `JobResult`.
Every sh delegation and every direct-user-command launch migrates to
`Job::new(...)`. Mapping table:
| main.rs line | Current pattern           | v9 migration                        |
|--------------|---------------------------|-------------------------------------|
| 63           | sh -c (startup)           | Job::new with observed=false        |
| 322          | lsattr (core-lock check)  | Job::new (internal, not observed)   |
| 382          | sh -c (REPL variant)      | Job::new with observed=true         |
| 400          | sh -c                     | Job::new (context-dependent)        |
| 408          | sh -c                     | Job::new (context-dependent)        |
| 437          | core (startup)            | Job::new (internal)                 |
| 441          | faelight-export           | Job::new (internal)                 |
| 542, 548     | .exec() replacements      | NOT migrated -- exec is terminal     |
| 628, 665     | sh -c (input paths)       | Job::new with observed=true         |
| 807          | core (specific feature)   | Job::new (internal)                 |
| 1188         | sh -c (stderr-only)       | Job::new with StderrMode::ToFile    |
| 1230         | parts[0] (redirect user)  | Job::new with stdout ToFile etc.    |
| 1356         | sh -c (builtin→pipe seam) | Job::new piping from in-memory str  |
| 1375         | cmd_name (pipe stage)     | Job::new with Piped stdout          |
| 1403         | sh -c (pipe fallback)     | REPLACE -- expand native pipe path   |
| 1605,1624,1641| sh -c fallbacks          | REPLACE -- see detect flow           |
| 1700         | faelight-notify           | Job::new (internal)                 |
| 1943         | core                      | Job::new (internal)                 |
| 2032         | git (prompt)              | Job::new (internal, not observed)   |
Per D2 commitment: every site migrates or gets explicitly replaced.
Zero sh delegations survive v9 without a recorded justification.
`parallel { a; b; c }` becomes roughly:
```rust
let handles: Vec<JobHandle> = block.statements.iter().map(|stmt| {
    Job::new(&ctx, stmt.program)
        .args(stmt.args)
        .label(format!("[{}]", stmt.program))
        .stdout(StdoutMode::LabeledStream(format!("[{}]", stmt.program)))
        .stderr(StderrMode::LabeledStream(format!("[{}]", stmt.program)))
        .observed(true)
        .observed_stream(true)
        .spawn()
}).collect::<Result<Vec<_>, _>>()?;
let results: Vec<JobResult> = handles.into_iter().map(|h| h.wait()).collect();
```
`parallel! { }` adds: on first failure, iterate remaining handles and
call `kill()` on each.
Migration from current main.rs to this design is a multi-session effort.
Reasonable phase breakdown:
1. **Phase 0:** Build the `Job` type, `JobHandle`, `JobResult`, `JobTable`,
   and `shell_jobs` migration. Nothing wired in yet -- pure infrastructure.
   Tests exercise it directly.
2. **Phase 1:** Migrate the 2 user-command sites (1230, 1375). Every
   external user command goes through Job. Prove observation works end
   to end for non-parallel cases.
3. **Phase 2:** Migrate the 12 sh delegations one by one, recording
   justification for any that cannot be replaced.
4. **Phase 3:** Add `parallel { }` parse and execution. Uses the Job
   type that now exists.
5. **Phase 4:** Add `parallel! { }` fail-fast variant.
6. **Phase 5:** Add `|||` operator as sugar for a 2-command parallel block.
7. **Phase 6:** Add `jobs` / `wait` / `cancel` / `job-log` builtins
   (queries against the jobs table).
Each phase is gate-sized. Each ships working. The foundation (Phase 0-1)
is the one that is pure refactor with no user-visible feature -- the
"complete job" Christian accepted at D1.
---
Section 4 proposed the Job subsystem. Section 5 maps where other v9
pillars plug in across the remaining source files, starting with job
control.
**Current shape.** Job struct with id, cmd, child, started fields.
JobTable with Vec<Job> + monotonic next_id. Operations: spawn (with
Stdio::inherit, Stdio::null for stdin), check_completed (before prompt
render, reports status+timing), list, fg, kill_job.
**What works.** Non-blocking try_wait. Clean separation. User-visible
output is good.
**What v9's Job design (Section 4.4.1) needs that this doesn't have:**
1. ExecContext parameter on spawn -- D1 context threading
2. Label field -- Pillar 1 labeled output
3. Observed flag -- preexec/postexec integration
4. Stdout capture mode -- mid-flight observation and labeling
5. state.db persistence for completed jobs -- Axis B hybrid storage
6. Signal-granular cancel (SIGTERM then SIGKILL) -- D2 Axis D fail-fast
7. Wait-by-label and wait-all operations -- parallel block semantics
8. Different abstraction shape -- launch spec vs remembered process
The current `Job` struct and the proposed v9 `Job` builder are different
abstractions with the same name:
- Current: "a process I launched and remember" (stored state after spawn)
- Proposed: "a launch specification with policy" (pre-spawn builder that
  produces a JobHandle)
These cannot coexist. One needs to rename or merge.
**Path I -- Extend current Job type incrementally.**
Pro: each phase is small. No ripple to current callers. Phase 0 stays pure infra.
Con: legacy shape persists into v9 foundation; builder pattern (D2 Axis A)
ends up grafted onto a struct designed for a simpler abstraction.
**Path II -- Replace entirely, rename current Job to BgJob (legacy).**
Pro: clean foundation matching Section 4.4 design. No compromises in the
builder shape.
Con: more files touched in Phase 0. Higher regression risk during migration.
Recorded as D3. Decision required before Phase 0 implementation begins.
Current `jobs.rs::Job` renames to `BgJob` for legacy background-process
tracking. New `Job` builder (Section 4.4.1) becomes the canonical launch
spec. Migration path:
1. Phase 0 step A: rename `Job` → `BgJob` across jobs.rs and its callers.
2. Phase 0 step B: build new `Job` builder + `JobHandle` + `JobResult` +
   `JobTable` (new type) per Section 4.4 design. BgJob and JobTable (old)
   coexist during migration.
3. Phase 0 step C: migrate `&` (background) handling to new Job system
   with observed=true default, retire BgJob.
Rationale captured from Christian 2026-04-24: "path 2 gives a cleaner
shell that handles more in the long run, we have to look at long term."
This commits to more files touched in Phase 0 in exchange for a clean
foundation. Consistent with D1 "complete job" acceptance.
---
**Role.** Thin wrapper around a single `rusqlite::Connection` to the
shared state.db at `~/0-core/runtime/state.db`. Owns three tables
(shell_history, shell_aliases, shell_state). Consumes many others
(events, friday_*, knowledge_entries, doctor events). Core owns the
shared schema; fsh is a consumer for most of it.
The known `.ok()` bug is at line 165 in `save_history_entry`. Auditing
the whole file reveals **six mutation sites with silent error handling:**
- Line 73 `add_alias` -- returns bool via is_ok() ✅ honest
- Line 83 `remove_alias` -- returns bool ✅ honest
- Line 165 `save_history_entry` -- **`.ok()` discard** ❌ bug (friction 9)
- Line 241 `set_focus_intent` -- **`let _ = x.execute(...)`** ❌ silent
- Line 258 `set_theme` -- **`let _ = x.execute(...)`** ❌ silent
- Line 273 `clear_focus_intent` -- **`let _ = x.execute(...)`** ❌ silent
Three of six mutations honestly report success/failure to the caller.
Three silently drop errors. Friction item 9 is not a single bug -- it
is a pattern that the codebase adopted inconsistently.
**v9 requirement.** Every mutation returns a result. Caller can choose
to ignore, but the option to know must exist. No `.ok()` on INSERT/UPDATE.
No `let _ = execute(...)`. This is an Invariant
**Current state:** db.rs creates three tables with CREATE IF NOT EXISTS.
No version tracking. No migration path for schema changes. Other tables
(events, friday_*, etc.) are created by core or other tools; fsh assumes
they exist.
**What bit us at INT-234 gate 8:** when state.db got a schema migration
elsewhere, fsh's long-held connection could see stale metadata. Combined
with silent errors (friction 9), this caused 1 hour of invisible history
failures.
**v9 requirement.** Schema version tracking. When fsh opens state.db,
it reads a `schema_version` row. If it doesn't match what fsh expects,
fsh either runs its own migrations (for its tables) or refuses to start
with a clear error (for core-owned tables). No silent stale state.
Section 4.4.4 specified a `shell_jobs` table. Two choices for where it
lives:
**Option X -- In db.rs::open()** alongside shell_history/shell_aliases/
shell_state. fsh owns the table because fsh creates and consumes it.
**Option Y -- In core's runtime init.** Centralizes all state.db schema
in one place. fsh just consumes.
**My lean: Option X.** shell_jobs is fsh-specific. Core has no reason
to know. Keeping the schema with the consumer reduces coordination debt.
If core ever wants to query it, it can without owning it.
Deferred to D4.
`query_events` at line 188 uses `format!("SELECT ... WHERE domain='{}'"`
rather than parameterized queries. If a caller ever passes user input
as `domain`, the shell is vulnerable. Today all callers pass string
literals. Tomorrow is tomorrow.
**v9 requirement.** Convert query_events to parameterized queries.
Audit for any other format!-into-SQL patterns across the shell.
`health_score` at line 279 reads `events` table where `domain='doctor'`
and parses `payload` as JSON looking for `detail.health`. This is an
implicit contract with core. If core changes doctor payload shape,
fsh silently gets None. Worth documenting in v9 as an "external
schema dependency" and potentially adding to the schema_version check.
1. `shell_jobs` table migration (Section 4.4.4)
2. Return types on all mutation methods (compliance with Invariant
3. `schema_version` tracking with migration paths
4. Parameterized queries everywhere
5. Explicit documentation of cross-module contracts for any table fsh
   reads but does not own
6. Possibly: reconnection or schema-revalidation on state.db change
   notification (solves the stale-connection class)
- db.rs is a facade, not a schema owner for most tables
- Friction item 9 is one of six silent-drop sites, not a standalone bug
- v9 needs a schema versioning layer
- shell_jobs migration should live in db.rs (D4 proposed)
- SQL injection surface exists today on query_events
- Cross-module contracts with core (health_score, session tables)
  are implicit and need explicit documentation
Per Section 5.2.3, the new `shell_jobs` table per Section 4.4.4 design
will be created in `db.rs::open()` alongside the existing fsh-owned
tables (shell_history, shell_aliases, shell_state). Core does not own it.
Rationale: fsh is the only writer and primary reader. Keeping schema
with consumer reduces coordination debt. Core retains the ability to
query the table if needed without owning the schema.
Friction item 9 (INT-245) described ONE silent-drop bug in
`save_history_entry`. Section 5.2.1 audit reveals **three siblings**
with the same failure mode:
- `db.rs:241 set_focus_intent` -- `let _ = conn.execute(...)`
- `db.rs:258 set_theme` -- `let _ = conn.execute(...)`
- `db.rs:273 clear_focus_intent` -- `let _ = conn.execute(...)`
Each can silently fail under the same conditions that bit
save_history_entry during INT-234 gate 8 (stale connection after
schema migration, lock contention, disk full). When they fail, fsh
reports nothing and appears to work.
**Current user-visible impact when these fail:**
- `set_focus_intent` silent fail: `cistart INT-NNN` appears to work
  but focus never changes. Subsequent commands run without intent context.
- `set_theme` silent fail: theme change appears to work but persists
  old theme across sessions.
- `clear_focus_intent` silent fail: `cicomplete` appears to close the
  intent but focus_intent row remains; next session opens claiming the
  completed intent is still active.
**Action required:** Update INT-245 Pillar 6 friction item 9 to
explicitly list all four sites. v9 implementation Phase 0 or an
earlier patch fixes all four at once with uniform error-returning
mutations.
Decision: file this as a friction update post-audit, not as a
hot-patch in the middle of the audit. Audits produce findings;
findings generate gate updates.
---
**Role.** Implements rustyline's `Helper` trait for fsh. `Completer` is
wired to `completions_for()`. `Hinter`, `Highlighter`, `Validator` are
implemented but empty or pass-through.
Seven case branches in `completions_for()`:
- Case 1 (line 118): pipe op + column completion after ` | `
- Case 2 (line 179): schema table name after `schema `
- Case 2b (line 197): multi-word forest command (hardcoded MULTI_CMDS list)
- Case 2c (line 402): path completion for cd and path-like arguments
- Case 2d (line 427): intent ID completion (cistart, cicomplete, intent show)
- Case 2d2 (line 462): git branch completion
- Case 2e (line 496): first-word with alias lookup and binary scan
- Case 3 (line 527): first-word with binary scan (no alias lookup)
**Case 3 is dead code for most inputs** -- Case 2e handles the same
condition and falls through only when it has no candidates. Case 3 is
redundant with Case 2e minus the alias lookup.
- Schema-aware pipe completion (Case 1) -- `ps | where c<TAB>` suggests
  ps's real columns. Most shells can't do this.
- Dynamic intent ID completion reads the filesystem per tab press.
- Git branch completion shells out to `git branch -a` per tab press.
The `Hinter` trait impl at line 659-661 is empty:
```rust
impl Hinter for ForestHelper {
    type Hint = String;
}
```
This is rustyline's "inline suggestion as ghost text" surface. Right now
fsh has **no inline hints while typing**. The existing predictive
suggestions in exec.rs:352-452 show AFTER execution, not DURING typing.
**This is the plug point for Pillar 2 "Command prediction":**
```rust
impl Hinter for ForestHelper {
    type Hint = String;
    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        // Read friday_patterns + shell_history, find likely continuation
        // Apply Phase 28 gate thresholds (confidence, occurrences, accuracy)
        // Return Some(continuation) only if gate passes
    }
}
```
Tab in rustyline accepts the hint by default. One signature change plus
a friday_patterns query plus the existing threshold gate from exec.rs
delivers v9's Pillar 2 "Tab accepts Friday suggestion."
Three static arrays drive first-word and multi-word completion:
- `COMMANDS` (line 14-99): 85 entries
- `PIPE_OPS` (line 101-103): 10 entries
- `MULTI_CMDS` (line 199-391): 150+ entries
All manually maintained. Already drifting -- line 68 and line 76 both
list `theme` (duplicate), `q` appears at line 82 and line 98 (duplicate),
and any command added in `commands/mod.rs` or in core engine after these
arrays were written is absent from tab completion until a human updates
the arrays.
**v9 direction:** generate these arrays at startup by introspecting
the actual command dispatch table (commands/mod.rs) and the core engine's
command registry. Eliminates drift. Makes "add a command" a single-file
change rather than a 3-file coordination.
Deferred as D5.
Three cases do real I/O on every matching tab press:
- Case 2d reads intents directory (filesystem stat per entry)
- Case 2d2 spawns `git branch -a` (process spawn + git invocation)
- Case 2e opens a fresh rusqlite connection to state.db
At v9 scale with Friday hints added, each tab press already does filesystem
work, spawns processes, and opens databases. Not disastrous at keyboard
speeds but worth caching. A `CompletionCache` or lazy-init pattern where
intents list is read once per session (or on filesystem change) and
connection is reused reduces this.
Deferred as D6.
`Highlighter::highlight` at line 664-666 returns the line unchanged:
```rust
fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
    Cow::Borrowed(line)
}
```
No syntax highlighting while typing. No visual cue when the user is
about to run something risky. Pillar 5 CHALLENGE level ("CHALLENGE: fsh
stops you before executing") could also surface as amber coloring on
dangerous substrings BEFORE the user hits Enter -- a visual warning
rather than a blocking prompt. That plug-in point lives in Highlighter.
`Validator` trait impl at line 677 is empty. rustyline uses Validator
to decide whether to accept Enter or continue reading. Empty = every
Enter submits. v9 could add syntactic validation for `parallel { }`
blocks (reject on unmatched brace), heredoc delimiters (warn on visible
leak patterns like those INT-249 wants to catch), or quoted-string
completeness. Right now these errors are only caught at execution time.
1. Populate `Hinter` with friday-aware predictions + Phase 28 threshold
   gate -- Pillar 2 Tab-to-accept.
2. Replace hardcoded COMMANDS / MULTI_CMDS arrays with introspection-based
   generation (D5 deferred decision).
3. Add a CompletionCache to reduce per-tab I/O (D6 deferred decision).
4. Populate `Highlighter` with risk-aware coloring for dangerous commands
   -- Pillar 5 CHALLENGE pre-execution visual.
5. Populate `Validator` for syntactic pre-checks on v9 constructs like
   `parallel { }` blocks.
6. Remove Case 3 (dead code) after confirming it's unreachable.
- Hinter is the Pillar 2 Tab-to-accept plug point, currently empty
- Highlighter is the Pillar 5 CHALLENGE visual plug point, currently pass-through
- Validator is the v9 syntactic pre-check plug point, currently empty
- Three hardcoded command arrays have started drifting (duplicates visible)
- Per-tab I/O cost is noticeable; caching is a v9 improvement
- Case 3 appears to be dead code masked by Case 2e's fallthrough
---
**Role.** Parses and executes .fsh script files and (via `run_source`)
inline script sources. Implements variable bindings, conditional and
event-driven blocks, command execution, event emission, and interactive
prompts. Phase 6 established this as a real scripting language, not a
macro system.
Seven statement variants: Let, If, When, Run, Emit, Warn, Confirm. Each
has a parse branch (lines 83-130) and an execute branch (run_stmt, lines
192-291).
The `collect_block()` function at line 135 tracks brace depth and
recursively parses nested bodies. The `if` and `when` parsers at lines
92-101 demonstrate the pattern:
```rust
let rest = line.strip_prefix("if ")?;
let condition = rest.trim_end_matches('{').trim().to_string();
let body = collect_block(&lines, &mut i);
stmts.push(Statement::If { condition, body });
```
**This is the exact shape `parallel { ... }` needs.** Pillar 1 extension
is approximately:
```rust
pub enum Statement {
    // ... existing ...
    Parallel {
        body: Vec<Statement>,
        strict: bool,  // true for parallel! { } fail-fast
    },
}
// In parse():
} else if line.starts_with("parallel {") || line.starts_with("parallel! {") {
    let strict = line.starts_with("parallel! ");
    let body = collect_block(&lines, &mut i);
    stmts.push(Statement::Parallel { body, strict });
}
// In run_stmt():
Statement::Parallel { body, strict } => {
    // Use Job builder for each body statement (Section 4.4.7)
    // Apply strict vs best-effort policy (D2 Axis D)
}
```
Pillar 1 block syntax is a ~15 line parser addition plus the executor
logic that delegates to the Job subsystem. The parser foundation already
exists.
`run_source` at line 472-482 is marked `#[allow(dead_code)]`:
```rust
pub fn run_source(source: &str, db: &ForestDb, core_root: &str) -> CommandResult {
    let stmts = parse(source);
    ...
}
```
This means **typing `parallel { ... }` at the fsh prompt today has no
path into scripting.rs.** Scripting only reaches via `run file.fsh` in
commands/mod.rs -- files, not REPL input.
For Pillar 1 to work at the REPL, main.rs needs to detect multi-line
block input (open brace without closing brace on the same line), buffer
the following lines until the matching close brace, and route the
accumulated source to `run_source`. That's a REPL-level change in
main.rs, not here.
**Deferred as D7.** Design question: does the REPL buffer silently and
present a continuation prompt (like bash's `>`), or does the user enter
an explicit multi-line mode? Either is defensible. Bash's continuation
prompt is familiar. An explicit mode is more discoverable.
Line 263 in Statement::Emit execution:
```rust
db.conn.execute(
    "INSERT INTO events (domain, action, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
    rusqlite::params![domain, action, payload.as_deref().unwrap_or(""), ts]
).ok();
```
Same silent-drop pattern. If an event emission fails (stale connection,
disk full, lock contention), the user sees the success message "emit
event.name" on line 264 while the event never got recorded.
**Running count of silent-drop sites in audited files:**
- db.rs: 4 sites (save_history_entry, set_focus_intent, set_theme, clear_focus_intent)
- scripting.rs: 1 site (Statement::Emit)
- Total: **5 sites across 2 files audited so far**
This is now documented as systemic, not localized. Friction item 9 in
INT-245 describes one bug. The audit has found four siblings. Updating
F1 (Section 5.2 Finding) to reflect the full count.
**`is_literal()` at line 170** uses string-prefix heuristics. Good
enough for today's scripting, insufficient for Pillar 1 auto-parallel
detection and Pillar 2 intelligent execution which will need to reason
about command dependencies. v9 auto-parallel requires a proper expression
parser, not prefix guessing.
**`eval_condition()` at line 293** supports only 3-token conditions
(`<expr> <op> <number-or-string>`). No boolean combinators, no
parentheses. Pillar 5 CHALLENGE levels need richer conditions ("confidence
above 0.85 AND knowledge-entry exists for error signature"). v9 extends
this but the extension is additive, not rewriting.
**`resolve_value()` at line 323** hardcodes `health` and `commits` as
forest-provided values. Adding more named values requires code change.
A named-value registry keyed on string with callable resolvers would
be cleaner and is a small v9 addition.
- Block parser with depth tracking already exists and is correct
- `parallel { }` and `parallel! { }` are a ~15 line parser extension
  plus executor delegation to Job subsystem
- `run_source` exists but is currently dead code -- REPL has no path to scripting
- D7 proposed: how the REPL detects and buffers multi-line block input
- Fifth silent-drop site found (Statement::Emit); friction-9 is systemic
- is_literal heuristic and eval_condition simplicity are v9-extension points
v9 supports two paths for entering multi-line blocks at the REPL:
**Default: continuation prompt.** When fsh detects an unclosed brace
(e.g., user types `parallel {` and hits Enter), it shows a continuation
prompt and accepts lines until the matching close brace appears. Then
dispatches the accumulated source to `scripting::run_source`. Familiar
pattern -- every bash/zsh/fish user already knows it.
**Explicit: `:block` command.** User invokes an explicit multi-line
editor mode for full editing (arrow keys, backspace across lines, etc.).
Terminates with Ctrl-D or `:run`. Preferred path for longer or more
complex blocks.
**Rationale.** Continuation alone taxes power users who need to fix
earlier lines in a long block. Explicit mode alone taxes quick blocks
with unnecessary ceremony. Both costs little once the parser path via
`run_source` is wired. Ships continuation first (common path), `:block`
second (power path).
Implementation phases:
- First: continuation prompt. Unclosed-brace detection in main.rs REPL
  loop. Buffer lines until close brace. Dispatch via `run_source`. This
  unlocks `parallel { }` at the prompt.
- Second: `:block` explicit command. Opens an inline multi-line editor
  (likely reuses rustyline multi-line mode or similar). Same dispatch
  target.
Both share the same parser (scripting::parse) and executor (run_source).
No duplication. The difference is purely input surface.
---
**Role.** ~7000 lines. Contains the main dispatch function `execute(line, db, core_root)`
and all fsh builtin implementations. Not read end-to-end in this audit --
targeted sections only.
```rust
pub fn execute(line: &str, db: &ForestDb, core_root: &str) -> CommandResult
```
Three parameters, no ExecContext. D1 signature change replaces `line`
with `ctx: &ExecContext`. Every caller site (internal recursive + external
across main.rs, exec.rs, scripting.rs) updates.
Lines 153-170 define a local `tokenize_args` function inside `execute`.
This duplicates `ExecContext::from_line`'s `tokenize` closure at
`exec.rs:52-69`. Same logic, two implementations. When D1 threads ctx
in, this local function is removed -- tokenization happens once in ctx
construction, not again in dispatch.
Lines 171-176 re-parse what ExecContext already knows:
```rust
let trimmed_line = line.trim();
let cmd = trimmed_line.splitn(2, ' ').next().unwrap_or("").to_lowercase();
let rest_str = trimmed_line.splitn(2, ' ').nth(1).unwrap_or("");
let owned_args: Vec<String> = tokenize_args(rest_str);
```
With D1 in place, these six lines become `let cmd = &ctx.cmd; let args = &ctx.args;`.
The parsing debt paid once at ctx construction replaces parsing debt
repaid at every dispatch call.
Alias expansion happens at line 190-198 **inside** `execute()`:
```rust
if let Some(aliased) = db.get_alias(&cmd) {
    let expanded = ...;
    return execute(&expanded, db, core_root);  // recurse
}
```
This means the outer `execute_with_context` builds ExecContext on the
raw (pre-alias) command, dispatches, and only THEN does alias expansion
happen. The ExecContext's `cmd` and `args` fields reflect the raw cmd,
not the expanded cmd.
**ExecContext has an `expanded: String` field at `exec.rs:26`** designed
exactly for post-alias content, but `from_line` sets `expanded = raw`
(line 81) and never updates it because alias expansion is a concern of
`execute()`, not `exec_with_context`.
**v9 direction:** move alias expansion UP into `execute_with_context`
before ctx dispatch. ctx then carries both raw (user's literal input)
and expanded (post-alias, what actually dispatches). This keeps alias
expansion as a single-point concern rather than a recursive re-entry.
Five recursive `execute(...)` sites inside commands/mod.rs:
- Line 183: `!!` repeat last command
- Line 197: alias expansion recurse
- Line 212: plugin resolution recurse (block not read)
- Line 1400: edit-then-execute builtin ($EDITOR flow)
- Line 6349: pipeline-internal recurse (context unread)
Each needs ctx forwarding under D1. Proposed helper: `ctx.reparse(new_line)`
returns a new ExecContext with updated raw/cmd/args, preserving intent,
cwd, timestamp from parent ctx. Five-site mechanical change.
`fsearch "Command::new(\"sh\")"` returned zero results. The complete
sh-delegation surface in fsh is in main.rs (12 sites, Section 4.1).
Commands are implemented natively. D2 commitment to sh-free v9 is
contained -- no additional audit scope for commands/mod.rs on this front.
`fsearch ".ok();"` found 24 sites. Categorization:
**Not silent-drop bugs -- legitimate `.ok()` uses (~9 sites):**
- `stdout().flush().ok()` (lines 628, 1437) -- flush is best-effort
- `stdin().read_line().ok()` (line 630) -- input fallthrough
- `.parse::<usize>().ok()` (line 6455) -- Result-to-Option conversion
- `fs::metadata(&path).ok()` (line 6738) -- optional existence check
- `read_dir(...).ok()` (line 7292) -- optional directory check
- `Connection::open(...).ok()` (line 1283) -- optional fallback connection
- Two others (lines 6660, 6707) likely similar Option conversions
**Potential silent-drop bugs -- `.ok()` on SQL writes (estimated ~15 sites):**
Lines 38, 3593, 3597, 3636, 3663, 3855, 4241, 4257, 4279, 4608, 5570,
5690, 6063, 6135, 6953. First-pass heuristic -- each looks like
`).ok();` at the end of what appears to be a SQL execute statement.
**Each requires individual verification** before being flagged as a
friction-9 sibling; some may turn out to be legitimate best-effort
writes (e.g., optional cache updates where failure is acceptable).
**Honest total across all audited files (with commands/mod.rs estimate):**
- db.rs: 4 confirmed silent-drop sites
- scripting.rs: 1 confirmed silent-drop site
- commands/mod.rs: up to 15 potential silent-drop sites (unverified)
Worst case: ~20 silent-drop sites across 3 files. Best case (if all
15 commands/mod.rs sites turn out to be legitimate Option conversions):
5 sites -- which itself is still systemic.
1. **D1 signature change** -- thread ctx through execute, remove duplicated
   tokenizer, stop re-parsing at dispatch entry. Five recursive sites
   update via ctx.reparse(). Mechanical change across function headers
   and recurse points.
2. **Alias expansion migration** -- move up to execute_with_context.
   Remove lines 190-198 from execute() body. ExecContext.expanded gets
   populated correctly.
3. **.ok() site verification** -- each of the 15 commands/mod.rs candidates
   reviewed individually. Real silent-drops converted to error-returning
   mutations. Legitimate Option conversions left alone.
4. **Individual builtin signatures** -- any builtin that currently takes
   just `args` but would benefit from ctx (intent-aware, timestamp-aware,
   or cwd-aware behavior) gets the upgraded signature.
- execute() signature is (line, db, core_root); D1 threads ctx to replace line
- Tokenizer duplicated across exec.rs and commands/mod.rs; v9 unifies
- Five recursive execute() sites inside commands/mod.rs; manageable count
- Alias expansion lives at wrong architectural layer; v9 moves up
- No sh -c inside commands/mod.rs; sh-delegation surface is main.rs only
- 24 `.ok();` sites; ~9 legitimate, up to 15 potentially silent-drop bugs
- Silent-drop pattern is systemic, not isolated to db.rs
---
Consolidated from Sections 4.4.7 (spawn subsystem) and 5.4.2 (block parser
extension). This section states Pillar 1's full requirements in one place.
v9 Pillar 1 introduces two user-visible constructs:
**Block form:**
parallel {
deploy core
deploy faelight-shell
deploy faelight-term
}
parallel! {
cargo check
cargo clippy
cargo fmt --check
}
**Operator form:**
deploy core ||| deploy faelight-shell
The block form is primary. The operator form is syntactic sugar for a
two-command parallel block.
For `parallel { a; b; c }` typed at the REPL:
1. **REPL input layer (main.rs):** Detect unclosed brace on input line.
   Buffer lines using continuation prompt (D7). On matching close brace,
   dispatch accumulated source to `scripting::run_source`.
2. **Scripting parser (scripting.rs):** Recognize `parallel {` / `parallel! {`
   prefix. Delegate body collection to existing `collect_block`. Produce
   `Statement::Parallel { body, strict }`.
3. **Scripting executor (scripting.rs):** For each body statement, build
   a `Job` via the builder. Spawn concurrently. Wait according to strict
   flag (best-effort or fail-fast).
4. **Job subsystem (new rust-tools/faelight-shell/src/jobs.rs or similar):**
   Each spawn goes through preexec with ExecContext. Child is launched
   with `StdoutMode::LabeledStream("[name]")` and `StderrMode::LabeledStream("[name]")`.
   JobHandle returned. Jobs table row inserted.
5. **Observation layer (exec.rs):** Preexec fires once per parallel
   child. Postexec fires once per parallel child on completion. Each
   child is observed independently -- safety rules apply, history records,
   knowledge queries run on individual failures.
6. **Output surface (terminal via labeled streams):** Per-child output
   is captured in reader threads, label is prepended to each line
   ("[core] Building..."), emitted to terminal. No interleaving.
Summary of new types, functions, and modifications:
**New types (Section 4.4.1-4.4.3):**
- `Job<'a>` builder with stdin/stdout/stderr modes, label, observed, observed_stream
- `JobHandle` with wait/wait_timeout/kill/status/id
- `JobResult` with exit_code/signal/captured stdout/stderr/duration_ms/label
- `JobTable` holding active handles in-memory
- `StdinMode`, `StdoutMode`, `StderrMode`, `JobStatus`, `JobError` enums
**New schema (Section 4.4.4):**
- `shell_jobs` table in state.db (D4 locates migration in db.rs)
**New statement type (Section 5.4.2):**
- `Statement::Parallel { body: Vec<Statement>, strict: bool }`
**Parser extension (scripting.rs::parse):**
- One new branch for `parallel {` / `parallel! {` -- ~15 lines
**Executor extension (scripting.rs::run_stmt):**
- One new match arm for Statement::Parallel -- spawn each body via Job
  builder, collect handles, wait according to strict flag, emit results
**REPL integration (main.rs):**
- Unclosed-brace detection
- Continuation prompt buffering
- Dispatch to `scripting::run_source` on matching close
- Later: `:block` explicit editor command (D7 second phase)
**Operator parsing (main.rs):**
- `|||` operator detection before ` | ` pipe detection
- Splits into parallel body, dispatches via same path as block form
Parallel execution does NOT require rewriting the following -- they
already do what v9 needs:
- `collect_block` (scripting.rs:135) -- block body collection with depth
  tracking. Works for parallel blocks unchanged.
- `scripting::parse` and `run_stmts` -- statement parse and execute loop.
  Adding one statement variant is additive, not structural.
- `scripting::run_source` -- inline source execution. Already marked
  dead_code; D7 wires it up.
- Rustyline's native pipe loop (for the underlying process spawn
  mechanics) -- the new Job subsystem uses `std::process::Command` the
  same way the pipe loop does, just with builder API wrapping it.
Once the Job subsystem exists for parallel, several adjacent capabilities
come effectively for free:
- **`jobs` / `wait` / `cancel` / `job-log` builtins (Pillar 1 job control)**
  become queries against the jobs table plus JobHandle operations.
- **Background jobs (`&` suffix)** migrate from jobs.rs's BgJob (D3 renamed
  legacy) to the new Job system with observed=true. A single-command
  parallel block, essentially.
- **Labeled stderr capture** for any command, not just parallel blocks.
  `cargo build 2>&1` observed by Friday instead of lost to inherit.
- **Mid-flight knowledge surfacing** (future Pillar 2 capability) -- when
  captured stdout contains a known error pattern, Friday can emit a
  hint WHILE the command still runs.
To scope Pillar 1 honestly:
- **Auto-parallel detection** -- INT-245 intent describes this, but it's
  a separate subsystem that sits ABOVE the scripting layer, analyzing
  sequences of user commands for dependencies before rewriting them
  into parallel blocks. Not blocked by the Job subsystem; complementary
  but separate. Probably its own later intent.
- **Cross-host parallel** -- no distributed execution. Single machine only.
- **Arbitrary nesting** -- `parallel { parallel { a; b }; c }` is allowed
  by the recursive parser but has subtle semantics (which child is
  "observed" with which context). v9 ships with shallow nesting supported
  but deep nesting reserved for later.
- **Resource-aware throttling** -- Pillar 3 "resource awareness" is a
  different concern that can layer on top of Job spawn as a policy,
  not built-in.
Phase 3 of the v9 implementation plan (Section 4.4.8) is "add parallel
block parse and execution." It ships when:
1. `parallel { a; b; c }` typed at the REPL dispatches via continuation
   prompt → run_source → Statement::Parallel
2. Three children spawn concurrently (verified via timing -- total time
   is max(individual times), not sum)
3. Each child's output is labeled with the command name
4. Each child is observed independently (preexec fires, postexec fires,
   history records each)
5. Best-effort failure handling: one child fails, others continue,
   results report per-child status
6. `jobs` builtin lists the running parallel block's children by id + label
7. `cancel <id>` kills a specific child; siblings continue
Phase 4 adds `parallel! { }` fail-fast. Phase 5 adds `|||` operator.
- Parallel execution is a consolidation of Sections 4 and 5 machinery,
  not a new design layer
- Existing block parser + existing pipe process launch + new Job subsystem
  + new Statement variant + REPL continuation = complete Pillar 1
- Labeled output requires Stdio capture (not inherit); D2 Axis C hybrid
  observation makes this policy-based rather than always-on
- Observation of parallel children is per-child -- preexec and postexec
  fire independently for each, meaning safety rules, history, and
  knowledge queries all work on individual parallel commands
- Several adjacent capabilities (jobs control, labeled stderr, mid-flight
  hints) unlock for free once Job subsystem exists
- Auto-parallel detection is NOT Pillar 1; it's a separate subsystem for
  a later intent
---
INT-245 Pillar 6 lists 10+ friction items. Audits sometimes treat such
lists as a patch queue. This audit treats them as diagnostics. Each
friction item is a symptom; the symptoms cluster around a small number
of architectural problems. Section 7 names each thread once, maps the
symptoms to it, and states v9's structural correction.
The threads, summarized:
- Thread A: Observation gap -- execution bypasses the intelligence layer
- Thread B: Silent-drop pattern -- mutations discard errors quietly
- Thread C: Context loss -- typed data becomes raw strings at boundaries
- Thread D: Ad-hoc parsing -- same logic implemented multiple times
- Thread E: Unwired extension points -- plug-in surfaces exist but are empty
- Thread F: Friction-capture absence -- the shell does not remember its own bugs
**The problem.** `exec::execute_with_context` (exec.rs:458) is the
documented single entry point to the intelligence layer (preexec safety
rules, postexec history and knowledge). Five execution paths exist
(Section 3.1). Only one goes through execute_with_context. The other four
bypass it entirely.
**Concrete impact.** Most non-trivial shell activity -- pipelines with
external commands, redirects, sh fallbacks, stderr-only operations --
runs unobserved. Friday cannot see these. shell_history does not
capture them via the ctx path. Safety rules do not fire on them.
**Friction items that are symptoms of Thread A:**
- Friction item 5 ("grep | in pattern -- unquoted | treated as pipe"):
  the pipe-parser split happens before ctx is built, so ctx-aware
  corrections cannot apply.
- Friction items 7 and 10 (redirect friction): redirects bypass
  execute_with_context (Section 3.2), so even if detect_redirect returns
  a bogus target, no safety rule fires that might catch it.
- Implicit: every friction item that describes a pipeline or redirect
  failure lives in code paths where observation is absent.
**What v9 does.** D1 + Section 4.4.4 jobs table = every execution path
routes through the same observation gateway. The central `Job` builder
calls preexec before spawn, records the row, runs the child with
appropriate stdio modes, calls postexec after wait. Pipelines become
N sequential or parallel Job spawns. Redirects become Job spawns with
StdoutMode::ToFile. sh fallback is migrated or replaced. Thread A is
the dominant architectural correction in v9 Phase 0-2.
**The problem.** Database mutations use three different conventions
across fsh: returning Result (honest), returning bool via is_ok()
(honest shorthand), and dropping the Result via `.ok()` or `let _ = ...`
(silent). The silent variant is scattered through the codebase
inconsistently.
**Confirmed sites from audit:**
- db.rs: 4 sites (save_history_entry, set_focus_intent, set_theme, clear_focus_intent)
- scripting.rs: 1 site (Statement::Emit events INSERT)
- commands/mod.rs: up to 15 candidates from first-pass `.ok();` grep,
  individual verification required
**Concrete impact.** When state.db hits a transient failure (stale
connection after schema migration, lock contention, disk pressure),
silent-drop sites fail invisibly. The user sees success messages,
cistart appears to work, theme appears to change, emit appears to record
-- but the underlying state never updated. INT-234 gate 8 debugged this
the hard way.
**Friction items that are symptoms of Thread B:**
- Friction item 9: the originally-reported `save_history_entry` bug.
  Audit revealed this is one of at least 5 confirmed sites, with more
  candidates in commands/mod.rs.
**What v9 does.** Every mutating method returns a Result. Every caller
either handles the Result or uses `?` to propagate. `.ok()` is permitted
only for genuine best-effort operations (stdout flush, Option-conversion
for parsing) -- documented at each site. A codebase-wide pass enforces
this. The architecturally-wrong `let _ = db.conn.execute(...)` pattern
is eradicated. Invariant
**The problem.** ExecContext is a typed description of every command
execution (exec.rs:22-39). It is constructed correctly at
`exec::execute_with_context`. It is then discarded at dispatch: line 468
passes the raw `line` string, not ctx, to `commands::execute`. The
typed data becomes untyped at the layer boundary.
**Concrete impact.**
- `commands/mod.rs:152-176` re-parses cmd and args from the raw string,
  duplicating work ctx already did.
- Alias expansion inside `execute()` (lines 190-198) recurses with a
  raw string, rebuilding context at each layer.
- Builtins that want to reason about intent, timestamp, or cwd either
  re-fetch from db or work without these inputs.
- The `in_pipeline: bool` field on ExecContext is never set to true
  anywhere because pipelines don't route through the context-building
  path at all (Section 2.6).
- The `expanded: String` field exists for post-alias content but is
  never populated because alias expansion happens inside execute(),
  not at ctx construction.
**Friction items that are symptoms of Thread C:**
- Friction item 11 (multi-line paste splits wrong): proper paste
  handling needs ctx-aware state (is this a continuation of a block?
  is this a heredoc body?). The current raw-string dispatch has no
  place for that state.
- Indirect: every friction item where "the shell guessed wrong about
  what the user meant" traces partly to context loss -- the dispatcher
  knows less than the typed data available to it.
**What v9 does.** D1 option (A) threads ctx through. `execute(ctx, db, core_root)`
replaces `execute(line, db, core_root)`. Duplicated tokenizer removed.
Alias expansion moves up to execute_with_context (Section 5.5.4). The
five recursive execute sites update via `ctx.reparse(new_line)`.
ExecContext.in_pipeline and ExecContext.expanded become meaningful because
they're populated at correct layers. Thread C is the dominant architectural
correction running through Phase 0 of v9 implementation.
**The problem.** The same parsing logic is implemented multiple times
across fsh. Discovered instances:
- Tokenizer: `ExecContext::from_line`'s tokenize closure (exec.rs:52-69)
  AND `commands::execute`'s tokenize_args (commands/mod.rs:153-170).
  Two implementations of quote-aware tokenization.
- Pipeline splitting: `main.rs:1269-1274` uses naive `contains(" | ")`
  AND `value::parse_pipeline` does richer parsing. Dual paths.
- Redirect detection: `detect_redirect` at `main.rs:151-183` does
  prefix-guard-based matching; no alternative parser exists today, but
  the function cannot be extended without touching all redirect shapes
  in one place.
**Concrete impact.** Bugs in one implementation don't surface in the
other. Fixes to one don't propagate. New features (like `|||` operator)
require deciding which parser to extend -- or duplicating further.
**Friction items that are symptoms of Thread D:**
- Friction item 5 (grep | in pattern): pipe splitting is the naive
  variant in one path, richer in another. Which one applies depends
  on which dispatch branch fires first.
- Friction items 7 and 10 (redirect friction): detect_redirect is the
  only implementation; extending it for v9 `>>` edge cases requires
  reviewing every caller that uses its return values (Section 3.2).
**What v9 does.** Unify the tokenizer as a single function called once
at ctx construction. Extend `value::parse_pipeline` to handle `|||`
alongside `|`. Treat `detect_redirect` as a single-source-of-truth
parser for stdout/stderr/append/merge cases and fix the edge cases with
live reproducers (Section 3.5.1). No new parser ad-hoc created; existing
parsers extended or unified.
**The problem.** Trait implementations for rustyline hooks exist but
are empty or pass-through (Section 5.3.6-5.3.7):
- `Hinter::hint` -- empty (line 659-661). Rustyline's inline ghost-text
  surface, exact plug point for Tab-to-accept (Pillar 2).
- `Highlighter::highlight` -- returns line unchanged (line 664-666).
  Rustyline's syntax-coloring surface, plug point for CHALLENGE-level
  visual warnings (Pillar 5).
- `Validator::validate` -- empty (line 677). Rustyline's Enter-is-complete
  check, plug point for pre-execution syntactic validation (unclosed
  brace detection for parallel blocks, heredoc delimiter checks).
**Concrete impact.** Pillar 2 (Intelligent Execution) and Pillar 5
(Friday Deep Integration) both reference input-time intelligence --
"Tab accepts Friday suggestion," "fsh interrupts you before executing
dangerous command." Neither has a surface today. The rustyline traits
are the exact surface -- they exist, they're wired to the REPL, they're
empty.
**Friction items that are symptoms of Thread E:**
- Friction item 3 (heredoc RSEOF contamination): a Validator could
  warn at Enter time when heredoc delimiters look malformed, before
  the command runs and corrupts output.
- Future v9 features named but not bug-list'd: stuck detection, command
  prediction, CHALLENGE-level interrupts. All need input-time surfaces.
**What v9 does.** Populate Hinter with friday-aware hints gated on
Phase 28 thresholds (confidence/occurrences/accuracy), Highlighter with
risk-aware coloring, Validator with syntactic pre-checks for v9
constructs. These are additive changes -- they fill empty functions with
content. Not structural rewrites.
**The problem.** Friction items 7, 8, 10, and 11 from INT-245 cannot
be traced from architecture reading alone (Section 3.5.1). They require
live reproducers: the exact input string that triggered the failure,
captured at the time of failure. Today, when these bugs fire, fsh emits
a generic error (or worse, silently creates a junk file) and moves on.
The user is expected to remember. Christian named this honestly during
audit: "it is difficult to keep with every little thing."
**Concrete impact.** Bugs that cannot be reproduced cannot be fixed
with confidence. Every session, friction items 7/10/11 may fire; by
the time the user returns to fix them, memory of exact input is gone.
The bug persists indefinitely.
**Friction items that are symptoms of Thread F:**
- Friction item 7 (`>>` append redirect): unverifiable from reading;
  awaiting reproducer.
- Friction item 8 (python3 -c arg size): documented with workaround
  but not root-caused.
- Friction item 10 (redirect junk files): audit could not explain the
  `=68`, `=69`, `=257` artifacts from reading alone.
- Friction item 11 (multi-line paste): hard to reproduce reliably because
  "it depends on what got pasted."
**What v9 does.** When a redirect parse fails, a sh fallback fires, a
heredoc contamination is detected, or any Pillar 6 friction pattern
triggers -- fsh writes a row to a new `shell_friction` table capturing:
- Exact input string (raw bytes, not processed)
- Timestamp
- cwd
- Terminal capabilities at time of failure (paste mode, line length, etc.)
- Best-effort classification of the failure pattern
- Session ID linking to shell_history entry
The user never has to remember. Christian's cognitive load reduces.
The bug's reproducer is right there in state.db. Next session begins
with "three friction events recorded since last session; review?"
This is infrastructure for Christian, built automatically. Not a feature
added to the checklist. It is the shell remembering its own pain so
Christian doesn't have to.
From Section 4.4.8 phase breakdown:
- **Phase 0 (Job foundation):** Thread A (start), Thread B (db.rs pass),
  Thread C (ctx threading), Thread D (tokenizer unification)
- **Phase 1 (user-command migration):** Thread A (complete for
  non-parallel cases)
- **Phase 2 (sh delegation migration):** Thread A (complete for all cases)
- **Phase 3 (parallel blocks):** Thread A (per-child observation for
  parallel), Thread D (parser extension for `parallel {`)
- **Phase 4 (parallel! fail-fast):** additive to Phase 3
- **Phase 5 (`|||` operator):** Thread D (parser extension)
- **Phase 6 (jobs builtins):** queries against shell_jobs table
- **Ongoing across phases:** Thread B (silent-drop pass on commands/mod.rs)
- **Parallel effort -- Pillar 2/5 enablement:** Thread E (populate Hinter,
  Highlighter, Validator)
- **Parallel effort -- infrastructure for Christian:** Thread F
  (shell_friction table, auto-reproducer capture)
INT-245 Pillar 6 currently lists 10+ items as a patch queue. After
this audit, they become:
- **Thread A symptoms** (friction 5, 7, 10 partially) -- resolve as
  Phase 0-2 observation rewiring
- **Thread B symptoms** (friction 9 and its siblings) -- resolve as
  Phase 0 db.rs pass + ongoing commands/mod.rs verification
- **Thread C symptoms** (friction 11) -- resolve as D1 ctx threading
- **Thread D symptoms** (friction 5, 7, 10 partially) -- resolve as
  parser unification
- **Thread E symptoms** (friction 3 partially) -- resolve as Thread E
  population
- **Thread F symptoms** (friction 7, 8, 10, 11 -- anything requiring
  reproducer) -- resolve as shell_friction table
- **Still individual-fix items** (friction 1, 2, 4, 6) -- small, contained
  patches to specific commands/documents; not architectural
**The 10 friction items are not 10 separate fixes.** They are
~4-5 architectural corrections plus a handful of individual patches.
v9 ships the corrections; the friction items close as side effects.
- Friction items cluster into six architectural threads (A-F)
- Threads A, B, C are the dominant v9 Phase 0 corrections
- Thread D (parser unification) is mid-phase ongoing work
- Thread E (rustyline plug points) is Pillar 2 and 5 enablement
- Thread F (friction-capture) reduces Christian's memory burden
  permanently; self-funding infrastructure
- Individual friction patches (items 1, 2, 4, 6) remain as small
  contained work, not tied to the threads
- The v9 implementation plan aligns 1:1 with the threads -- no orphan
  work, no unclaimed gaps
---
Consolidates every decision point and open item surfaced during the
audit. Status tags:
- **LOCKED** -- decision made, rationale recorded, implementation can proceed
- **PROPOSED** -- design written, awaiting Christian's explicit sign-off
- **OPEN** -- question stated, no resolution yet, may block a phase
- **DEFERRED** -- acknowledged but scoped out of v9
**D1 -- Context threading: Option (A).** Thread `&ExecContext` through
`commands::execute`. ~7000 lines of mechanical parameter addition plus
~15-20 re-parse sites that collapse into structured ctx reads. Phase 0
mandatory foundation. See D1 record and Section 1.5.
**D2 -- Spawn subsystem: four axes locked.**
- Axis A: Builder pattern (`Job::new(...).arg(...).stdout(...).spawn()`)
- Axis B: Hybrid jobs storage (in-memory active + state.db completed)
- Axis C: Hybrid observation (wrap-only default, mid-flight opt-in)
- Axis D: Policy-per-block (`parallel { }` best-effort, `parallel! { }`
  fail-fast)
See D2 record and Section 4.4.
**D3 -- Job type: Path II (replace with rename).** Current `jobs.rs::Job`
becomes `BgJob` (legacy background tracking). New `Job` builder per
Section 4.4.1 becomes canonical. Phase 0.A: rename + new type construction.
See D3 record and Section 5.1.2.
**D4 -- `shell_jobs` migration lives in db.rs.** fsh owns the schema for
its jobs table, alongside shell_history/shell_aliases/shell_state. Core
can query but does not own. See D4 record and Section 5.2.3.
**D7 -- REPL multi-line block input: both modes.** Continuation prompt
is default (familiar, matches bash/zsh/fish). Explicit `:block` mode
is power path for longer blocks with full editing. Both share parser
via `run_source`. Ships continuation first, `:block` second. See D7
record and Section 5.4.3.
**D5 -- Completion command lists: introspection-based generation.**
Replace the 85-entry COMMANDS array, 10-entry PIPE_OPS array, and
150+-entry MULTI_CMDS array (completion.rs) with auto-generation from
the actual dispatch table and core engine command registry. Eliminates
drift (duplicates already visible: `theme` at line 68/76, `q` at line 82/98).
Non-blocking for Phase 0-3; ideal for a Phase 4 or 5 implementation
window. See Section 5.3.4.
**Status:** Proposed. No implementation blocker until v9 Phase 4+.
Decision can be made at that time.
**D6 -- Completion cache for per-tab I/O.** Current Tab press can do
filesystem read (intent IDs), process spawn (git branch), and fresh
SQLite connection (alias lookup). A `CompletionCache` struct held by
ForestHelper would reduce cost to first-tab-per-session for most operations.
See Section 5.3.5.
**Status:** Proposed. Performance improvement, not correctness. Non-blocking
for v9 correctness gates.
**OQ-1 -- Friction item 7 root cause.** `cat file >> target` reportedly
fails with "No such file or directory" but the detect_redirect function
reads correctly for this case. Cannot be resolved without live reproducer.
See Section 3.5.1.
**Resolution path:** When the bug fires again, capture the exact input
string into shell_friction table (Thread F infrastructure). Trace
detect_redirect and downstream redirect handling with real values.
**OQ-2 -- Friction item 10 root cause.** Junk files like `=68`, `=69`,
`=257`, and a SQL-fragment filename were created in the repo root,
apparently by fsh redirect parsing failures. The detect_redirect guards
should prevent this; cannot determine from reading alone.
**Resolution path:** Same as OQ-1 -- requires reproducer via shell_friction.
**OQ-3 -- `.ok();` sites in commands/mod.rs.** ~15 candidates identified
by first-pass grep. Some are legitimate best-effort operations, some
are likely silent-drop bugs (Thread B). Individual verification required.
**Resolution path:** During Phase 0 Thread B cleanup pass, review each
site with context. Classify legitimate vs silent. Convert silent to
error-returning. Document legitimate uses inline.
**OQ-4 -- value::parse_pipeline internal behavior.** Section 3.5 flagged
this function as called but not read. Determines classification between
External and internal pipe ops. Misclassification routes commands to
the wrong execution path.
**Resolution path:** Read during Phase 0 Thread D parser work. Not
blocking Phase 0 if classification behavior holds under existing tests.
**DF-1 -- Auto-parallel detection.** INT-245 intent text describes
Pillar 1 including automatic parallelization of independent sequential
commands. Section 6.6 scoped this out of v9: it's a separate subsystem
that sits above the scripting layer, analyzing dependencies. Legitimate
future intent, not blocked by v9.
**DF-2 -- Cross-host parallel execution.** Not v9 scope. Single machine
only.
**DF-3 -- Deep nesting of parallel blocks.** Scripting parser recursively
handles nested blocks, but observation semantics for `parallel { parallel { }; }`
have subtle concerns (which child sees which ctx). v9 ships with shallow
nesting supported; deep nesting reserved.
**DF-4 -- Async-first shell architecture.** Axis A considered this
(Section D2). Rejected for v9 as too large a leap. Future consideration
if the Job subsystem's synchronous model hits limits.
These are the small patches Christian explicitly asked to collect for
post-audit execution:
- **P1:** completion.rs line 68/76 duplicate `theme` -- remove one entry
- **P2:** completion.rs line 82/98 duplicate `q` -- remove one entry
- **P3:** db.rs four silent-drop sites -- convert to error-returning mutations
  uniformly (save_history_entry, set_focus_intent, set_theme, clear_focus_intent)
- **P4:** scripting.rs line 263 silent-drop on Emit event INSERT --
  convert to error-returning or record to shell_friction on failure
- **P5:** Any additional duplicates found in other audits (Section 5
  recommended a scan of all hardcoded arrays in future audits)
**Status:** Queued for execution after audit document commits. Each is
a small cohesive patch. No architectural dependencies between them.
- 5 decisions LOCKED, ready for implementation
- 2 decisions PROPOSED, non-blocking
- 4 open questions require future data or phase-context decisions
- 4 items explicitly deferred out of v9 scope
- 5 post-audit patches queued as separate work from architectural v9
---
**Session:** 2026-04-24, single sitting
**Duration:** ~2.5 hours of focused read+write
**Source commit at audit start:** 61b7e783 (v11.9.0 pre-tag, INT-232 peak)
**Intent state:** INT-245 (fsh v9) active, checkpoint auto-recorded at start
A 1600+ line architectural document mapping the current fsh execution
layer, identifying where v9 pillars plug in, and committing to a set
of implementation decisions before any v9 code is written.
**Code files read end-to-end:**
- exec.rs (474 lines) -- execution pipeline foundation
- jobs.rs (168 lines) -- current background job control
- db.rs (290 lines) -- state.db facade and silent-drop discovery
- completion.rs (678 lines) -- rustyline hook surfaces
- scripting.rs (482 lines) -- .fsh language parser and executor
- detect_redirect function (33 lines) -- mini-audit for friction 7/10
**Code files read selectively (targeted greps + critical sections):**
- main.rs -- pipeline execution, dispatch branching, Command::new sites,
  execute_with_context call sites, detect_redirect, redirect handling
- commands/mod.rs -- dispatch signature, recursive call sites, .ok() surface
**Total source lines inspected:** approximately 3,400 lines across 7 files.
**v9 Phase 0 -- Foundation (infrastructure only, no user-visible features).**
All blockers resolved:
- D1 locked -- ctx threading approach committed
- D2 locked -- Job builder shape, storage model, observation scope,
  failure policy all committed
- D3 locked -- Path II (replace with rename) committed
- D4 locked -- shell_jobs lives in db.rs
- D7 locked -- REPL block input has both modes
**v9 Phase 1-6 -- ready in sequence.** Section 4.4.8 phasing stands:
Phase 1 migrates 2 user-command launch sites; Phase 2 migrates 12 sh
delegations; Phase 3 adds parallel block parse+execute; Phase 4 adds
fail-fast; Phase 5 adds `|||` operator; Phase 6 adds jobs builtins.
**Post-audit patches -- ready (P1-P5).** Section 8.5 lists five small
contained patches that can ship before or alongside v9 Phase 0:
- P1, P2: completion.rs duplicates
- P3: db.rs four silent-drop sites
- P4: scripting.rs Emit silent-drop
- P5: further duplicates scan as found
**Not read:** commands/mod.rs end-to-end (~7000 lines remaining),
triggers.rs, main.rs non-execution sections (prompt rendering, input
handling, startup), value.rs, schema.rs, config.rs, and any other
modules not touched by the execution layer investigation.
**Not produced:** an implementation plan with date estimates, gate
acceptance criteria per phase, test plans, or regression risk analysis.
Those follow from this audit as separate work. The audit says "this
is where things plug in"; the implementation plan says "here is how
we land each phase."
**Not fixed:** any code. This was a read-and-document pass. Post-audit
patches (P1-P5) are queued but not executed.
When Christian returns from the 3-day break:
1. **Read this audit document.** Section 7 (architectural threads) first,
   then Section 4.4 (spawn subsystem), then Section 8 (decisions). That
   is the minimum re-orientation to resume work.
2. **Commit the audit doc itself.** Before any code changes. This document
   is a primary artifact of INT-245 Phase 0.
3. **Execute post-audit patches (P1-P5).** Small, contained, each a
   separate commit. Run `d` between to confirm health. These are the
   "momentum entry" work -- ~30-60 min of honest small wins.
4. **Begin Phase 0 foundation work.** The `Job` type, JobHandle, JobResult,
   JobTable, shell_jobs migration, and BgJob rename. Pure infrastructure.
   Multiple sessions expected. No user-visible feature ships here, by
   design.
5. **Update INT-245 gates.** Pillar 6 friction items fold into Threads
   A-F architectural corrections per Section 7. The gate list in the
   intent file should reflect this -- some friction items close "as a
   consequence of Thread X." This makes the intent's progress tracking
   match the actual work.
**The audit is a read, not a proof.** It states what the code appears
to do based on careful reading. Compiler and runtime behavior occasionally
surprise. The architectural conclusions stand, but specific line-level
claims (e.g., "this if-branch is unreachable") can be wrong and should
be verified when the relevant area is touched.
**Friction items 7 and 10 remain unresolved.** Section 3.5.1 honestly
flagged that detect_redirect reads correctly for the reported failure
cases. Without live reproducers (Thread F infrastructure), these cannot
be root-caused. The audit does not pretend otherwise.
**Silent-drop count in commands/mod.rs is an estimate.** Section 5.5.7
did not verify each of the 15 candidates individually. The worst-case
total (20 sites across 3 files) is a ceiling, not a confirmed count.
**Time estimates for phases are absent deliberately.** Sections 4.4.8
and 8.1 describe phase ORDERING but not durations. Christian's working
rhythm (14-15 days/month) and the multi-session nature of Phase 0 make
date estimates unreliable. Phases will land when they land.
This audit exists because the alternative -- "start fixing and improving
everything we can" -- would have led to v9 patching around the same
architectural problems that produced the friction items in the first
place. The audit took one session. It will save weeks.
The most important finding is not any single decision. It is that the
friction items in INT-245 Pillar 6 are not 10 bugs. They are ~4-5
architectural patterns (Threads A-F, Section 7) manifesting as 10
visible failures. v9 corrects the patterns. The failures close as
consequences.
This is the "complete job" Christian committed to. Foundation first.
Features on foundation. No patching. No routing around.
-- End of Audit A.

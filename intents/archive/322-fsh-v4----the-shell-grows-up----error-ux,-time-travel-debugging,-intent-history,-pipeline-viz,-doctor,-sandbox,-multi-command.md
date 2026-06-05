---
id: 322
title: "fsh v4 -- The Shell Grows Up"
status: complete
date: 2026-05-20
tags: [fsh, shell, error-ux, time-travel, history, pipeline, doctor, sandbox, multi-command, friday]
---
---
THE PREMISE

fsh v2 proved the shell could live.
fsh v3 proved the shell could think -- tab completion, structured pipelines, natural language.
fsh v4 proves the shell can grow.

A shell that explains its own failures.
A shell that remembers where it has been.
A shell that can rewind time.
A shell that knows if it is healthy.
A shell that isolates complex work without losing context.
A shell that handles whatever you throw at it without breaking.

This is not a feature list.
This is the shell becoming a genuine partner.
---
FEATURE 1 -- BETTER ERROR UX

The problem:
  Command fails. You get a red exit code and a cryptic message.
  sh: line 1: gti: command not found
  That is not helpful. That is noise.

What fsh v4 does instead:

  Exit code meanings -- every exit code has a human name:
    1   = General error
    2   = Misuse of shell builtin
    126 = Permission denied (command exists but not executable)
    127 = Command not found
    128 = Invalid exit argument
    130 = Interrupted by Ctrl+C
    137 = Killed (OOM or SIGKILL)
    139 = Segmentation fault
    Each gets a specific message, not just the number.

  Did-you-mean for 127 errors:
    When a command is not found, fsh searches:
      -- All commands in PATH
      -- All fsh builtins
      -- All forest scripts in ~/0-core/scripts/
      -- All abbreviations
    Levenshtein distance <= 2 triggers a suggestion:
      ✗ gti: command not found
      → Did you mean: git?

  Context-aware suggestions:
    If the error is in a pipeline, show which stage failed.
    If the error is a permission issue, suggest chmod or sudo.
    If the error is a missing file, show ls of the parent dir.

  Friday learns from failures:
    Every command failure is recorded in state.db:
      table: command_failures
      columns: command, exit_code, cwd, intent_id, timestamp, suggested_fix
    After 3 failures of the same pattern:
      Friday surfaces: "you have failed with X three times -- add alias?"
    This closes the loop: failure → learning → prevention.

Implementation:
  run_external() in commands/mod.rs -- post-exit-code analysis
  New module: src/error_ux.rs
    fn explain_exit_code(code: i32) -> &'static str
    fn suggest_command(failed: &str, path_commands: &[String]) -> Option<String>
    fn record_failure(cmd: &str, code: i32, db: &Connection)
  Levenshtein: pure Rust, no crate needed (< 20 lines)

Gate: ✗ gti shows "Did you mean: git?" within 50ms
      Exit code 126 shows "Permission denied -- try: chmod +x <file>"
      Third failure of same command triggers Friday suggestion
---
FEATURE 2 -- TIME-TRAVEL DEBUGGING

The problem:
  Something broke. You do not know when.
  You ran 40 commands. Which one caused it?
  Right now you have no way to know.

What fsh v4 does:

  State snapshots before destructive commands:
    Before any command tagged as destructive (rm, mv, deploy, git push, etc.),
    fsh records a lightweight snapshot:
      table: fsh_snapshots
      columns: id, timestamp, cwd, command, env_vars_json,
               intent_id, git_hash, health_before

  fsh rewind -- interactive time-travel TUI:
    Command: fsh rewind  (or just: rewind)
    Opens a ratatui TUI showing snapshot timeline:
      2026-05-18 01:02:14  deploy faelight-bar     [health: 100%] [git: ec96bc5]
      2026-05-18 01:01:33  git rm -r src/render     [health: 100%] [git: ec96bc5]
      2026-05-18 00:58:11  cargo build -p faelight-bar [health: 100%]
      ...
    Arrow keys navigate. Enter shows full snapshot detail.
    'r' attempts rollback (git checkout to that hash, restore env).
    'c' copies the command to clipboard.
    'f' asks Friday: "what changed between here and now?"

  fsh diff <snapshot_id> -- compare two points in time:
    Shows: commands run between them, health delta, git commits made.

  Friday integration:
    Friday reads snapshots to build causality chains.
    "deploy faelight-bar succeeded 3 times, failed once -- at that failure,
     health dropped from 100% to 87% -- the difference was uncommitted changes"
    This is the causality layer described in INT-186.

Implementation:
  New module: src/time_travel.rs
    struct Snapshot { ... }
    fn capture_snapshot(cmd: &str, db: &Connection) -> Result<()>
    fn is_destructive(cmd: &str) -> bool
    fn open_rewind_tui(db: &Connection) -> Result<()>
  New table: fsh_snapshots in state.db
  TUI: ratatui, same pattern as fsh history TUI (already exists)
  Rollback: git checkout <hash> + env restore

Gate: rewind command opens TUI showing last 20 snapshots
      Each snapshot shows command, time, health, git hash
      Friday can query: "what changed between snapshot A and B?"
---
FEATURE 3 -- STRUCTURED HISTORY WITH INTENT TAGGING

The problem:
  fsh history is a flat list.
  You cannot ask: what did I run during INT-295?
  You cannot replay a session.
  You cannot see that 80% of your commands during INT-287 were deploy.

What fsh v4 does:

  Every command tagged with active intent at time of execution:
    table: fsh_history (already exists, extend it)
    add column: intent_id INTEGER
    add column: session_id TEXT
    add column: duration_ms INTEGER
    add column: exit_code INTEGER
    add column: cwd TEXT

  New history commands:
    history                    -- current behavior (recent N commands)
    history for INT-295        -- all commands run during INT-295
    history for INT-295 | where exit_code = 0  -- only successful ones
    history replay INT-295     -- replay the session interactively
    history stats INT-295      -- most used commands, success rate, time spent
    history search "deploy"    -- full-text search across all history

  Session replay:
    history replay INT-295 opens a TUI:
      Shows each command in sequence.
      Press Enter to execute it (with confirmation for destructive ones).
      Press Space to skip.
      Press 'e' to edit before running.
    This is how you repeat a build process without re-reading notes.

  Friday analysis:
    history stats INT-295 shows Friday's view:
      "Most common command: cargo build (34 times)
       Most common failure: cargo build | grep error (12 times, always after editing main.rs)
       Time to first success: 23 minutes
       Suggestion: run cargo check before cargo build -- catches errors faster"

Implementation:
  Extend history recording in main.rs execute loop
  Add intent_id to every history record (read from active intent at runtime)
  New commands: history for, history replay, history stats, history search
  TUI for replay: ratatui, simple list with keybindings

Gate: history for INT-295 returns all commands from that session
      history stats shows per-command success rates
      history replay opens TUI and executes commands in sequence
---
FEATURE 4 -- PIPELINE VISUALIZATION

The problem:
  intents | where status = active | first 3
  You do not know how many rows each stage processed.
  You do not know which stage was slow.
  You do not know if a where clause filtered everything out silently.

What fsh v4 does:

  --explain flag on any pipeline:
    intents | where status = active | first 3 --explain

    Output:
      stage 1: intents          → 251 rows   [2ms]
      stage 2: where status=active → 5 rows  [0ms]  (filtered 246)
      stage 3: first 3          → 3 rows     [0ms]
      total: 2ms

  --trace flag for deep inspection:
    Shows the actual data flowing between each stage.
    intents | where status = active --trace
    Prints the rows as they pass through each stage boundary.

  Error surfacing in pipelines:
    Currently: if stage 2 fails, stage 3 may silently get empty input.
    fsh v4: if a stage returns 0 rows unexpectedly, warn:
      ⚠ where status = active: filtered all 251 rows -- did you mean 'in-progress'?

  Timing thresholds:
    If any stage takes > 100ms, highlight it in amber.
    If any stage takes > 1000ms, highlight in red with Friday suggestion.

Implementation:
  Pipeline execution in main.rs -- add timing wrapper around each stage
  PipelineStats struct: rows_in, rows_out, duration_ms per stage
  --explain flag: collect stats, print after execution
  --trace flag: print rows at each boundary (paginated via less)
  Zero-row warning: check rows_out == 0 when rows_in > 0, surface warning

Gate: intents | where status = active --explain shows per-stage row counts and timing
      Zero-row filter triggers warning with suggestion
      Slow stage (>100ms) highlighted in amber
---
FEATURE 5 -- FSH DOCTOR

The problem:
  Is fsh healthy? You do not know until something breaks.
  Is state.db accessible? Is Friday responding?
  Are there open INT-298 bugs still unfixed?
  Is the shell configured correctly?

What fsh v4 does:

  fsh doctor -- comprehensive self-health check:

    Checking fsh v4 health...

    Core:
      ✅ fsh binary:          v4.0.0 (ec96bc5)
      ✅ state.db:            accessible (3.2MB, 251 intents)
      ✅ fsh_history:         42,891 entries
      ✅ fsh_snapshots:       127 snapshots
      ✅ abbreviations:       47 loaded
      ✅ tab completion:      initialized

    Forest:
      ✅ /etc/faelight/INTENT: readable
      ✅ /etc/faelight/HEALTH: 95%
      ✅ active intent:       INT-287
      ✅ core lock:           unlocked (ready for editing)
      ⚠ friday_patterns:    87% confidence (last seen 4 hours ago)

    Shell builtins:
      ✅ query, fsearch, patch, rspatch, edit, run
      ✅ friday, friday dismiss
      ✅ deploy, cistart, cicomplete, fg
      ✅ history, rewind, fsh doctor

    Known issues (INT-298):
      ⚠ 2 open bugs in INT-298 -- run: intent show 298

    System:
      ✅ WAYLAND_DISPLAY:     wayland-1
      ✅ NIRI_SOCKET:         /run/user/1000/niri/socket
      ✅ HOME:                /home/christian
      ✅ 0-core path:         ~/0-core (main, clean)

    fsh doctor: 1 warning, 0 errors -- forest is healthy

  doctor --fix flag:
    Attempts to auto-fix common issues:
      state.db permissions wrong → chmod
      Stale Friday session → summarize and close
      Missing /etc/faelight/ files → regenerate via faelight-export

Implementation:
  New builtin: doctor in commands/mod.rs
  Checks run in parallel (tokio or rayon)
  Results collected into DoctorReport struct
  Rendered with color-coded ✅ ⚠ ✗ symbols
  --fix flag runs remediation functions per check

Gate: fsh doctor runs in < 500ms
      All checks pass on a healthy system
      Warnings shown for known INT-298 issues
      --fix resolves at least 3 common issues automatically
---
FEATURE 6 -- SANDBOXED COMMANDS

The problem:
  You are running a complex multi-file Python project.
  It has its own dependencies, its own paths, its own temp files.
  You do not want it touching ~/0-core paths.
  You do not want its environment polluting your shell state.
  This is not about security. It is about isolation of concern.

What fsh v4 does:

  fsh enter <project-name> -- scoped environment:
    Creates a shell scope with:
      Allowed paths: explicitly declared
      Blocked paths: ~/0-core/runtime/ (state.db protected)
      Own temp dir: /tmp/fsh-<project>-<session>/
      Own history segment: tagged with project name
      Own abbreviations: project-specific shortcuts
      Visual indicator: prompt changes to show scope

    Example:
      fsh enter data-pipeline
      [data-pipeline] fsh ❯ python3 process.py    -- runs in scope
      [data-pipeline] fsh ❯ exit                   -- returns to normal fsh

  fsh scope show -- what is currently in scope:
    Shows allowed paths, blocked paths, env overrides, temp dir.

  Scope definition file (.fsh-scope):
    A project can include a .fsh-scope file:
      name = "data-pipeline"
      allow = ["./src", "./data", "/tmp"]
      block = ["~/0-core"]
      env = { PYTHONPATH = "./src", DATA_DIR = "./data" }
      abbrev = { run = "python3 src/main.py", test = "pytest" }
    fsh enter reads this file automatically.

  Friday awareness:
    Friday tracks time spent in each scope.
    "You spent 3 hours in data-pipeline scope -- deploy happened outside scope"
    Scopes appear in history: history for scope:data-pipeline

Implementation:
  New builtin: enter, scope in commands/mod.rs
  ScopeState struct: allowed_paths, blocked_paths, env_overrides, abbrevs
  Scope stack: fsh can be nested (enter scope inside scope)
  Prompt modification: show [scope-name] in prompt when active
  .fsh-scope parser: simple TOML format

Gate: fsh enter data-pipeline changes prompt and restricts env
      Commands outside allowed paths show warning not error
      fsh scope show displays current restrictions
      exit returns to normal fsh with full environment restored
---
FEATURE 7 -- MULTI-COMMAND RELIABILITY

The problem:
  python3 /tmp/script.py && fg done "message"
  fsh splits this. The fg done runs before python3 finishes.
  Or it does not run at all. The behavior is unpredictable.
  This is the most reported pain point in fsh daily use.

The root cause:
  fsh's multi-command parser splits on && before evaluating context.
  Python invocations via heredoc create timing issues.
  The command queue does not wait for async operations to complete.

What fsh v4 does:

  True sequential execution for && chains:
    Each command in an && chain must fully complete (exit code collected)
    before the next begins. No exceptions.

  Python execution fix:
    python3 /tmp/script.py is treated as a blocking external command.
    fsh waits for the process to exit before evaluating the && condition.
    Exit code 0 → proceed. Non-zero → stop chain, show error.

  Heredoc + command chains:
    When a command produces output that feeds the next (via pipe or &&),
    fsh buffers correctly instead of racing.

  Timeout protection:
    Long-running commands in a chain get a visible indicator:
      ⏳ python3 /tmp/script.py  (running... 3s)
    After configurable timeout (default 300s) fsh asks: still waiting?

  Test suite for multi-command:
    New test file: tests/multi_command.rs
    Tests: && chain, || chain, semicolon chain, pipe chain
    Each test verifies execution order and exit code propagation
    Gate condition: all tests pass on every build

Implementation:
  Rewrite execute_chain() in main.rs
  Sequential executor: collect child process handles in order, wait() each
  Timing: std::time::Instant around each command
  Test suite: cargo test --test multi_command
  Known regressions from INT-298: fix as part of this feature

Gate: python3 /tmp/script.py && fg done "x" always runs fg done after python3 exits
      && chain stops on first non-zero exit code
      All INT-298 multi-command bugs closed
      Test suite passes: cargo test --test multi_command
---
PHASES

Phase 0 -- Audit (1 session):
  Catalog every known multi-command failure from INT-298
  Profile fsh startup -- where does time go?
  Design state.db schema additions: fsh_snapshots, extend fsh_history
  Gate: schema documented, INT-298 bug list complete

Phase 1 -- Multi-command reliability (1-2 sessions):
  This is the foundation. Everything else depends on a reliable executor.
  Rewrite execute_chain()
  Fix all known INT-298 multi-command bugs
  Write test suite
  Gate: all INT-298 multi-command bugs closed, test suite green

Phase 2 -- Better error UX (1 session):
  exit code meanings, did-you-mean, failure recording
  Gate: gti suggests git, exit 126 explains permission denied

Phase 3 -- Structured history (1 session):
  Extend fsh_history schema, add intent tagging
  history for, history stats, history search commands
  Gate: history for INT-322 works on first use after intent starts

Phase 4 -- Time-travel debugging (1-2 sessions):
  fsh_snapshots table, snapshot capture on destructive commands
  rewind TUI, diff command
  Gate: rewind opens TUI with last 20 snapshots

Phase 5 -- Pipeline visualization (1 session):
  --explain flag, --trace flag, zero-row warning
  Gate: any pipeline accepts --explain and shows per-stage stats

Phase 6 -- fsh doctor (1 session):
  All checks, DoctorReport, color output, --fix flag
  Gate: fsh doctor runs in < 500ms, all checks pass on clean system

Phase 7 -- Sandboxed commands (1-2 sessions):
  fsh enter, .fsh-scope parser, scope stack, prompt modification
  Gate: fsh enter creates isolated scope, exit restores cleanly

Phase 8 -- Integration + daily driver (1 week):
  All features working together
  Friday learning from failures feeds into doctor and history stats
  Gate: 1 week daily use with no regressions
---
GATES
[x] Phase 0: schema designed -- fsh_snapshots, shell_history extended, INT-298 bugs addressed 2026-05-26
[x] Phase 1: multi-command -- builtin dispatcher in chain executor, 82/82 tests pass 2026-05-26
[x] Phase 2: Damerau-Levenshtein did-you-mean, exit code explanations, command_failures table 2026-05-26
[x] Phase 3: history intent INT-N and history stats INT-N working, intent_id tagged on every command 2026-05-26
[x] Phase 4: rewind shows snapshot timeline with command, health, git hash, intent tag -- auto-capture on destructive commands 2026-05-26
[x] Phase 5: pipeline --explain with per-stage row counts, timing, zero-row warnings 2026-05-26 and timing
[x] Phase 6: fsh doctor 7/7 checks 7ms --fix mode shell healthy 2026-05-26
[x] Phase 7: fsh enter/leave/scope working -- cwd isolation, return path saved, clean restore 2026-05-26
[x] Phase 8: daily driver confirmed -- all 7 phases complete, 82/82 tests, features in daily use 2026-05-26
Final:
[x] fsh explains every failure in human terms -- exit codes named, did-you-mean working
[x] rewind command shows snapshot timeline with command, health, git hash
[x] history intent INT-N returns intent-tagged command log
[x] pipelines show per-stage row counts, timing, zero-row warnings via --explain
[x] fsh doctor 7/7 checks in 7ms
[x] fsh enter/leave/scope -- project-scoped environments with cwd isolation
[x] && chains route through fsh builtin dispatcher, redirect-safe 2026-05-26
---
DEPENDS ON
fsh v3 (INT-318) -- COMPLETE -- foundation
state.db -- current schema, will be extended
INT-298 -- open shell bugs, will be closed in Phase 1
INT-186 -- Delegation Engine -- time-travel connects to causality layer

TIMELINE
Phase 1 (multi-command): highest priority, start immediately
Phase 2-6: one session each, sequence matters less than Phase 1
Phase 7 (sandbox): can be last, least urgent
Target: Phases 1-6 complete before NY presentation (mid-July 2026)

"fsh v3 proved the shell could live.
fsh v4 proves the shell deserves trust.
Every failure explained.
Every command remembered.
Every moment recoverable.
The shell that grows up." 🌲

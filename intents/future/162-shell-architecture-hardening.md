---
id: 162
date: 2026-03-28
type: future
title: "Shell Architecture Hardening — The Foundation Must Be Solid"
status: planned
tags: [shell, architecture, schemas, pipelines, layers, fsh, v12, integrity]
version: 12.0.0
priority: critical
depends_on: [146]
---

## Why This Exists
Outside critique confirmed what the data shows:
faelight-shell is trying to be too many things at once.
Before v12 builds on top of fsh, the foundation must be solid.

This is not about adding features.
This is about making what exists correct, reliable, and honest.

## The Three Hard Decisions

### Decision 1 — Path A: Personal Super-Shell
faelight-shell is NOT a general-purpose shell.
It is a personal computing interface for one human and one forest.

This means:
- Tight forest coupling is a FEATURE not a bug
- Portability is NOT a goal
- Competing with Nushell is NOT a goal
- Being the best shell FOR THIS SYSTEM is the only goal

Everything below serves Path A.

### Decision 2 — Hard Layer Separation
Three layers. Hard boundaries. Never blurred.
```
Layer 1 — Shell Engine (fsh owns this)
  parsing, tokenization, history, completion
  job control, signal handling, redirection
  pipeline execution, external command passthrough
  NO business logic. NO forest state reads.

Layer 2 — Data Model (shared contract)
  structured tables with defined schemas
  type-safe pipeline operators
  explicit serialization at external boundaries
  NO side effects. NO system calls.

Layer 3 — System Integration (core owns this)
  health, intents, predictions, reactions
  all forest state reads and writes
  all policy decisions
  fsh CALLS core — core does NOT embed in fsh
```

DEC-005 is the law: fsh is interface only.
Policy, reactions, predictions live in core.
Every fsh command fires events into state.db — that is the causal link.

### Decision 3 — Formal Schemas
Every structured command defines its output schema.
No more implicit tables. No more silent type failures.

## Phase 0 — ExecContext: From String-Driven to Context-Driven (1 session)

This is the most important change in the entire intent.
Everything else builds on it.

### The Problem
Right now execution is string-driven:
```
string → split → match → execute
```
This means the shell has no memory of what it is executing,
no context to pass to hooks, no structure for policy enforcement.
preexec hooks (INT-171), structured errors (INT-174), and failure
recovery (INT-176) are impossible to build cleanly without this.

### The Solution
Create ExecContext — a typed description of every command execution:
```rust
pub struct ExecContext {
    pub raw:       String,         // exactly what the user typed
    pub expanded:  String,         // after alias expansion
    pub cmd:       String,         // resolved command name
    pub args:      Vec<String>,    // resolved arguments
    pub cwd:       PathBuf,        // current working directory
    pub intent:    Option<String>, // active intent (INT-NNN) if any
    pub timestamp: u64,            // when this was executed
}
```

### The Execution Pipeline
Replace recursive string dispatch with a clean lifecycle:
```rust
// engine/exec.rs — ONE place, ONE responsibility
pub fn execute(line: &str, db: &ForestDb) -> CommandResult {
    let mut ctx = parse_to_context(line);  // build ExecContext
    resolve_alias(&mut ctx, db);           // expand aliases
    preexec(&mut ctx, db);                 // before_run hooks (INT-171)
    let result = dispatch(&ctx, db);       // run the command
    postexec(&ctx, &result, db);           // logging, events, suggestions
    result
}
```

### commands/mod.rs Becomes Pure Dispatch
```rust
// Before: commands/mod.rs does everything
// After:  commands/mod.rs does ONE thing
pub fn dispatch(ctx: &ExecContext, db: &ForestDb) -> CommandResult {
    match ctx.cmd.as_str() {
        "gc" => gc(ctx, db),
        "d"  => doctor(ctx, db),
        // ...
    }
}
```

### What This Unlocks Immediately
```
INT-171 Pre-Command Decision Layer  → preexec hook exists
INT-174 Structured Errors           → ExecContext gives error context
INT-176 Failure Recovery            → ctx stored on failure
INT-177 Shell Observability         → postexec collects metrics
INT-173 Command Registry            → dispatch table becomes registry
```

### Acceptance Criteria
```
⬜ ExecContext struct defined in engine/exec.rs
⬜ execute() pipeline: parse → alias → preexec → dispatch → postexec
⬜ commands/mod.rs reduced to pure dispatch only
⬜ No recursive execute() calls anywhere
⬜ All existing commands pass ExecContext instead of raw strings
⬜ Build passes with zero regressions
```

## Phase 1 — Layer Audit (1 session)
Audit every function in commands/mod.rs and classify:
```
SHELL    — pure shell behavior (keep in fsh)
DATA     — table/pipeline operation (keep in fsh, add schema)
FOREST   — reads/writes forest state (move call to core)
POLICY   — makes decisions (must live in core only)
```

Deliverable: documented classification of all 4,507 lines.

## Phase 2 — Schema System (1-2 sessions)
Define schemas for every structured command output.

Priority commands needing schemas first:
```rust
// ps output schema
struct ProcessRow {
    pid:     u32,     // always present
    name:    String,  // always present
    cpu:     f32,     // 0.0-100.0, never null
    memory:  f32,     // MB, never null
    status:  String,  // running/sleeping/zombie
}

// gc output schema
struct CommitRow {
    hash:    String,  // 7-char short hash
    message: String,  // full message
    author:  String,  // author name
    date:    String,  // ISO date
    domain:  String,  // extracted from conventional commit
}

// health output schema
struct HealthRow {
    check:   String,  // check name
    status:  String,  // pass/warn/fail
    message: String,  // human message
}
```

Schema benefits:
- Tab completion knows column names
- `where` operator validates field names
- Pipeline errors are caught early, not silently
- `core predict coupling` can analyze real schemas

## Phase 3 — Pipeline Hardening (1-2 sessions)
Make pipelines first-class. Compete seriously with structure.

Current operators: first, last, where, sort, select, count
Missing operators that matter:
```
map <expr>     — transform each row
reduce <expr>  — aggregate to single value  
join <cmd>     — join two command outputs
group <field>  — group by field, aggregate
unique <field> — deduplicate by field
flatten        — expand nested tables
```

External boundary — EXPLICIT serialization:
```fsh
# Current (implicit, lossy):
gc | first 20 | grep feat

# Correct (explicit boundary):
gc | first 20 | to-text | grep feat

# Or stay structured:
gc | first 20 | where message contains "feat"
```

## Phase 4 — Grammar Formalization (1 session)
Write down the actual grammar rules fsh follows.
Not aspirational — what it actually does today.
```
command    := name arg*
pipeline   := command ("|" operator)*
operator   := "where" expr | "sort" field dir | "first" n | "last" n
             | "select" fields | "count" | "map" expr | "group" field
expr       := field op value
field      := identifier
op         := "==" | "!=" | ">" | "<" | ">=" | "<=" | "contains"
value      := string | number | bool
```

Then enforce it. No more "whatever feels right."

## Phase 5 — Forest Command Separation (1 session)
Forest commands move OUT of fsh dispatch into core calls.
```fsh
# Before (blurred):
health        # fsh builtin that reads state
intents       # fsh builtin that reads files
decisions     # fsh builtin that reads db

# After (clear boundary):
core health   # explicit core call
core intents  # explicit core call  
core decisions # explicit core call

# Short forms stay via core shortcuts (already built):
health → calls core health
intents → calls core intents
```

This does NOT mean removing convenience.
It means being honest about where the logic lives.

## Phase 6 — Scripting Story (1 session)
.fsh scripts must be deterministic and reliable.

Define and enforce:
```
let x = <value>           # variable assignment
if <condition> { }        # conditional
for <item> in <list> { }  # iteration
fn name(args) { }         # function definition
emit <event>              # fire forest event
on <event> { }            # event handler
```

Every script must be testable in isolation.
No hidden state. No ambient forest reads inside scripts.

## Acceptance Criteria
```
⬜ Phase 1 — every command in commands/mod.rs classified
⬜ Phase 2 — schemas defined for top 10 commands
⬜ Phase 2 — pipeline type errors caught at boundary
⬜ Phase 3 — map/reduce/group operators implemented
⬜ Phase 3 — explicit to-text serialization at external boundary
⬜ Phase 4 — grammar documented and enforced
⬜ Phase 5 — forest commands call core, not embed logic
⬜ Phase 6 — .fsh scripts deterministic and testable
⬜ DEC-005 fully implemented — zero policy logic in fsh
⬜ Zero silent type failures in pipelines
```

## Gate Check
```
⬜ Phase 0 — ExecContext implemented, execution pipeline clean
⬜ Phase 1 — Layer audit complete
⬜ Phase 2 — Schema system implemented
⬜ Phase 3 — Pipeline operators complete
⬜ Phase 4 — Grammar formalized
⬜ Phase 5 — Forest commands separated
⬜ Phase 6 — Scripting story complete
```

## The Phrase
**"A shell that knows what it is
does not try to be everything.
It does its job perfectly
and hands everything else
to the right layer."**

---
*"Structural integrity is not a feature.
It is the prerequisite for every feature that follows."* 🌲

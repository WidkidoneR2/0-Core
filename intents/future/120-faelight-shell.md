---
id: 120
date: 2026-03-12
type: future
title: "faelight-shell — Forest-Native Shell Environment"
status: in-progress
tags: [shell, repl, structured-data, security, plugins, scripting, rust, v11, v12]
version: 11.0.0
priority: high
---

## Vision

Not a POSIX shell. Not bash. Not Nu. Not NixOS.

**Beyond NixOS.**

NixOS knows WHAT a system is.
faelight-shell knows WHY it became that way,
HOW it is being used, WHAT worked,
and WHERE it is going.

This is not configuration management.
This is a living, self-aware computing environment
expressed through its own native language.

## The Core Philosophy
```
POSIX:          text | text | text
Nu:             table | filter | transform
faelight-shell: forest_data | judgment | wisdom
```

Everything is structured data.
Every command is forest-aware.
The prompt is a live system instrument, not a string.
Security is the foundation, not an afterthought.
The shell and the forest are one thing.

## The Interactive Prompt

The prompt is not separate from the shell.
The prompt IS the shell's voice.
```
🌲 forest ~/0-core [main ✓] 100% HEALTHY INT-120 ❯ _
         ↑            ↑        ↑       ↑       ↑
    location      git branch  health  status  active intent
```

### Prompt Interactions
- Tab          — fuzzy completion: commands + filesystem + history
- Ctrl+F       — search past commands with forest context
- Ctrl+I       — show active intents inline
- Ctrl+A       — open advise panel without leaving prompt
- Ctrl+H       — quick health summary inline
- Ctrl+D       — show recent decisions

The prompt reflects live system state on every render.
Not cached. Not static. Live.

### Prompt Configuration (prompt.toml)
```toml
[prompt]
show_health = true
show_intent = true
show_git = true
show_zone = true
show_risk = false       # show Core v6 risk score
color_theme = "faelight"
compact_mode = false
```

## Beyond NixOS — What This Achieves

| Capability | NixOS | faelight-shell |
|-----------|-------|----------------|
| Reproducible state | ✅ declarative config | ✅ intent ledger |
| WHY it became this way | ❌ | ✅ decision ledger |
| HOW it's being used | ❌ | ✅ event history |
| WHAT worked | ❌ | ✅ Core v6 judgment |
| System language | Nix (hostile) | .fsh (forest-native) |
| Usage patterns | ❌ | ✅ audit scores |
| Anticipates needs | ❌ | ✅ predictive layer |
| Speaks to you | ❌ | ✅ (long-term) |

NixOS describes a system state.
Faelight Forest understands a system's life.

## Structured Data — Everything is a Value
```
forest> events today | where domain == "git" | sort by timestamp
forest> tools | where score < 70 | select name score issues
forest> decisions | where outcome == "pending" | count
forest> intents | where status == "active" | first 5
forest> health | get checks | where status == "warn"
```

No text parsing. No grep. No awk.
Forest data is structured from the source.

## The Value System
```rust
enum Value {
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Date(DateTime),
    Row(HashMap<String, Value>),
    Table(Vec<Row>),
    Nothing,
}
```

Every command returns a Value.
Pipes pass Values between commands.
Output renders any Value as table, list, or summary.

## The .fsh Scripting Language

Not for automating tasks.
For expressing forest behavior and intent.
```fsh
# When health degrades — forest responds
when health < 90 {
    advise "system degraded — review doctor output"
    checkpoint "auto-pre-recovery"
}

# When intent completes — forest grows
when intent.complete {
    audit affected_tools
    emit "forest.grew"
}

# Deploy with safety
let tool = $args.0
let score = (audit show $tool | get score)

if $score < 70 {
    warn $"($tool) has low audit score: ($score)"
    confirm "Proceed anyway?"
}

run cargo build --release -p $tool
emit "tool.deployed" { name: $tool, score: $score }
```

## Security — Built In, Not Bolted On

Every command execution:
- Policy check before execution
- Logged to state.db as shell.command event
- External commands sandboxed via faelight-sandbox
- Environment variables read-only by default
- Filesystem writes require explicit permission
- sudo blocked at shell level
- All security events flow to core security advise

## Architecture
```
faelight-shell/src/
├── main.rs
├── repl/
│   ├── mod.rs          — REPL loop
│   ├── prompt.rs       — live context-aware prompt engine
│   ├── history.rs      — history persisted to state.db
│   └── keybinds.rs     — Ctrl+F, Ctrl+I, Ctrl+A, Ctrl+H
├── parser/
│   ├── lexer.rs        — token stream
│   ├── ast.rs          — expression tree
│   └── pipeline.rs     — pipe operator
├── engine/
│   ├── value.rs        — Value type system
│   ├── pipeline.rs     — execute pipeline stages
│   └── scope.rs        — variables and environment
├── commands/
│   ├── forest/         — health, events, decisions, intents...
│   ├── data/           — where, select, sort, count, first, last
│   └── system/         — run (sandboxed), cd, env (read-only)
├── security/
│   ├── policy.rs       — command security policies
│   ├── audit.rs        — execution audit log
│   └── sandbox.rs      — faelight-sandbox bridge
├── plugins/
│   ├── loader.rs       — .fsh plugin files
│   └── api.rs          — plugin API surface
├── completion/
│   ├── fuzzy.rs        — fuzzy matching
│   └── context.rs      — context-aware suggestions
├── scripting/
│   ├── variables.rs    — let x = ...
│   ├── control.rs      — if, for, when
│   └── events.rs       — forest event hooks
└── output/
    ├── table.rs        — ratatui table rendering
    ├── color.rs        — Faelight Forest palette
    └── format.rs       — Value display formatting
```

## Dependencies
```toml
rustyline   — readline input, history, keybinds
ratatui     — table rendering, TUI panels
crossterm   — terminal control, raw mode
rusqlite    — state.db direct access
colored     — Faelight Forest color palette
serde_json  — structured output
chrono      — date/time in Values
```

## Build Phases

### Phase 1 — REPL Skeleton (1-2 sessions)
- rustyline input with live prompt
- Ctrl+I, Ctrl+A, Ctrl+H keybinds
- 10 forest commands working
- state.db connected directly
- History persisted to state.db
- Deployed as fs alias

### Phase 2 — Data Pipeline (2-3 sessions)
- Value type system
- where, select, sort, count, first, last
- Pipe operator
- events today | where domain == "git"

### Phase 3 — Security Layer (1 session)
- Every command logged to state.db
- External command sandbox integration
- Policy declarations
- Blocked commands enforcement

### Phase 4 — Completion & Fuzzy (1-2 sessions)
- Tab completion for forest commands
- Fuzzy history search
- Context-aware suggestions

### Phase 5 — Plugin System (2-3 sessions)
- .fsh plugin loader
- Plugin API surface
- First forest plugin

### Phase 6 — Scripting Language (3+ sessions)
- Variables, conditions, loops
- when/on event hooks
- .fsh script execution

### Phase 7 — Full Shell & Beyond NixOS (v12+)
- Replace zsh for forest workflows
- Complete external command support
- Built-in package helpers
- Prompt fully interactive
- Predictive command suggestions from history
- Voice/natural language foundation
- The system that knows itself completely


## Extended Vision — Phases 8-32

These phases represent the long-term evolution of faelight-shell
toward a full structured shell operating system interface.

### Phase 8 — System Tables (HIGH PRIORITY)
Expose OS state as structured tables — the most impactful next step.
```
processes | where cpu > 20 | sort cpu
ports | where port == 8080
services | where status == failed
files | where size > 1gb | sort size
network | where rx > 1mb
packages | where outdated == true
```
This makes faelight-shell immediately useful as a daily driver.
Real OS data flowing through the existing pipeline engine.

### Daily Driver Target — Phase 18
By Phase 18 faelight-shell should be the primary forest interface.
Not replacing zsh entirely — but first tool opened for forest work.

### Phase 9 — Streaming Pipelines
Live pipelines for observability.
```
logs --follow | where level == error
network | watch | histogram traffic
processes | watch
```

### Phase 10 — Terminal Visualization (partial ✅)
histogram already exists. Expand with:
```
processes | chart cpu
dashboard system
```

### Phase 11 — Schema-Aware Autocomplete
Tab completion understands column names.
```
files | where <TAB>  →  name, size, modified, owner
processes | where <TAB>  →  pid, name, cpu, memory
```

### Phase 12 — Package Manager
```
fsh install docker
fsh install aws
```

### Phase 14 — File System Index
Persistent index for fast file queries.
```
files | where size > 1gb
files | group extension
```
Much faster than find.

### Phase 15 — Git Data Engine (partial ✅)
gc/gf already exist. Expand to:
```
git.commits | where author == christian
git.files | top churn
git.branches | where merged == false
```

### Phase 16 — History Analytics (partial ✅)
ht exists. Expand to:
```
history | histogram command
history | where duration > 5s
```

### Phase 17 — Event System (Reactive Shell)
```
on file_change run build
on log_error notify
```

### Phase 18 — Time Travel
```
snapshot
timeline processes
diff snapshot1 snapshot2
```

### Phase 21 — Query Language
SQL-like syntax:
```
select name, size from files where size > 1gb
```

### Phase 22 — Observability Dashboard
```
dashboard        # full system overview
metrics          # live metrics
dashboard system # CPU, memory, network, top processes, errors
```

### Phase 25 — AI Command Assistant
Natural language to shell commands:
```
find biggest files       → files | sort size desc | first 10
show memory hogs         → processes | sort memory desc | first 5
why is my computer slow  → auto-diagnose CPU/memory/disk/network
```

### Phase 32 — Shell as OS Layer
At this point faelight-shell becomes a meta-OS interface.
Everything accessible through pipelines:
processes, files, containers, network, cloud, metrics

The "Secret Sauce" pipeline:
```
processes
| where cpu > 50
| join ports on pid
| table
```

## The Four-Layer Architecture

This is the organizing principle for all remaining phases.
Each layer must be solid before the next is built.

### Layer 1 - REALITY (Ground Truth)
What the shell knows about the world.
- Phase 8  DONE: System tables (ps, ports, services, files, net, pkgs)
- Phase 14: File system index DONE (2026-03-21)
- Phase 15: Git data engine DONE (2026-03-21)

### Layer 2 - UNDERSTANDING (Query + Schema)
How users think with the system.
- Phase 11a: Formal schema system DONE (2026-03-20)
- Phase 11:  Schema-aware autocomplete DONE (2026-03-20)
- Phase 11:  Schema-aware autocomplete (needs 11a first)
- Phase 2   DONE: Data pipelines
- Phase 21: Query language (adoption bridge, not core)

### Layer 3 - REACTION (Events + Time)
Where the shell becomes alive.
Only build after Layer 2 is solid.
- Phase 16: History analytics
- Phase 17: Event system (on solid schema foundation)
- Phase 18: Time travel
- Phase 9  DONE: Streaming pipelines

### Layer 4 - INTELLIGENCE (Judgment + AI)
Where the shell becomes opinionated.
- Phase 22: Observability dashboard
- Phase 25: Natural language assistant (INT-139)
- Phase 32: Shell as OS layer

THE RULE: Do not build Reaction before Understanding is solid.
Event triggers on brittle schema = debugging nightmare.
AI on unreliable data = a crutch, not wisdom.

## Phase 11a - Formal Schema System (NEXT PRIORITY)

The most important unbuilt foundation. Without it:
- Autocomplete is hardcoded strings, not real schema
- Joins are fragile and break silently
- Query language has no type safety
- AI assistant has no ground truth to reason from

Every system table needs a registered schema in the shell:
- ps/processes: pid(Int), name(Text), cpu(Float), memory(Float), user(Text), status(Text)
- files:        name(Text), kind(Text), size(Int), modified(Timestamp)
- services:     name(Text), active(Text), load(Text), status(Text)
- ports:        port(Int), state(Text), address(Text), process(Text)
- tt:           name(Text), version(Text), score(Int), deployed(Bool)
- et:           domain(Text), action(Text), timestamp(Int), time(Text)
- gc:           hash(Text), author(Text), date(Text), message(Text)

This schema registry makes joins reliable and type-checked at parse time.

## The Join System - SHIPPED (2026-03-20)

The most powerful thing built so far.

    processes | where cpu > 50 | join ports on pid | join logs on pid | table

This is not a shell feature.
This is ad-hoc relational joins over live system state.
That is an observability platform, not a shell.

## Corrected Strategic Priority Order

DONE:
- Phase 8  - System tables
- Phase 9  - Streaming pipelines
- Phase 10 - Shell personality and living welcome

NEXT (in order):
- Phase 11a - Formal schema system DONE (2026-03-20)
- Phase 11  - Schema-aware autocomplete
- Phase 14  - File system index DONE (2026-03-21)
- Phase 15  - Git data engine DONE (2026-03-21)
- Phase 16  - History analytics
- Phase 17  - Event system DONE (2026-03-21)
- Phase 18  - Time travel DONE (2026-03-21)
- Phase 21  - Query language (adoption bridge)
- Phase 22  - Observability dashboard
- Phase 25  - Natural language assistant (INT-139, amplifier not crutch)
- Phase 32  - Shell as OS layer

The single highest-leverage move:
Build the schema system first. It unlocks everything.


## The Three Pillars (from architectural review)
```
Structured data pipelines      ✅ built
Real-time system observability  ⬜ Phase 9/22
Queryable OS state              ✅ Phase 8 + schema (11a) DONE
```

## Gate Check

- ✅ Phase 1: REPL with live prompt and 10+ commands
- ✅ Phase 1: state.db connected, history persisted
- ✅ Phase 2: Value pipeline — filter, sort, select
- ✅ Phase 3: Security audit log
- ✅ Phase 4: Fuzzy tab completion (2026-03-20)
- ✅ Phase 4: Alias system — persistent named commands
  - alias h=health
  - alias eg="events today | where domain == git"
  - alias mycommits="gc | where author == christian"
  - Stored in shell_aliases table in state.db
  - Loaded on startup, applied before dispatch
  - alias / unalias commands
  - Tab completion suggests aliases too
- ✅ Phase 5: Plugin system — .fsh TOML plugins, forest-utils shipped, plugins/plr commands
- ⬜ Phase 6: .fsh scripting language
- ⬜ Phase 7: Full shell — beyond NixOS

## Development Philosophy

This is a multi-year craft project — 16,000 to 45,000 lines of Rust.
It is built alongside INT-109 and the rest of the forest, not instead of them.
```
When DRM gets hard     — write some shell code
When shell gets complex — work on a compositor session
When both need a break — build something small and fun
```

No deadlines. No rushing. Each phase complete before the next.
The forest grows at its own pace.

---
*"A forest deserves a shell that knows it is a forest."* 🌲
*"Not text streams. Not configuration. Structured wisdom."* 🌲
*"NixOS knows what. faelight-shell knows why."* 🌲
*"When DRM gets hard — write some shell code."* 🌲

## Dependencies and Next Steps

### Before faelight-vault (INT-132)
- INT-109 faelight-compositor DRM must be completed first
- faelight-vault builds on faelight-gen (INT-130 complete)
- faelight-vault is the credential layer for a self-contained forest

### Phase 9+ Priority Order
- Phase 9  — Streaming Pipelines
- Phase 10 — Terminal Visualization
- Phase 11 — Schema-aware autocomplete
- Phase 17 — Event triggers
- Phase 18 — Daily driver milestone then evaluate faelight-vault

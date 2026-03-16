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

## Gate Check

- ⬜ Phase 1: REPL with live prompt and 10+ commands
- ⬜ Phase 1: state.db connected, history persisted
- ⬜ Phase 2: Value pipeline — filter, sort, select
- ⬜ Phase 3: Security audit log
- ⬜ Phase 4: Fuzzy tab completion
- ⬜ Phase 4: Alias system — persistent named commands
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

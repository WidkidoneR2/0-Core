---
id: 120
date: 2026-03-12
type: future
title: "faelight-shell — Forest-Native Structured Shell Environment"
status: planned
tags: [shell, repl, structured-data, security, plugins, scripting, rust, v11]
version: 11.0.0
priority: high
---

## Vision

Not a POSIX shell rebuilt in Rust.
Not bash. Not zsh. Not a query wrapper.

A forest-native structured shell environment inspired by Nu shell —
where everything is structured data, every command is forest-aware,
and security is built into the foundation, not bolted on top.

**The core philosophy:**
```
POSIX thinks in:        text | text | text
Nu thinks in:           table | filter | transform  
faelight-shell thinks:  forest_data | filter | render
```

## What Makes It Different

### Structured Data — Everything is a Value
```
forest> events today | where domain == "git" | sort by timestamp
forest> tools | where score < 70 | select name score issues
forest> decisions | where outcome == "pending" | count
forest> intents | where status == "active" | first 5
```

No text parsing. No grep. No awk.
Forest data is structured from the source.

### Security Built In — Not Bolted On
Every command execution:
- Checked against security policy before running
- Logged to state.db as shell.command event
- External commands run through faelight-sandbox
- Environment variables read-only by default
- Filesystem writes require explicit permission
- sudo blocked entirely at shell level

### Forest-Native Commands
```
health          events          decisions
intents         tools           audit
story           advise          simulate
version         commits         checkpoint
```

All return structured Values, pipeable, filterable.

### Context-Aware Prompt
```
🌲 forest ~/0-core [main] 100% ❯
```
Location, git branch, health %, forest zone.
Configurable via prompt.toml.

## Architecture
```
faelight-shell/src/
├── main.rs
├── repl/          — REPL loop, prompt, history in state.db
├── parser/        — lexer, AST, pipeline operator
├── engine/        — evaluation, Value types, scope
├── commands/
│   ├── forest/    — events, health, decisions, intents...
│   ├── data/      — where, select, sort, count, first, last
│   └── system/    — run (sandboxed), cd, env (read-only)
├── security/      — policy engine, audit log, sandbox bridge
├── plugins/       — .fsh plugin loader and API
├── completion/    — fuzzy tab completion
├── scripting/     — .fsh language: variables, conditions, loops
└── output/        — ratatui tables, Faelight color palette
```

## The Value System
```rust
enum Value {
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Row(HashMap<String, Value>),    // single record
    Table(Vec<Row>),                // list of records
    Nothing,
}
```

Every command returns a Value.
Pipes pass Values between commands.
Output renderer displays any Value as table, list, or summary.

## The Scripting Language (.fsh)
```
# deploy-tool.fsh
let tool = $args.0
let score = (audit show $tool | get score)

if $score < 70 {
    warn $"Tool ($tool) score is low: ($score)"
    confirm "Proceed anyway?"
}

run cargo build --release -p $tool
emit "tool.deployed" { name: $tool, score: $score }
```

Forest-aware. Variables, conditions, event emission.
Not bash. Not Python. Forest language.

## Dependencies

- rustyline    — readline input, history, Ctrl+C
- ratatui      — table rendering, TUI output
- crossterm    — terminal control
- rusqlite     — state.db direct access
- colored      — Faelight Forest palette
- serde_json   — structured output

## Build Phases

### Phase 1 — REPL Skeleton (1-2 sessions)
- rustyline input loop
- Context-aware prompt
- 10 forest commands (health, events, decisions...)
- state.db connected directly
- help, exit, version
- Deployed as fs alias
- History persisted to state.db

### Phase 2 — Data Pipeline (2-3 sessions)
- Value type system
- where, select, sort, count, first, last
- Pipe operator between commands
- events today | where domain == "git"

### Phase 3 — Security Layer (1 session)
- Every command logged to state.db
- External command sandbox integration
- Policy declarations
- Blocked commands list

### Phase 4 — Completion & Fuzzy (1-2 sessions)
- Tab completion for forest commands
- Fuzzy search across history
- Context-aware suggestions

### Phase 5 — Plugin System (2-3 sessions)
- .fsh plugin file loader
- Plugin API surface
- First forest plugin

### Phase 6 — Scripting Language (3+ sessions)
- Variables (let x = ...)
- Conditions (if/else)
- Loops (for item in list)
- .fsh script execution
- Forest event hooks

### Phase 7 — Full Shell (long-term, v12+)
- Replace zsh for forest workflows
- Complete external command support
- Built-in package helpers
- Voice/natural language layer foundation

## Success Criteria

- [ ] Phase 1: REPL running with 10+ forest commands
- [ ] Phase 1: state.db connected, history persisted
- [ ] Phase 2: Value pipeline — filter, sort, select
- [ ] Phase 3: Security audit log for every command
- [ ] Phase 4: Fuzzy tab completion
- [ ] Phase 5: Plugin system with .fsh files
- [ ] Phase 6: Basic scripting language
- [ ] Phase 7: Full shell replacement (long-term)

## The Prompt Vision
```
🌲 forest ~/0-core [main ✓] 100% HEALTHY ❯
🌲 forest ~/0-core [main !3] 95% ADVISORY ❯
```

The shell knows the forest state at all times.

---
*"A forest deserves a shell that knows it is a forest."* 🌲
*"Not text streams. Structured wisdom."* 🌲

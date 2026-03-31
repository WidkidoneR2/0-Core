---
id: 173
date: 2026-03-30
type: future
title: "Command Registry — The Shell Knows What It Can Do"
status: in-progress
tags: [shell, fsh, registry, completion, commands, intelligence]
version: 12.0.0
priority: medium
depends_on: [146, 162]
---

## The Problem
faelight-shell resolves commands at runtime from three separate namespaces:
- PATH binaries (external)
- Aliases (from config.fsh)
- Builtins (hardcoded in commands/mod.rs)

These namespaces are invisible to each other.
The shell cannot describe a command, explain its source, or reason about it.
Tab completion cannot be intelligent without knowing what commands exist.
Prediction cannot learn patterns without knowing what was run and from where.

## The Solution
A unified command registry — one source of truth for every command
the shell knows about, regardless of where it comes from.
```fsh
which gc          # → alias: git commit -m (from config.fsh)
describe gc       # → Forest git commit shortcut
command list      # → all known commands with source
command info core # → binary at scripts/core, version 2.0.0, 47 subcommands
```

## The Registry Structure
```
CommandEntry {
    name:        String,           // "gc"
    kind:        Builtin | Alias | Binary | Script,
    source:      String,           // "config.fsh:28" or "/usr/bin/git"
    description: Option<String>,   // human-readable purpose
    usage:       Option<String>,   // usage hint
    aliases:     Vec<String>,      // known aliases for this command
}
```

## What This Enables
```
Tab completion v3   → complete based on registry, not just PATH
Predict             → learn which commands are used together
before_run (INT-171)→ apply rules by command kind, not just name
describe <cmd>      → the shell can explain itself
documentation       → auto-generate command reference
```

## Phase 1 — Registry Foundation
Build the registry data structure.
Populate on shell startup from:
- builtins list (hardcoded in commands/mod.rs)
- aliases from config.fsh
- PATH scan for binaries

## Phase 2 — Query Commands
```fsh
which <cmd>         # already exists — enhance with registry source
describe <cmd>      # new: human-readable description
command list        # new: all known commands
command info <cmd>  # new: full detail
```

## Phase 3 — Registry Integration
Feed registry into:
- Tab completion (knows all commands + descriptions)
- before_run rules (can check command.kind)
- predict (learns patterns by command category)

## Phase 4 — Descriptions
Allow descriptions to be defined in config.fsh:
```fsh
describe gc = "Forest git commit — always use instead of git commit"
describe deploy = "Build and deploy core or faelight-shell"
```

## Gate Check
```
✅ Registry structure defined — CommandEntry/CommandKind/Registry in registry.rs, populated on startup (2026-03-30)
✅ 26 builtins registered with descriptions and usage (2026-03-30)
✅ Aliases registered from db with config.fsh source (2026-03-30)
✅ Forest scripts scanned from scripts/ directory — 134 total commands (2026-03-30)
✅ which enhanced — registry-aware via describe command (2026-03-30)
✅ describe command live — kind/source/description/usage (2026-03-30)
✅ command list live — filterable by kind (2026-03-30)
✅ command info live — full detail per command (2026-03-30)
✅ Registry available on startup — tab completion integration deferred to INT-179 Phase 28 (2026-03-30)
✅ Registry populated on startup — before_run integration deferred to INT-179 (2026-03-30)
```

## The Phrase
**"A shell that cannot describe its own commands
cannot be trusted with complex work.
The registry is the shell's self-knowledge —
the foundation of every intelligent feature above it."**

---
*"You cannot reason about what you cannot name.
The registry gives the shell names for everything it knows."* 🌲

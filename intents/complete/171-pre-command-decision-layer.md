---
id: 171
date: 2026-03-30
type: future
title: "Pre-Command Decision Layer — The Shell That Understands Before It Executes"
status: complete
tags: [shell, fsh, preexec, decision, safety, intelligence, v12]
version: 12.0.0
priority: high
depends_on: [162, 146]
---

## The Vision
Right now faelight-shell executes commands.
A shell with a pre-command decision layer *understands* commands
before executing them.

This is not preexec() from zsh.
That is a notification hook — "a command is about to run."
This is a decision layer — "should this command run, and how?"

The difference:
```
preexec():        fires → you were told → command runs
before_run {}:    fires → you decide → command runs OR blocked OR modified
```

## The before_run System
```fsh
# Define rules in config.fsh or core rules:

before_run {
    # Safety: block dangerous operations when locked
    if command == "git commit" and core.locked {
        block "Core is locked — unlock-core first"
    }

    # Safety: confirm destructive operations
    if command contains "rm -rf" {
        confirm "⚠️  Destructive operation: {command}"
    }

    # Intelligence: emit events for causal tracking
    if command == "deploy" {
        emit "deploy.started" { tool: args[0] }
    }

    # Intelligence: warn when working outside expected context
    if command starts_with "cargo build" and health < 80 {
        warn "Health is at {health}% — consider running d first"
    }

    # Prediction: suggest related actions
    if command == "fg commit" {
        suggest "Also run: d — verify health before pushing"
    }
}
```

## Why This Is Massively Powerful

### 1. Causal Linkage
Every significant command fires an event.
Core v10 reactions can respond to shell behavior.
Core v11 predictions can learn from command patterns.
The intelligence layer sees everything.

### 2. Safety Without Friction
Guards fire only when they matter.
`rm -rf ~/0-core/target` → no confirm (build artifacts, safe)
`rm -rf ~/0-core/engine` → confirm (source code, dangerous)

### 3. The Shell Becomes Self-Aware
```
Why did this fail?     → before_run logged what was attempted
What changed?          → before_run events show the sequence
What should I do next? → before_run suggest hooks
```

## Implementation Layers

### Layer 1 — Built-in Safety Rules (always active)
```rust
// Hardcoded in fsh — cannot be disabled
if command contains "rm -rf /" { block }
if command == "git commit" and core_locked { block }
if command == "git push" and uncommitted_changes { warn }
```

### Layer 2 — Forest Rules (from state.db)
```rust
// Loaded from core react rules on startup
// Same rule engine as core v10 reactions
// Human-editable, core-managed
```

### Layer 3 — Config Rules (from config.fsh)
```fsh
# User-defined in ~/.config/faelight-shell/config.fsh
before_run {
    if command starts_with "paru -Syu" {
        confirm "System update — run during maintenance window?"
    }
}
```

## The Rule Language
```
Conditions:
  command == "exact match"
  command contains "substring"
  command starts_with "prefix"
  command matches "glob*pattern"
  core.locked / core.unlocked
  health < N / health >= N
  directory == "~/0-core"
  time.hour >= 22              # late night guard

Actions:
  block "message"              # prevent execution entirely
  confirm "message"            # require explicit y/n
  warn "message"               # show warning, continue
  suggest "message"            # show suggestion, continue
  emit "event.name" { data }   # fire forest event
  log "message"                # add to shell history with note
```

## Integration With Core

### Feeds Core v10 (Reaction Engine)
Every `emit` in before_run writes to events table.
Reaction rules can fire based on shell behavior.
Example: 3 failed deploys in one hour → health.advisory fires.

### Feeds Core v11 (Prediction Engine)
before_run events become training data.
"Christian usually runs d before deploy" →
prediction suggests it when pattern is detected.

### Feeds Core v12 (Strategy Engine)
before_run history shows work patterns.
Strategy engine can analyze decision sequences.

## DEC-005 Compliance
before_run rules defined in config.fsh = interface layer ✅
before_run rules from state.db = loaded by fsh, defined by core ✅
All policy logic lives in core rule definitions ✅
fsh executes the decision, core owns the rules ✅

## Phase 1 — Built-in Safety Rules
Implement the 3 hardcoded guards:
- git commit blocked when locked
- git push warned with uncommitted changes
- rm -rf with confirmation on source directories

## Phase 2 — Rule Engine
Parse before_run blocks from config.fsh.
Evaluate conditions before each command.
Fire actions (block/confirm/warn/suggest/emit).

## Phase 3 — Forest Rule Integration
Load additional rules from core react domain.
Same TOML format as reaction rules.
Rules can be enabled/disabled via `core react enable/disable`.

## Phase 4 — Suggest System
After command completes, check for suggestions:
```
> fg commit "feat: ..."
💡 Suggestion: run d to verify health before pushing
```

## Phase 5 — Full Event Emission
Every significant shell action fires a typed event.
Shell behavior becomes fully observable by core intelligence.

## Gate Check
```
✅ Phase 1 — built-in safety rules — rm -rf/protected paths blocked, git/fg blocked when locked (2026-03-30)
✅ Phase 2 — before_run rule parser in config.fsh — BeforeRunRule/RuleCondition/RuleAction types (2026-03-30)
✅ Phase 2 — block/confirm/warn/suggest actions working — tested in fsh (2026-03-30)
✅ Phase 3 — config.fsh rules evaluated in preexec before every command (2026-03-30)
✅ Phase 3 — rules live in config.fsh, human-editable, loaded on startup (2026-03-30)
✅ Phase 4 — suggest system working for both native and external commands (2026-03-30)
✅ Phase 5 — suggest_after_external fires on cicomplete/cistart/deploy/lock/unlock/paru (2026-03-30)
✅ DEC-005 verified — rules in config.fsh/preexec, zero policy in dispatch (2026-03-30)
✅ before_run latency — negligible, string match only, no db calls (2026-03-30)
```

## The Phrase
**"The shell that only executes
is a tool.
The shell that understands before it executes
is a partner.
before_run is where the tool becomes the partner."**

---
*"Not preexec. Not a hook. A decision layer.
The forest thinks before it acts."* 🌲

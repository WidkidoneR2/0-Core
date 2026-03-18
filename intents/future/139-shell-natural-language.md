---
id: 139
date: 2026-03-18
type: future
title: "faelight-shell — Natural Language Pipeline Translation"
status: planned
tags: [shell, natural-language, ai, pipelines, v12]
version: 12.0.0
priority: medium
depends_on: [134, 135]
---

## The Vision

The shell understands what you mean, not just what you type.
```
forest> find biggest files
→ files | sort size desc | first 10

forest> show memory hogs
→ ps | sort memory desc | first 5

forest> which services are failing
→ services | where status == failed

forest> what changed recently
→ files | where modified < 10m | sort modified desc

forest> why is my computer slow
→ ps | sort cpu desc | first 5
   [checks memory, disk, network automatically]
   "faelight-bar is using 14% CPU — consider restarting"
```

## Philosophy

This is NOT bolted-on AI. It is pattern matching on structured data
the forest already has.

The forest knows:
- All table schemas (ps, files, services, ports, logs, tt, et...)
- All pipeline operations (where, sort, first, last, count, watch)
- All forest-specific concepts (health, intents, audit, events)
- The user's command history (what they actually use)

Natural language translation is just mapping intent to structure.

## The Translation Engine

### Layer 1 — Pattern Library (no AI required)

A curated library of natural language patterns → pipeline templates:
```toml
[[pattern]]
phrases = ["biggest files", "largest files", "most space"]
pipeline = "files | sort size desc | first 10"
context = "filesystem"

[[pattern]]
phrases = ["memory hogs", "using most memory", "ram usage"]
pipeline = "ps | sort memory desc | first 5"
context = "processes"

[[pattern]]
phrases = ["cpu hogs", "using most cpu", "slow processes"]
pipeline = "ps | sort cpu desc | first 5"
context = "processes"

[[pattern]]
phrases = ["failing services", "broken services", "service errors"]
pipeline = "services | where status == failed"
context = "services"

[[pattern]]
phrases = ["changed recently", "modified recently", "new files"]
pipeline = "files | where modified < 10m | sort modified desc"
context = "filesystem"

[[pattern]]
phrases = ["open ports", "listening ports", "network ports"]
pipeline = "ports"
context = "network"

[[pattern]]
phrases = ["unhealthy tools", "stale tools", "needs attention"]
pipeline = "tt | where score < 70 | sort score"
context = "forest"

[[pattern]]
phrases = ["recent commits", "latest changes", "git history"]
pipeline = "gc | first 10"
context = "git"

[[pattern]]
phrases = ["why slow", "computer slow", "system slow"]
pipeline = "ps | sort cpu desc | first 5"
followup = ["check memory", "check disk", "check events"]
context = "diagnosis"
```

### Layer 2 — Fuzzy Matching

If no exact pattern matches, use token similarity:
```
"show me what is eating memory" 
→ tokens: [memory, eating, show]
→ matches: memory → ps/memory, show → first 5
→ generates: ps | sort memory desc | first 5
→ confidence: 0.78
```

Always shows the generated pipeline before executing:
```
forest> show me what is eating memory
  → ps | sort memory desc | first 5  (78% confidence)
  Run? [y/n/edit]:
```

### Layer 3 — History-Aware (Phase 11 foundation)

Learn from what you actually run:
```
If user frequently runs: ps | sort cpu desc | first 5
Then "slow" should map to that pipeline with high confidence.
```

Stored in state.db. Improves over time.

### Layer 4 — Forest-Specific Understanding

The shell knows forest concepts no generic AI would:
```
"check the forest"     → health
"what's planned"       → intents | where status == planned
"recent decisions"     → dt | last 5
"audit scores"         → tt | sort score | first 10
"what happened today"  → et today
```

## Activation
```
forest> ?find biggest files
```

The `?` prefix activates natural language mode explicitly.
Without `?` the shell tries pattern matching silently.
If confidence < 0.6, asks for confirmation.

Or as a command:
```
forest> ask find biggest files
forest> nl show memory hogs
```

## Integration with Core v9

When Core v9 Intent engine generates a goal, the shell can express it:
```
Core v9: "Goal: reduce dependency risk"
Shell:   "Run: core deps risk  (generated from goal)"
```

## Pattern Library Location
```
~/.config/faelight-shell/nl-patterns.toml
~/0-core/01-registry/shell-patterns.toml  (forest-specific)
```

User can add custom patterns:
```toml
[[pattern]]
phrases = ["my work today", "what did I do"]
pipeline = "et today | where domain == git"
context = "personal"
```

## Success Criteria

- ⬜ Pattern library — 30+ patterns covering common queries
- ⬜ ?prefix activates natural language mode
- ⬜ Generated pipeline shown before execution
- ⬜ Confidence score displayed
- ⬜ User can confirm, reject, or edit generated pipeline
- ⬜ Forest-specific patterns (health, intents, decisions, audit)
- ⬜ History-aware pattern weighting
- ⬜ Custom pattern support via TOML
- ⬜ Core v9 goal → shell command translation

## The Phrase

**"A shell that understands you
does not replace your knowledge.
It amplifies it."**

---
*"Not magic. Pattern recognition on structured wisdom.
The forest already knows. Now it listens."* 🌲

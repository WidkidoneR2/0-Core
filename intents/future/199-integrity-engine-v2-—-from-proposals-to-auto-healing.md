---
id: 199
date: 2026-04-05
type: planned
title: "Integrity Engine v2 — From Proposals to Auto-Healing"
status: planned
tags: [integrity, auto-heal, checks, v2, doctor]
---
## Current State
The integrity engine (INT-184) runs 13 checks across 8 categories.
It detects issues and proposes fixes. But `core integrity apply`
does not actually apply fixes — it only shows proposals again.
Integrity is 100% detective, 0% corrective.

## What v2 Fixes

### Fix core integrity apply
The most critical fix — apply actually applies the fix.
Each proposal has a concrete action:
- move file: execute the move
- remove reference: execute the sed command
- update registry: write the TOML change
With confirmation prompt and rollback on failure.

### Expanded Check Coverage
Current checks are surface-level. v2 goes deeper:
- Detect duplicate tool functionality (two tools doing same thing)
- Detect dead aliases (alias points to non-existent command)
- Detect orphaned state.db entries (references to retired tools)
- Detect config drift (symlink target differs from expected)
- Detect version mismatch (binary version vs registry version)

### Continuous Integrity (not just on-demand)
Currently: run core integrity run manually.
v2: integrity checks run in background via faelight-contextd.
Critical issues surface as insights immediately.
Minor issues queued for next d run.

### Integrity Score Trend
Currently: point-in-time score.
v2: track integrity score over time.
"Integrity has been 100% for 7 days — stable"
"Integrity dropped from 100% to 67% after last deploy — investigate"

### Self-Healing Protocol
For safe, reversible fixes (dead aliases, orphaned entries):
core integrity heal         — apply all safe auto-fixes
core integrity heal --dry   — show what would be healed
Unsafe fixes still require manual confirmation.

## Commands
core integrity run          — run all checks
core integrity apply <n>    — actually apply fix n (v2: works)
core integrity heal         — apply all safe auto-fixes
core integrity heal --dry   — preview auto-fixes
core integrity trend        — integrity score over time
core integrity expand       — show all check categories

## Gate Check
⬜ core integrity apply actually applies fixes
⬜ Rollback on failed apply
⬜ Dead alias detection
⬜ Orphaned state.db entry detection
⬜ Version mismatch detection
⬜ Continuous integrity via contextd
⬜ Integrity score trend tracking
⬜ core integrity heal — safe auto-healing

## The Phrase
"An integrity engine that only detects
is a smoke detector with no sprinklers.
v2 detects and heals." 🌲

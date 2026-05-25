---
id: 339
title: "Jarvis Purge -- remove all jarvis references, tables, and checks from the forest"
status: in-progress
date: 2026-05-25
tags: [jarvis, purge, cleanup, friday, technical-debt, integrity, ghost-removal]
---

## The Problem

Jarvis was the original name for what became Friday. The rename happened
incrementally -- Friday emerged, Jarvis faded -- but the code never fully
followed. As of 2026-05-25, Jarvis ghosts exist throughout the forest:

- `jarvis_readiness_log` table referenced but never created -- caused 67%
  integrity drift for an unknown period
- `Category::Jarvis` in the integrity system
- `JarvisLogFreshnessCheck` in the integrity suite
- `jarvis_*` tables still referenced in queries
- `jarvis_*` naming in state.db schema
- Lingering `jarvis` references in commands, docs, and config

These ghosts caused real damage: the integrity score showed 67% for weeks
while the system was actually healthy. Every `d` run was lying.

This is not acceptable. The forest must not lie to itself.

## Why This Matters for the Conference

If the forest is presented at USENIX HotOS, WCRE, MSR, or ASE, it must
be coherent. Jarvis ghosts are incoherence. A system that references
its own dead predecessor is not a system that thinks clearly.

## Why This Matters for NixOS

The NixOS migration (INT-340) starts fresh. Before anything moves to
NixOS, Jarvis must be completely gone from the Arch forest. The NixOS
forest will never know Jarvis existed.

## The Full Purge

### Step 1: Find everything

Audit every Jarvis reference in the entire forest:
- Source code (engine/src, rust-tools/*)
- State.db tables and schema
- Config files
- Documentation
- Intent files
- Scripts

### Step 2: Database cleanup

Tables to check and drop if Jarvis-era:
- jarvis_readiness_log (referenced but may not exist)
- jarvis_log (may exist)
- Any table with jarvis_ prefix

### Step 3: Code cleanup

- Remove all `jarvis_*` function names
- Remove all `Category::Jarvis` references
- Remove all `jarvis_*` check names
- Replace any remaining Jarvis terminology with Friday equivalents

### Step 4: Verify

After purge:
- `grep -r "jarvis" ~/0-core/engine/src/ | grep -v target` returns nothing
- `grep -r "jarvis" ~/0-core/rust-tools/` returns nothing
- `sqlite3 state.db ".tables"` shows no jarvis_ tables
- `d` shows 100% integrity consistently
- 10 consecutive `d` runs all show 100% integrity

## Gates

✅ Full audit: 36 refs in engine, 8 in rust-tools -- all identified 2026-05-25
✅ jarvis_readiness_log identified and dropped from state.db 2026-05-25
✅ jarvis_readiness_log removed from all queries -- renamed to friday_readiness_log 2026-05-25
✅ Category::Jarvis removed from integrity system 2026-05-25
✅ JarvisLogFreshnessCheck retired, all other checks renamed 2026-05-25
✅ compute_jarvis_score→friday_score, get_jarvis_score→friday_score, jarvis()→friday_readiness() 2026-05-25
✅ faelight-shell audited -- theme jarvis→friday, prompt vars renamed 2026-05-25
⏸ Documentation audit -- deferred: historical docs may mention Jarvis as context, not functional -- approved by: christian 2026-05-25
⏸ Intent file audit -- deferred: historical intent files may reference Jarvis, acceptable -- approved by: christian 2026-05-25
✅ grep returns zero results -- verified 2026-05-25
✅ grep returns zero results -- verified 2026-05-25
✅ jarvis_readiness_log dropped -- no jarvis tables in state.db 2026-05-25
⏸ 10 consecutive d runs -- deferred: track over next sessions -- approved by: christian 2026-05-25
✅ Demonstrated: full session, zero jarvis refs in source or database 2026-05-25

---
id: 157
date: 2026-03-26
type: future
title: "faelight-docs v2 — The Forest Documents Itself Accurately"
status: in-progress
tags: [docs, automation, readme, release, accuracy, v12]
version: 12.0.0
priority: high
---

## The Problem
faelight-docs v1 updates the dynamic README section but reads stale data.
Tool counts, health scores, and intent numbers are hardcoded or cached.
Every release requires manual verification and correction.
The forest that values accuracy must document itself accurately.

## The Solution
faelight-docs v2 reads live data from the forest at release time:
- Tool count from path resilience check (not hardcoded)
- Health score from most recent doctor run
- Intent counts from actual ledger scan
- Core version from /etc/faelight/VERSION
- Commit count from git log
- Active intents from in-progress status scan

## Phase 1 — Live Data Reads
Replace all hardcoded values with live queries:
```rust
// Instead of: tools: 67
// Do: scan ~/0-core/scripts/ and count deployed binaries

// Instead of: health: 95%
// Do: read ~/.cache/faelight/health or state.db

// Instead of: intents: 101
// Do: count intents/complete/*.md
```

## Phase 2 — Smart Dynamic Section
The dynamic section regenerates with full accuracy on every release:
- Latest release block — from CHANGELOG.md last entry
- Stats block — live counts, not cached
- Badge URLs — accurate health and version

## Phase 3 — Release Diff Preview
Before publishing, show what will change:
```bash
faelight-docs preview
# Shows: health 95% → 100%, tools 67 → 44, intents 101 → 113
# Confirm? (y/n)
```

## Phase 4 — Auto-sync on Version Bump
When core release runs, faelight-docs sync runs automatically with
verified live data. No manual step. No drift.

## Gate Check
```
✅ Phase 1 — live tool count from tools.toml — matches doctor exactly (2026-03-31)
✅ Phase 1 — live health from ~/.cache/faelight/health-status (2026-03-31)
✅ Phase 1 — live intent count mirrors doctor logic — all categories scanned (2026-03-31)
✅ Phase 2 — dynamic section fully accurate — tools/health/intents all live (2026-03-31)
✅ Phase 3 — verify-links command — scans all README links before release (2026-03-31)
✅ Phase 4 — deploy faelight-docs works — added to deploy script (2026-03-31)
✅ README never shows stale data — live reads verified, link check on every sync (2026-03-31)
```

## The Phrase
**"A forest whose map is wrong
leads travellers astray.
The forest that documents itself accurately
trusts no cache — only truth."**

---
*"faelight-docs v2 is not a documentation tool.
It is the forest's commitment to honesty about itself."* 🌲

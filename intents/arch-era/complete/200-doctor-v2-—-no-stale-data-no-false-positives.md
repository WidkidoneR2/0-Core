---
id: 200
date: 2026-04-05
type: planned
title: "Doctor v2 — No Stale Data, No False Positives"
status: complete
tags: [doctor, health, stale, v2, accuracy, checks]
---
## The Problem
The health doctor (d) is the most-used command in the forest.
It must be perfectly accurate. Currently:
- Some checks show stale cached data
- Hardcoded tool names that should respect registry
- Check messages are sometimes outdated after changes
- Security audit timestamp can be weeks old without warning
- Forecast can show misleading trends during active development

## What v2 Fixes

### No Stale Data Policy
Every check must verify live state, not cached state.
If a check uses cached data, it must show the cache age.
Security audit: show days since last scan prominently.
"Security Audit: 25 findings (scanned 3 days ago — consider rescan)"

### Registry-Aware Checks
All tool checks must respect registry retired status.
Currently: archaeology check is hardcoded — we patched it manually.
v2: all tool checks read retired status from registry dynamically.
No more manual patches needed when retiring tools.

### Forecast Accuracy
Current forecast can show -7.7 trend during active retirement work
(which is expected) but no context is provided.
v2: forecast includes context:
"trend: -4.3 (tool retirement in progress — expected)"
Context comes from active intents and recent commit types.

### Check Freshness Indicators
Each check shows when it was last verified:
✅ Security Audit    25 findings (3 days ago)
✅ Path Resilience   44/44 (just now)
Stale checks (>24h) shown with age indicator.

### Doctor Profiles
Different contexts need different doctor runs:
d --quick    — only critical checks (5 seconds)
d --full     — all 23 checks (current behavior)
d --security — security-focused checks only
d --tools    — tool deployment checks only

### False Positive Elimination
Audit stale tools check currently flags tools with low audit scores
even when they are recently deployed and working.
v2: distinguish between "low score" and "broken" — different severity.

### Predictive Health
Based on current trends and active work:
"Health likely to dip during fsh v4 development — normal"
Uses active intents + historical health patterns.

## Commands
d                    — full doctor run (current)
d --quick            — critical checks only
d --full             — all checks with freshness indicators
d --security         — security-focused
d --tools            — tool deployment focus
core doctor freshen  — update all cached check data
core doctor history  — health score over time

## Gate Check
✅ No stale data — all checks verify live state
✅ Security audit shows days-since-scan — "scanned today", "3 days ago", etc.
✅ Registry-aware checks — retired tools respected dynamically
✅ Forecast includes context from active intents
✅ Check freshness indicators on all checks
✅ Doctor profiles — core doctor quick live (6 critical checks, fast)
✅ False positive elimination — score vs broken distinction
✅ Predictive health based on active work patterns
✅ core doctor history — health trend from horizon_snapshots

## The Phrase
"The doctor that shows stale data
is worse than no doctor.
v2 shows only truth,
always fresh,
always in context." 🌲

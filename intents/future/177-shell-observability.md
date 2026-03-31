---
id: 177
date: 2026-03-30
type: future
title: "Shell Observability — The Shell Watches Itself"
status: in-progress
tags: [shell, fsh, observability, sessions, self-aware, intelligence, metrics]
version: 12.0.0
priority: medium
depends_on: [146, 173, 174, 176]
---

## The Problem
faelight-shell is structured and partially observable through doctor.
But the shell itself cannot answer the questions that matter most:
```
What changed between this session and last session?
Why is this session different from usual?
What commands do I run most in this directory?
What is my actual workflow — not what I think it is?
```

Without answers to these questions, the shell is observable in theory
but blind in practice.

The current state:
```
structured    ✅  — everything is intentional
observable    ⚠️  — doctor sees health, shell cannot see itself
self-aware    ❌  — cannot answer why, what changed, what next
```

This intent bridges observable ⚠️ to self-aware ✅.

## The Solution
Add `observe` as a first-class shell primitive:
```fsh
observe session          # summary of current session activity
observe diff             # what changed vs last session
observe commands         # most used commands this session
observe timing           # slowest commands this session
observe anomalies        # things that look different from normal
observe patterns         # learned patterns from session history
```

## Session Summary
```fsh
observe session
# 🌲 Session Summary (2026-03-30 09:14 → now)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Duration:    47 minutes
# Commands:    83 total, 4 failed (95% success)
# Directories: ~/0-core (71%), ~/0-core/engine (18%), other (11%)
# Most used:   d (12), fg commit (8), deploy (5), cargo build (4)
# Intents:     INT-172, INT-163, INT-151
# Commits:     3 (1736 → 1739)
```

## Session Diff
```fsh
observe diff
# 🔄 Changes since last session
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Commits:     +3 (1736 → 1739)
# Intents:     +2 complete (163, 172)
# Health:      stable at 100%
# New files:   docs/COMMAND-GUIDE.md
#              intents/complete/163-alias-audit.md
# Tools:       no change
```

## Anomaly Detection
```fsh
observe anomalies
# ⚠️  Anomalies detected this session:
# · 4 failures (usual: 0-1 per session)
# · 2 permission errors (E_PERMISSION — core lock?)
# · Session 40% longer than your median (47 min vs 28 min)
```

## Integration With Core Intelligence
observe feeds data into:
- core v11 predictions: learns session patterns
- core v12 strategy: uses session diffs for planning
- before_run (INT-171): anomalies can trigger warnings

## Phase 1 — Session Metrics Collection
Collect during every session:
- command count, success/failure rate
- directory time distribution
- most used commands
- session duration

## Phase 2 — observe session
Display session summary on demand.

## Phase 3 — observe diff
Compare current session state to last session snapshot.

## Phase 4 — observe timing
Show slowest commands, track performance over time.

## Phase 5 — observe anomalies
Detect sessions that differ significantly from learned patterns.
Feed anomalies into core v11 as signals.

## Gate Check
```
✅ Session metrics collected — commands/failures from shell_state and shell_history (2026-03-31)
✅ observe session — commands, success rate, commits, active intent (2026-03-31)
✅ observe diff — commits, failures, errors delta (2026-03-31)
✅ observe commands — top 10 commands by frequency as structured table (2026-03-31)
✅ observe patterns — top 5 command patterns from full history (2026-03-31)
✅ observe anomalies — detects high failure rate and permission errors (2026-03-31)
✅ All observe subcommands live — session/commands/diff/anomalies/patterns (2026-03-31)
✅ Shell state observable — data available for core v11/v12 analysis (2026-03-31)
✅ Anomaly detection live — before_run integration deferred to INT-179 (2026-03-31)
```

## The Phrase
**"A shell that cannot watch itself
cannot improve itself.
observe is not monitoring.
It is the shell turning its own gaze inward —
asking the questions that make it
more than a tool."**

---
*"structured ✅  observable ✅  self-aware ✅
That is the progression.
observe is the bridge from the second to the third."* 🌲

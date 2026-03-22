---
id: 143
date: 2026-03-21
type: future
title: "faelight-digest — Morning Forest Summary"
status: planned
tags: [digest, summary, shell, session, morning, v11]
version: 11.0.0
priority: medium
depends_on: [135, 120]
---

## The Vision

When you open the terminal each morning, the forest greets you
with a summary of what happened while you were away and
what matters most today.
```
🌲 Good morning. The forest has been thinking.

Since last session (8 hours ago):
  → 3 new commits pushed
  → Health steady at 95%
  → forecast +0.6 trending up

Today's focus:
  → INT-120 Phase 17 — event system (next logical step)
  → Core v8 Phase 5 — evolution proposals

Worth noting:
  → dispatcher.rs changed 57 times — highest churn file
  → faelight-notify audit score dropped to 62
  → 2 pending decisions older than 30 days

"The roots hold. The branches grow."
```

## Data Sources

All data already exists — no new infrastructure needed:

- `session_state` table — last session timestamp, commit count
- `gc` via faelight-git — commits since last session
- `core doctor` — health score
- `core forecast` — forecast trend
- `file_index` — recently changed files
- `audit_scores` — tool scores
- `decisions` table — pending decisions
- intents/future — in-progress intent titles

## Where It Shows

**Option A — faelight-shell welcome (preferred):**
Replace the current welcome screen with digest output.
Already wired in main.rs `print_welcome()`.

**Option B — faelight-term startup:**
faelight-term launches faelight-shell which shows digest.
When faelight-shell is the daily driver, this happens naturally.

## Build Plan

Single new file: `src/digest.rs` in faelight-shell.
Called from `print_welcome()` when gap > 4 hours.
Short gap (< 1 hour): minimal — just momentum line.
Long gap (> 4 hours): full digest.
Morning (5am-10am): full digest always.

## Gate Check
```
✅ Commits since last session (2026-03-21)
✅ Health + forecast trend (2026-03-21)
✅ Active intents summary (2026-03-21)
⬜ Top churn files (from file_index + git)
✅ Pending decisions reminder (2026-03-21)
✅ Low audit score tools (2026-03-21)
✅ Time-aware greeting (2026-03-21) (morning/evening/night)
✅ Replaces print_welcome on long gaps (2026-03-21)
```

## The Phrase

**"A forest that knows what changed while you slept
is a forest that never loses context.
Every morning, the forest catches you up."**

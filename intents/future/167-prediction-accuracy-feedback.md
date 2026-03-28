---
id: 167
date: 2026-03-28
type: future
title: "Prediction Accuracy Feedback Loop — Close the Learning Circle"
status: planned
tags: [prediction, accuracy, feedback, learning, v11, v12]
version: 12.0.0
priority: high
depends_on: [148]
---

## The Problem
Core v11 Prediction Engine (INT-148) is built.
9 predict commands work correctly.
But the feedback loop is broken:
```bash
core predict accuracy
# Shows:
# · 0 predictions generated
# · 0 outcomes recorded
# · Confidence: 85% HIGH (based on data volume)
```

The `forest_predictions` table is NEVER written.
Predictions are computed on demand but never stored.
So `predict accuracy` can never measure actual accuracy.
The calibration system is a skeleton with no flesh.

This means:
- The forest cannot learn if its predictions were right
- Confidence score is based on data volume, not actual accuracy
- v12 Strategy will plan on uncalibrated predictions
- Jarvis readiness score cannot be honest

## Why This Matters
The difference between a prediction engine and an intelligence:
```
Prediction engine:  computes answer → displays → forgets
Intelligence:       computes answer → displays → stores → compares → learns
```

Right now fsh has a prediction engine.
This intent adds the learning loop that makes it intelligence.

## What Needs Storing
Every prediction that can be verified should be stored:
```sql
-- forest_predictions (already exists, never written)
INSERT INTO forest_predictions (
    kind,           -- "sessions" | "health" | "intents" | "coupling"
    prediction,     -- the actual prediction text
    confidence,     -- 0-100
    evidence,       -- JSON: what data supports this
    created_at,     -- when predicted
    expires_at      -- when this prediction can be verified
);

-- prediction_outcomes (already exists, never written)
INSERT INTO prediction_outcomes (
    prediction_id,  -- links to forest_predictions
    actual,         -- what actually happened
    correct,        -- boolean: was prediction right?
    delta,          -- how far off was it?
    verified_at     -- when verified
);
```

## Phase 1 — Store Predictions on Generation
When `core predict sessions` runs, store the prediction:
```rust
// After computing prediction:
ctx.runtime.db.execute(
    "INSERT INTO forest_predictions (kind, prediction, confidence, evidence, created_at, expires_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    params![
        "sessions",
        "Thursday is your most active build day",
        85,
        json_evidence,
        now,
        now + 7_days  // verify in a week
    ]
)?;
```

Apply to all 9 predict commands.

## Phase 2 — Automatic Verification
When `core predict health` predicted "100% in 3 runs"
and doctor has now run 3 times — verify automatically:
```rust
// On each doctor run, check expired predictions:
let expired = query_expired_predictions(&ctx)?;
for pred in expired {
    let actual = get_actual_outcome(&ctx, &pred)?;
    record_outcome(&ctx, pred.id, actual)?;
}
```

## Phase 3 — Accuracy Dashboard
```bash
core predict accuracy
# Shows:
# · 47 predictions stored (last 30 days)
# · 38 verified (80% verification rate)
# · 31/38 correct (81% accuracy)
# · Best: health trajectory (94% accurate)
# · Worst: intent velocity (61% accurate)
# · Confidence calibration: well-calibrated
```

## Phase 4 — Calibrated Confidence
Once accuracy data exists, confidence scores become honest:
```
85% confidence + 81% actual accuracy = well calibrated
85% confidence + 45% actual accuracy = overconfident → adjust down
```

Auto-adjust confidence thresholds based on measured accuracy.

## Phase 5 — Jarvis Score Integration
`core strategy jarvis` (v12) reads prediction accuracy:
```
Jarvis readiness factors:
  Prediction accuracy > 75%   → +10 points
  Reaction accuracy > 80%     → +10 points
  Health trajectory accuracy  → +5 points
```

## Gate Check
```
⬜ forest_predictions written on every predict command
⬜ prediction_outcomes written on verification
⬜ Automatic verification on doctor run
⬜ core predict accuracy shows real numbers
⬜ core predict history shows past predictions vs actual
⬜ Confidence scores calibrated to measured accuracy
⬜ Jarvis score reads prediction accuracy
⬜ 30 days of predictions stored and verified
```

## The Phrase
**"A prediction that is never verified
is not a prediction.
It is a guess with confidence clothing.
The forest must know if it was right."**

---
*"Closing the feedback loop is the difference
between a system that computes
and a system that learns."* 🌲

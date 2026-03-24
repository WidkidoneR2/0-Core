---
id: 148
date: 2026-03-25
type: future
title: "Core v11 — Prediction: The Forest Anticipates"
status: planned
tags: [core, v11, prediction, patterns, anticipation, ai, v13]
version: 13.0.0
priority: medium
depends_on: [140]
---

## The Core Timeline

| Version | Capability | Meaning |
|---------|-----------|---------|
| v9  | Intent    | the forest chooses where to grow |
| v10 | Reaction  | the forest responds without being asked |
| **v11** | **Prediction** | **the forest anticipates before it happens** |
| v12 | Strategy  | the forest plans across multiple horizons |
| v13 | Autonomy  | the forest chooses its own purpose |

## The Core Insight

v10 Reaction responds to what IS.
v11 Prediction responds to what WILL BE.

The difference between a system with reflexes and a system with foresight.

v11 reads your patterns — session timing, build cadence, health cycles,
intent completion velocity — and surfaces insights before you need them.

> *"You typically build on Tuesday and Wednesday.
>  Your health tends to drop after major intent sprints.
>  At current pace, INT-146 completes in 2 more sessions.
>  Coupling in dispatcher.rs will become critical in ~3 sessions."*

The forest doesn't wait for health to drop.
It sees the trajectory and speaks before the fall.

## The Human-Computer Prediction Model

Humans predict using intuition and experience.
Computers predict using data and algorithms.
Together they create better decisions than either alone.

v11 is the computer half of that partnership.
You bring the intuition. The forest brings the data.

## The Five Pillars

### Pillar 1 — Session Pattern Recognition
The forest learns your work rhythms.
```bash
core predict sessions     # when do you typically work?
core predict cadence      # commit frequency and burst patterns
core predict focus        # which domains get attention when?
```

### Pillar 2 — Health Trajectory Forecasting
Beyond the current forecast — multi-session health prediction.
```bash
core predict health       # projected health over next 7 sessions
core predict decline      # early warning before health drops
core predict recovery     # how long to recover from current state
```

### Pillar 3 — Intent Velocity
How fast are intents completing? What's the backlog trajectory?
```bash
core predict intents      # estimated completion dates
core predict backlog      # when will the backlog stabilize?
core predict next         # what's most likely to ship next?
```

### Pillar 4 — Coupling Forecasting
Where is the architecture heading? What will become critical?
```bash
core predict coupling     # which domains will hit critical coupling?
core predict churn        # which files will need attention soon?
core predict debt         # technical debt trajectory
```

### Pillar 5 — Prediction Confidence
Every prediction includes its evidence and confidence score.
```bash
core predict explain <id> # why does the forest predict this?
core predict history      # past predictions vs actual outcomes
core predict accuracy     # how accurate have predictions been?
```

## state.db Tables
```
forest_predictions  — generated predictions with confidence scores
prediction_outcomes — actual outcomes vs predicted (for calibration)
session_patterns    — learned session timing and rhythm data
```

## Build Order
```
Phase 1 — Session Pattern Engine (learn rhythms from git/event history)
Phase 2 — Health Trajectory (multi-session health forecasting)
Phase 3 — Intent Velocity (completion rate and backlog projection)
Phase 4 — Coupling Forecasting (architectural debt prediction)
Phase 5 — Prediction Confidence (accuracy tracking and calibration)
```

## Gate Check
```
⬜ Core v10 complete — reaction engine ready
⬜ Phase 1 — Session Pattern Engine
⬜ Phase 2 — Health Trajectory
⬜ Phase 3 — Intent Velocity
⬜ Phase 4 — Coupling Forecasting
⬜ Phase 5 — Prediction Confidence
```

## The Phrase

**"A forest that sees tomorrow
does not fear today.
It prepares."**

---
*"v11 is not clairvoyance. It is pattern recognition
applied with honesty about what it knows and what it doesn't."* 🌲

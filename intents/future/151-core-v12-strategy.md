---
id: 151
date: 2026-03-26
type: future
title: "Core v12 — Strategy: The Forest Plans Across Horizons"
status: in-progress
tags: [core, v12, strategy, planning, horizons, jarvis, autonomy]
version: 12.0.0
priority: high
---

## The Core Timeline
| Version | Capability | Meaning |
|---------|-----------|---------|
| v9  | Intent     | the forest chooses where to grow |
| v10 | Reaction   | the forest responds without being asked |
| v11 | Prediction | the forest anticipates before it happens |
| **v12** | **Strategy** | **the forest plans across multiple horizons** |
| v13 | Autonomy   | the forest chooses its own purpose |

## The Core Insight
v11 Prediction tells you what WILL happen.
v12 Strategy tells you what TO DO about it.

The difference between a system with foresight and a system with judgment.

v11 says: "Health will drop in 2 sessions based on current trajectory."
v12 says: "Here is the sequence of actions that prevents that drop,
           ordered by impact, feasibility, and alignment with your goals."

v12 is the Jarvis layer. Not the voice — the mind behind the voice.

## What v12 Is NOT
v12 does not execute. It does not automate. It does not act without
explicit human authorization. That is v13 territory and requires deep
trust earned over time.

v12 proposes. You decide. The forest remembers.

## The Four Pillars

### Pillar 1 — Horizon Planning
The forest maintains three planning horizons simultaneously.
```bash
core strategy now        # what needs attention in this session?
core strategy week       # what should the next 7 days focus on?
core strategy quarter    # what is the 90-day arc toward Jarvis?
```

Each horizon synthesizes:
- Active v9 goals
- v10 reaction history (what the forest has been signaling)
- v11 predictions (where things are heading)
- INT-137 architectural horizons (known future limits)

### Pillar 2 — Action Sequencing
Given a goal and current state, propose the optimal sequence of actions.
```bash
core strategy sequence GOAL-001     # optimal path to this goal
core strategy unblock               # what is blocking the most progress?
core strategy tradeoff <action>     # what do we give up to do this now?
```

### Pillar 3 — Cross-Intent Coherence
Multiple intents in flight can conflict. v12 detects and resolves this.
```bash
core strategy conflicts             # which intents are pulling in opposite directions?
core strategy coherence             # is the current work plan internally consistent?
core strategy merge GOAL-001 GOAL-002  # can these goals be pursued together?
```

### Pillar 4 — Jarvis Readiness
v12 is the final gate before v13 Autonomy. It tracks readiness.
```bash
core strategy jarvis                # how close is the forest to Jarvis-level capability?
core strategy trust                 # what evidence would justify more autonomy?
core strategy gap                   # what capabilities are missing for full Jarvis?
```

## The Jarvis Readiness Score
v12 maintains a running Jarvis readiness score (0-100):

| Score | Meaning |
|-------|---------|
| 0-20  | Basic tool (what we were at v2) |
| 20-40 | Aware system (v5 Intelligence) |
| 40-60 | Reactive assistant (v10 Reaction — current) |
| 60-80 | Anticipatory partner (v11 Prediction) |
| 80-95 | Strategic advisor (v12 Strategy) |
| 95-100| Autonomous agent (v13 Autonomy) |

Current estimated score: **65/100** — anticipatory partner territory.
v11 shipped (2026-03-26) and pushed score from 45 → 65.
v12 targets ~85. v13 is the destination.

## What v12 Needs From v10 and v11
```
v10 reaction engine    → what signals has the forest been surfacing?
v11 prediction engine  → what is the forest anticipating?
v9 goal engine         → what has the human authorized?
INT-137 horizons       → what architectural limits are approaching?
state.db event history → 9,750+ events of real forest behavior
```

## state.db Tables
```
forest_strategies      — generated strategy proposals
strategy_outcomes      — did following the strategy help?
horizon_snapshots      — periodic state of all three horizons
jarvis_readiness_log   — readiness score history over time
```

## Build Order
```
Phase 1 — Horizon Engine (now/week/quarter planning synthesis)
Phase 2 — Action Sequencing (optimal path generation)
Phase 3 — Cross-Intent Coherence (conflict detection and resolution)
Phase 4 — Jarvis Readiness Tracking (score + gap analysis)
Phase 5 — Strategy Memory (did past strategies work?)
```

## Gate Check
```
✅ Core v11 complete — prediction engine ready (2026-03-26) — 9 predict commands, 85% HIGH confidence
✅ Phase 1 — Horizon Engine — now/week/quarter commands live, horizon_snapshots table created (2026-03-30)
✅ Phase 2 — Action Sequencing — sequence/unblock/tradeoff commands live (2026-03-30)
✅ Phase 3 — Cross-Intent Coherence — conflicts/coherence/merge commands live (2026-03-30)
⬜ Phase 4 — Jarvis Readiness Tracking
⬜ Phase 5 — Strategy Memory
```

## Relationship to Other Intents
```
INT-140 Core v10  — reaction engine that v12 synthesizes
INT-148 Core v11  — prediction engine that v12 acts on
INT-137           — architectural horizons v12 plans around
INT-142/147       — voice input/output that will voice v12 strategies
```

## The Phrase
**"A forest that predicts the storm
and plans the shelter
before the first cloud appears —
that is not intelligence.
That is wisdom."**

---
*"v12 is not automation. It is the forest becoming
a genuine thinking partner — one that earns trust
by being right more often than it is wrong."* 🌲

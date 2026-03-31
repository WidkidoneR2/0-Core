---
id: 181
date: 2026-03-30
type: feature
title: "Forest Next Intent Engine — The Forest Knows What Comes Next"
status: planned
tags: [core, intelligence, strategy, intents, v12, autonomy, planning]
version: 12.1.0
---

## Vision
The forest tells you what to work on next.
Not based on intent numbers. Not based on your mood.
Based on dependencies, health, velocity, and the path to partnership.
```
core strategy next
→ Recommended: INT-168 — Test Suite Foundation
   Reason: INT-162, 171, 173, 174 all complete — foundation needs tests before v12 builds more
   Confidence: 87%
   Alternative: INT-180 — Sway Removal (quick win, low risk)
```

## Why This Exists
Right now the forest has:
- 128 complete intents
- 18 planned intents
- Core v12 Strategy engine
- Prediction engine (v11)
- Reaction engine (v10)

But none of these answer: "What should I work on right now?"
That question is answered by intuition. That is not good enough for a partner system.

## The Algorithm
```
score(intent) =
    dependency_readiness     × 0.35   (are prerequisites done?)
  + health_impact            × 0.25   (does this fix something urgent?)
  + velocity_alignment       × 0.20   (does this match current momentum?)
  + presentation_proximity   × 0.15   (does this matter for summer?)
  + complexity_fit           × 0.05   (is this the right size for now?)
```

## Dependency Graph
The engine reads intent files and extracts:
- Explicit dependencies (mentions of INT-NNN in body)
- Implicit dependencies (tags that share domains)
- Completion blockers (gate checks with ⬜)

## Commands
```
core strategy next              # top recommendation with reasoning
core strategy next --list       # ranked list of all planned intents
core strategy next --why INT-168 # explain why this is ranked here
core strategy queue             # ordered work queue for next 5 sessions
core strategy blockers          # what is blocking the most progress
```

## Integration With INT-161
INT-161 describes the build order philosophically.
INT-181 implements it as a running engine that updates dynamically.
Together they answer: "What comes next and why?"

## Stepping Stone to v13 Autonomy
v13 (INT-156) is the forest choosing its own purpose.
INT-181 is the stepping stone:
- v12 Strategy: forest PROPOSES what comes next
- INT-181: forest RANKS and EXPLAINS the proposal
- v13 Autonomy: forest ACTS on the proposal with permission

## Phase 1 — Dependency Parser
Read all planned intent files.
Extract INT-NNN references as dependency edges.
Build directed graph.

## Phase 2 — Scoring Engine
Implement the scoring algorithm.
Weight each factor.
Produce ranked list.

## Phase 3 — core strategy next
Wire into core strategy domain.
Display recommendation with reasoning.

## Phase 4 — Feedback Loop
When an intent completes, re-score all remaining intents.
The queue updates dynamically.
Core v11 prediction feeds into velocity_alignment score.

## Gate Check
```
⬜ Dependency graph built from intent files
⬜ Scoring algorithm implemented — 5 factors weighted
⬜ core strategy next — top recommendation with reasoning
⬜ core strategy next --list — ranked queue of all planned
⬜ core strategy queue — 5-session work plan
⬜ core strategy blockers — what is blocking most progress
⬜ Queue updates dynamically after each cicomplete
⬜ Integrated with INT-161 build order philosophy
⬜ Jarvis readiness score factors in strategy engine quality
```

## The Phrase
**"A partner that cannot say what comes next
is not a partner.
It is a tool waiting to be told.
The forest knows what comes next.
Ask it."**

---
*"core strategy next — three words that change the relationship."* 🌲

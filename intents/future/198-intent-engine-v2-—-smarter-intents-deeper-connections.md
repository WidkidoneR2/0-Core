---
id: 198
date: 2026-04-05
type: planned
title: "Intent Engine v2 — Smarter Intents, Deeper Connections"
status: in-progress
tags: [intents, engine, genealogy, auto-link, v2, intelligence]
---
## Current State
The intent engine (v9, INT-133) is solid. 150 complete intents.
But it operates on intents individually — no awareness of relationships,
no auto-detection of patterns, no assistance in creation.

## What v2 Adds

### Auto-Linking
When you create an intent, the engine suggests related intents.
"INT-194 (fsh v4) likely relates to INT-179 (fsh v3) — link?"
Uses title similarity, tag overlap, and command patterns.

### Intent Health Score
Each intent gets a health score based on:
- Gate completion rate
- Time since last activity
- Dependency status (are prerequisites done?)
- Stall detection (in-progress for >14 days with no commits)
core intent health          — show health scores for all active intents
core intent health --stale  — show stalled intents

### Smart Intent Creation
core intent new --smart     — forest suggests title, tags, gates
                               based on current work context
"You have been working on fsh for 3 sessions — intent for fsh v4?"

### Dependency Graph v2
Currently: basic dependency tracking.
v2: visual dependency graph with critical path analysis.
"If INT-194 blocks INT-195, critical path is 194 → 195 → v14 gate"

### Intent Autobiography v2
Currently: story command gives basic narrative.
v2: richer narrative with session highlights, key decisions,
turning points, and what each intent enabled.
"INT-156 (v13 Autonomy) was the convergence of 18 months of building.
It unlocked the partner system and changed what the forest could become."

### Completion Prediction
Based on gate completion rate and historical velocity:
"INT-194 at current pace: complete in ~3 sessions"
Helps with planning and setting expectations.

## Commands
core intent health                — intent health scores
core intent health --stale        — stalled intents
core intent new --smart           — context-aware intent creation
core intent deps --critical-path  — critical path analysis
core intent story --rich INT-NNN  — rich autobiography entry
core intent predict INT-NNN       — completion prediction

## Gate Check
⬜ Auto-linking on intent creation
⬜ Intent health scoring (gate completion + stall detection)
⬜ Smart intent creation with context suggestions
⬜ Dependency graph v2 with critical path
⬜ Intent autobiography v2 — richer narrative
⬜ Completion prediction based on velocity

## The Phrase
"150 intents built the forest.
v2 makes the forest understand
what those intents meant
and what they made possible." 🌲

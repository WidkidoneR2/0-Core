---
id: 197
date: 2026-04-05
type: planned
title: "Intelligence Layer v2 — The Forest Learns From Itself"
status: in-progress
tags: [intelligence, prediction, feedback, learning, v2, contextd]
---
## The Problem
The intelligence layer (v9-v13) was built correctly.
But four critical gaps remain:

1. Prediction feedback loop not closed — outcomes not recorded back
2. No memory decay — state.db grows indefinitely
3. Signal detection is reactive — contextd only detects after the fact
4. No cross-session learning — each session starts cold

## What v2 Fixes

### Prediction Feedback Loop (Critical)
Every prediction made must be verified.
core predict verify <id> — mark prediction correct or incorrect
Auto-verification: if predicted command runs within 10 min, mark correct.
Accuracy score updates in real time. Jarvis score reflects true accuracy.

### Memory Decay
state.db currently grows forever.
v2: entries older than 90 days decay unless marked important.
forest_events pruned to last 30 days.
prediction_outcomes archived after 90 days.
shell_history pruned to last 10,000 entries.
Decay is gradual — important patterns survive.

### Proactive Signal Detection
contextd currently detects failure-loop AFTER 4 failures.
v2: detect the FIRST sign of a pattern forming.
"2 identical errors in 5 minutes — watching for failure loop"
Intervention earlier = less wasted work.

### Cross-Session Pattern Learning
Currently: each session is analyzed independently.
v2: patterns are aggregated across sessions.
"You always run d after fg commit — across 47 sessions"
The longer the forest runs, the smarter it gets.

### Causality Engine Upgrade
core why currently gives basic causality.
v2: deeper chain — "this failure likely caused by that deploy 3 hours ago"
Uses forest_events timeline to trace causes across time.

## Commands
core predict verify <id>    — close the feedback loop
core predict accuracy       — current accuracy across all predictions
core memory decay           — show what would be pruned
core memory decay --apply   — apply decay (with confirmation)
core context cross-session  — patterns spanning multiple sessions

## Gate Check
⬜ Prediction feedback loop closed — verify command live
⬜ Auto-verification for predictions (10 min window)
⬜ Prediction accuracy > 70% demonstrated
⬜ Memory decay policy defined and running
⬜ state.db size bounded — growth trend stable
⬜ Proactive signal detection (warn before 4 failures)
⬜ Cross-session pattern aggregation
⬜ Causality engine v2 — timeline-based chain analysis

## The Phrase
"Intelligence without feedback is guessing.
Intelligence with feedback is learning.
v2 closes the loop." 🌲

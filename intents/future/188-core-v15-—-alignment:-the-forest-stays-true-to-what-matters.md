---
id: 188
date: 2026-04-03
type: arch
title: "Core v15 — Alignment: The Forest Stays True to What Matters"
status: planned
tags: [alignment, values, drift, v15, philosophy, consistency]
priority: medium
depends_on: [178, 187]
---

## The Problem
v14 gives the forest opinions. v15 ensures those opinions stay consistent.
Right now the system tracks intents, outcomes, patterns.
It does not track what it optimizes for.
Without declared values, the system can be clever and directionless.

## What Alignment Means
Not ethics. Not philosophy lectures.
Behavioral consistency with your own declared principles over time.

You have already declared:
  "manual control over automation"
  "understanding over convenience"
  "nothing runs without explicit human authorization"
  "recovery over perfection"

v15 makes these machine-readable and checkable.

## Value System
core values list
core values define "focus > speed"
core values define "understanding > convenience"
core values define "ship consistently"

Stored in state.db values table with:
  priority weight (1-10)
  domain scope (shell / intents / tools / all)
  declared_at timestamp

## Alignment Checking
core align check INT-162

Output:
  Alignment Score: 68%
  Conflicts:
    - violates "focus > speed" — 5 intents active simultaneously
    - aligns with "ship consistently" — 14 commits this week
  Recommendation: reduce active intents before proceeding

## Drift Detection (the killer feature)
core align drift

Output:
  Detected Drift:
    - Last 14 sessions prioritized speed over understanding
    - Value violation trend: +32% over 30 days
  Interpretation: behavior diverging from stated philosophy
  Suggestion: review active intents against declared values

## Behavioral Grounding Rules (from external review)
ALWAYS keep observations behavioral, never personal:
  GOOD: "you switch tasks frequently after errors"
  BAD:  "you struggle with focus"
Stay observable. Not interpretive. Not psychological.

## Disagreement Threshold
v14 honest disagreement + v15 values = grounded pushback:
  Not "I think this is wrong"
  But "this contradicts your own declared value: focus > speed"
That is much stronger and far less arbitrary.

core partner config disagreement-threshold
  low    -> frequent pushback
  medium -> only strong signals  
  high   -> rare, high-confidence only

## Roadmap Simulation
core align simulate
  Path A (current): stability focus -> slower growth
    Projected 30-day: +18% velocity
  Path B (proposed): architecture cleanup -> faster long-term
    Projected 30-day: +34% velocity
  Alignment with values: Path B scores higher on "understanding > convenience"

## Commands
core values list/define/remove
core align check <intent-id>
core align drift
core align simulate
core align report

## Gate Check
⬜ core values list/define/remove live
⬜ Values stored in state.db with weight and scope
⬜ core align check — score intent against declared values
⬜ core align drift — detect behavioral drift over time
⬜ Observations strictly behavioral (no personal interpretation)
⬜ Disagreement grounded in declared values, not opinion
⬜ core align simulate — roadmap path comparison
⬜ Integrated with v14 partner disagreement system

## The Phrase
"The forest that knows its values
can detect when it betrays them.
Alignment is not a constraint.
It is the compass that makes
every decision navigable." 🌲

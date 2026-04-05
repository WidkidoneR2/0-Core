---
id: 188
date: 2026-04-04
type: planned
title: "Core v15 — Alignment: The Forest Stays True to What Matters"
status: planned
tags: [alignment, values, drift, v15, philosophy, consistency, behavioral]
---
| Version | Capability | Meaning |
|---------|-----------|---------|
| v13 | Autonomy   | the forest acts within mandate |
| v14 | Partnership | the forest thinks alongside you |
| **v15** | **Alignment** | **the forest stays true to what matters** |
v14 gives the forest opinions.
v15 ensures those opinions stay consistent with your declared values.
Without declared values, the system can be clever and directionless.
A partner without principles is not a partner — it is a mirror.
Not ethics. Not philosophy lectures. Not psychological interpretation.
Behavioral consistency with your own declared principles over time.
You have already declared your principles:
  "manual control over automation"
  "understanding over convenience"
  "nothing runs without explicit human authorization"
  "recovery over perfection"
v15 makes these machine-readable, checkable, and enforceable.
The forest earns the right to say: "this contradicts your own values."
All observations must be strictly behavioral. Never personal.
  ✅ GOOD: "you switched tasks 4 times in the last hour after errors"
  ✅ GOOD: "5 intents active simultaneously — violates focus > speed"
  ❌ BAD:  "you struggle with focus"
  ❌ BAD:  "you seem distracted"
Stay observable. Stay factual. Never interpret personality or character.
This is the line between a useful tool and an annoying one.
Values are stored in state.db with priority weight and scope.
```bash
core values list
core values define "focus > speed"
core values define "understanding > convenience" --weight 9
core values define "ship consistently"  --weight 7 --scope intents
core values remove <id>
core values weight <id> <1-10>
```
Seed values on first run:
  "manual control over automation"      weight 10  scope all
  "understanding over convenience"      weight 9   scope all
  "recovery over perfection"            weight 8   scope all
  "focus > speed"                       weight 8   scope intents
  "ship consistently"                   weight 7   scope commits
```bash
core align check INT-162
```
Output:
  Alignment Score: 68%
  Aligned:
    ✅ "ship consistently" — 14 commits this week
    ✅ "recovery over perfection" — 2 checkpoints before major changes
  Conflicts:
    ⚠️  "focus > speed" — 5 intents active simultaneously
  Recommendation: reduce active intents before proceeding
```bash
core align drift
```
Output:
  Behavioral Drift Report (last 30 days):
    ⚠️  "focus > speed" violated 8 times — 3 more than previous period
    ✅ "ship consistently" maintained — 180+ commits, steady cadence
    ⚠️  Deploy-without-health-check pattern rising (+32%)
  Trend: moderate drift in focus discipline
  Observation: task-switching increases after failed commands
This is not judgment. It is pattern recognition from your own data.
v14 honest disagreement + v15 values = grounded pushback.
Without v15:
  "I think starting INT-193 now is risky"  ← opinion, easy to dismiss
With v15:
  "Starting INT-193 now contradicts your declared value 'focus > speed'
   — 4 intents already in-progress. Your own principle says focus first."
  ← grounded in your words, much harder to dismiss
```bash
core partner config disagreement-threshold
  low    -> frequent pushback (every potential conflict)
  medium -> only strong signals (score < 60%)
  high   -> rare, high-confidence only (score < 40%)
```
Every 7 days, the forest generates a conscience check.
Short. Honest. Behavioral.
  "Week of April 5–11:
   3 value alignments detected.
   1 drift signal: focus > speed violated 3 times.
   Strongest alignment: ship consistently (42 commits).
   Weakest: focus discipline (5 intents in flight twice)."
```bash
core align report
core align report --week 2
core align history
```
```bash
core align simulate
```
Compare two paths against declared values.
Requires 30+ days of alignment data before results are meaningful.
Do not block v15 completion on this feature — mark as stretch.
- v14 partner disagree — now grounded in declared values not opinion
- faelight-contextd — alignment violations become insights
- core strategy jarvis — alignment score contributes to Jarvis readiness
- forest journal (INT-195) — weekly report written to journal
INT-178 Core v14 — partnership system operational
INT-167 Prediction Accuracy — behavioral data foundation
30+ days of forest_events — enough behavioral data for drift detection
Phase 1 — Value System (define/list/remove/weight)
Phase 2 — Alignment Checking (check intent against values)
Phase 3 — Drift Detection (behavioral patterns over time)
Phase 4 — Disagreement Grounding (v14 integration)
Phase 5 — Weekly Report (automated conscience check)
Phase 6 — Roadmap Simulation (stretch — after 30 days data)
⬜ Values table in state.db with weight, scope, declared_at
⬜ core values list/define/remove/weight live
⬜ 5 seed values loaded from declared principles
⬜ core align check — score intent against declared values
⬜ core align drift — behavioral drift detection over 30 days
⬜ Observations strictly behavioral — no personal interpretation
⬜ Disagreement grounded in declared values not opinion
⬜ core align report — weekly behavioral conscience check
⬜ Integrated with v14 partner disagree system
⬜ Alignment factor added to Jarvis score
⬜ STRETCH: core align simulate — path comparison against values
**"The forest that knows its values
can detect when it betrays them.
Alignment is not a constraint —
it is the compass that makes
every decision navigable.**
*A partner without principles is clever.
A partner with principles is trustworthy.
v15 is not about being good.
It is about being consistent
with what you already said matters."* 🌲

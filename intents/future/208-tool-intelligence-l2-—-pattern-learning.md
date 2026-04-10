---
id: 208
date: 2026-04-08
type: planned
title: "Tool Intelligence L2 — Pattern Learning"
status: in-progress
tags: [tools, intelligence, patterns, learning, state.db, friday, weight-engine, v2]
---

## What Level 1 Cannot Do

Level 1 tools are context-aware.
They read state before acting and shape output accordingly.
But they do not learn.

Every run of faelight-git is the same as the last.
Every deploy is evaluated fresh with no memory of the previous thousand.
Every doctor run produces a health score that disappears.

Level 2 tools remember.
They contribute to the pattern weight engine.
They build the data Friday will reason from.

## Level 2: Pattern Learning

Each tool records structured outcomes to state.db.
Each outcome feeds the pattern weight engine (INT-205).
Each pattern becomes something Friday can reference.

Not just "what happened" — but "what happened, in what context, with what outcome."

## Tool Upgrades

### faelight-update v4.2.0
Every update run produces structured learning:
update_pattern:
timestamp:        <when>
packages_updated: 12
risk_level:       MEDIUM
critical_count:   1
duration_ms:      18400
health_before:    100
health_after:     100
outcome:          success
active_intents:   ["INT-188", "INT-194"]
drift_before:     LOW
drift_after:      LOW

Pattern engine learns:
- Updates during active development sessions → higher failure rate?
- Critical updates on certain days → reboot patterns?
- Time between updates → drift prediction accuracy?
- Which packages most often cause post-update health drops?

### faelight-git v3.2.0
Every commit produces structured learning:
commit_pattern:
timestamp:        <when>
intent_id:        "INT-188"
files_changed:    3
lines_added:      47
lines_removed:    12
commit_velocity:  8.2  // commits per hour this session
health_at_commit: 100
alignment_at_commit: 100
session_depth:    23   // which commit of the session
outcome:          pushed | local-only | failed

Pattern engine learns:
- High velocity sessions → more likely to need rollback?
- Commits at certain session depth → higher quality?
- Files changed vs gates completed correlation?
- Time between commits and deploy → success predictor?

### faelight-shell (fsh) v0.8.0
Every session produces structured learning:
session_pattern:
date:             <when>
duration_minutes: 187
commands_run:     342
deploys:          8
commits:          23
health_changes:   1  // times health dropped and recovered
failed_commands:  4
focus_score:      0.85  // time on single intent vs switching
flow_indicators:  ["long-unbroken-build", "rapid-commits"]
exit_health:      100

Pattern engine learns:
- Session duration → productivity indicators?
- Failed command rate → debugging session signature?
- Focus score → correlates with intent completion speed?
- What time of day produces best sessions?

### core doctor v2.2.0
Every health check produces structured learning:
health_pattern:
timestamp:        <when>
health_pct:       100
integrity_pct:    100
active_intents:   5
checks_passed:    22
checks_warned:    1
checks_failed:    0
trigger:          manual | post-deploy | scheduled
time_since_last:  3600  // seconds since last check
pattern_weight:   0.0   // from weight engine (once v17 ships)

Pattern engine learns:
- Which intents correlate with health drops?
- How long between deploys and health checks?
- What warning patterns precede failures?
- Optimal check frequency based on work patterns?

## The friday_observations Connection

All Level 2 patterns feed directly into friday_observations.
Friday does not need to be built to start learning.
Every structured outcome from Level 2 tools is a lesson Friday will inherit.

When Friday wakes up in Phase 0 (INT-203):
- It already has thousands of deployment patterns
- It already has commit velocity data
- It already has session flow signatures
- It already has health trajectory history

Friday's first words will be informed by everything Level 2 built.

## The Pattern Weight Engine Connection

Every Level 2 outcome feeds INT-205 (Pattern Weight Engine).
When v17 ships, it will have real data to weight.
Not synthetic data. Not test data. Your actual work history.

The weight engine and Level 2 tools are designed together.
Level 2 produces the data. The weight engine gives it meaning.

## Gate Check
✅ faelight-update v4.2.0 — structured update_pattern logged to state.db (2026-04-09)
⬜ faelight-update — pattern feeds INT-205 pattern_weights table
✅ faelight-git v3.2.0 — structured commit_pattern logged to state.db (2026-04-09)
⬜ faelight-git — commit velocity and session depth tracked
✅ fsh v0.8.0 — structured session_pattern logged on exit (2026-04-09)
⬜ fsh — focus_score computed per session
⬜ core doctor v2.2.0 — structured health_pattern logged per run
⬜ core doctor — trigger type recorded (manual | post-deploy | scheduled)
⬜ friday_observations populated from all four tools
⬜ pattern_weights table receiving data from tool outcomes
⬜ At least 30 days of structured data across all four tools
⬜ core engines signals shows tool signals flowing
⬜ deploy all four tools and d passes 100%

## The Phrase

"A tool that does not remember what it did
cannot improve at what it does.

Level 2 is not intelligence either.
It is memory with structure.

The difference between data and wisdom
is not the amount.
It is the shape." 🌲

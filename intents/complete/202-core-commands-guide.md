---
id: 202
date: 2026-04-08
type: planned
title: "Core Commands Guide"
status: complete
tags: [documentation, core, commands, guide, friday, reference]
---
A complete, accurate, human-readable reference for every `core` command.
Not generated. Not scraped. Written with intent.
Every domain documented: what it does, when to use it, example output.
This serves three purposes:
1. You — when you forget what a command does after not using it for a month
2. Friday — when it needs to understand what actions are available
3. External observers — Linus, Graydon, anyone who looks at this system
Every domain in core 3.0.0:
- `core predict` — 9 commands, session patterns, health trajectory, intent velocity
- `core react` — 6 rules, health advisory, security aging, checkpoint staleness
- `core strategy` — horizon/sequence/coherence/jarvis/trust
- `core goals` — generate, accept, reject, prioritize
- `core autonomy` — goal evaluation, delegation simulation
- `core partner` — propose, discuss, disagree, consult, reflect, pattern, growth, roadmap
- `core values` — list, define, remove, weight
- `core align` — check, drift, report
- `core delegate` — simulate, contracts, history, accuracy, activate, suspend
- `core engines` — status, sync, signals, check, upgrade-log
- `core doctor` — run, quick, history
- `core integrity` — run, apply, heal
- `core intent` — new, start, complete, list, show, search, health, burndown, velocity
- `core decision` — record, outcome, list, show, hindsight, advise, lessons
- `core checkpoint` — create, list, restore, diff
- `core events` — list, since, filter, status, watch
- `core security` — audit, harden, debt
- `core git` — status, log, diff
- `core db` — query, vacuum, stats
- `core profile` — list, switch, create
Each command documented as:
core align check
Purpose: Score an action or decision against your declared values.
When to use: Before starting a new intent, before a major change,
when you want to verify your next action is consistent with your principles.
Usage:
core align check "starting INT-189 now"
core align check "deploying to production"
Output:
Alignment Score: 100%
✅ Aligned:
· "focus > speed" — 0 intents active (within range)
· "ship consistently" — 153 commits this week
Notes: Observations are strictly behavioral. Never personal.
Score above 80% = proceed. Below 60% = consider pausing.
Stored as: `~/0-core/docs/core-commands.md`
Accessible via: `core docs commands` (new command)
Also linked from README.
✅ All 50+ core domains documented with purpose, usage, output
✅ core docs commands — opens the guide from the terminal
✅ docs/core-commands.md created and accurate
✅ Living document — updated with every new domain
✅ Friday can reference this guide for action discovery
✅ core docs list shows guide alongside all forest docs
"A system that cannot explain itself
cannot be trusted.
The guide is not for the maintainer.
The guide is for the forest —
so Friday knows what tools it has,
and you know what the forest can do." 🌲

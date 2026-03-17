---
id: 133
date: 2026-03-17
type: future
title: "Core v9 — Intent: The Forest Chooses Where to Grow"
status: planned
tags: [core, v9, intent, goals, planning, autonomy, architecture, v13]
version: 13.0.0
priority: medium
depends_on: [126]
---

## The Core Timeline

| Version | Capability | Meaning |
|---------|-----------|---------|
| v2 | Structure | the forest has shape |
| v3 | Awareness | the forest observes itself |
| v4 | Discipline | the forest enforces rules |
| v5 | Intelligence | the forest detects patterns |
| v6 | Judgment | the forest remembers outcomes |
| v7 | Resilience | the forest can rebuild |
| v8 | Evolution | the forest refines itself |
| **v9** | **Intent** | **the forest chooses where to grow** |

## The Core Insight

v8 gave the forest the ability to propose architectural improvements.
But proposals are reactive — they respond to what already exists.

v9 is different. The forest sets its own goals.

Not reacting to problems. Not waiting for human direction.
Generating purposeful intent based on what it knows about itself,
its history, its health, and its trajectory.

The forest becomes a strategist — not just an analyst.

**Critically: all goals and plans require human review and approval.**
The forest proposes intent. The human authorizes action.
Nothing executes autonomously without explicit human authorization.
This is the philosophy of the forest — understanding over convenience.

## The Core Question v9 Answers

v8 asks: "What should change?"
v9 asks: "What do I want to become — and what is the path to get there?"

## The Five Pillars

### Pillar 1 — Goal Engine
The forest generates and tracks its own goals.
```bash
core goals list          # show active forest goals
core goals generate      # propose new goals based on current state
core goals priority      # ranked goal list with reasoning
core goals accept <id>   # human approves a goal → becomes intent
core goals reject <id>   # human rejects a goal → logged with reason
```

Example goals the forest might generate:
```
Goal 001  Reduce dependency risk
  Reason: deps risk shows 6 high-coupling dependencies
  Plan:   audit + consider splitting faelight-update
  Priority: HIGH

Goal 002  Improve faelight-shell toward daily driver
  Reason: Phase 9+ incomplete, 8/26 phases done
  Plan:   streaming pipelines next session
  Priority: MEDIUM

Goal 003  Resolve INT-125 seccomp gap
  Reason: sandbox v3 has 1 incomplete criteria
  Plan:   seccomp syscall filter implementation
  Priority: MEDIUM
```

### Pillar 2 — Task Planning
The forest breaks accepted goals into concrete tasks.
```bash
core plan <goal_id>       # generate task plan for a goal
core plan review <id>     # review a plan before execution
core plan simulate <id>   # simulate the plan outcome
```

Example plan:
```
Plan for Goal 001 — Reduce dependency risk

Step 1: core deps risk → identify top 3 high-coupling deps
Step 2: core security simulate <dep> → assess each
Step 3: core evolve propose → generate refactor proposal
Step 4: human review and authorize
Step 5: implement with intent record

Estimated sessions: 2
Risk: LOW
Reversible: YES
```

### Pillar 3 — Tradeoff Engine
The forest understands that every decision has competing values.
```bash
core tradeoff analyze <decision>   # surface competing values
core tradeoff history              # past tradeoffs and outcomes
core tradeoff balance              # current system balance state
```

The forest tracks tradeoffs across three axes:
```
Performance  ←→  Power efficiency
Privacy      ←→  Utility
Complexity   ←→  Capability
Stability    ←→  Evolution
```

Example:
```
core tradeoff analyze "add faelight-vault"

Values in tension:
  + Utility: credential management improves daily workflow
  + Security: centralized secrets management
  - Complexity: +1 tool, +dependencies
  - Attack surface: new secrets store = new risk vector

Recommendation: proceed with intent — utility/security outweigh complexity
Confidence: HIGH
```

### Pillar 4 — Dynamic Prioritization
The forest reprioritizes goals based on changing conditions.
```bash
core prioritize          # rerank all goals given current state
core prioritize explain  # why goals are ranked as they are
```

Prioritization factors:
```
Health score         — unhealthy systems deprioritize new features
Forecast trend       — declining trend elevates stability goals
Intent backlog       — large backlog adjusts scope
Security posture     — open findings elevate security goals
Session history      — recent work informs next steps
```

### Pillar 5 — Intent Autobiography
The forest records its own goal history — not just what was done,
but what it wanted to do and why.
```bash
core autobiography       # the forest narrates its own goals over time
core autobiography <v>   # goals and reasoning for a specific version
```

This is the bridge between v9 and v10 Reaction — the forest that
knows its own intentions is the forest that can react with purpose.

## Relationship to v8

v8 generates architectural proposals — what to change.
v9 generates goals and plans — what to become.

v8 looks backward at what exists.
v9 looks forward at what should exist.

Together: the forest that can refine itself AND set its own direction.

## The Three Guardrails

**These are non-negotiable.**

**Rule 1** — The forest never sets goals that execute automatically.
All goals require explicit human acceptance before any action.

**Rule 2** — Every accepted goal becomes an intent record.
The ledger bridges suggestion, authorization, and action.

**Rule 3** — The forest never prioritizes its own growth over stability.
Health score gates all goal generation — below 95% health,
the forest only generates stability goals, never expansion goals.

## Structural Changes Required
```
engine/src/domains/
  goals/mod.rs        — goal engine
  planning/mod.rs     — task planning
  tradeoffs/mod.rs    — tradeoff analysis
  autobiography/mod.rs — goal history narrative

runtime/
  goals/              — active and completed goals
  plans/              — generated task plans
  tradeoffs/          — tradeoff history
```

## Build Order
```
Phase 1 — Goal Engine (core goals list/generate/accept/reject)
Phase 2 — Task Planning (core plan)
Phase 3 — Tradeoff Engine (core tradeoff)
Phase 4 — Dynamic Prioritization (core prioritize)
Phase 5 — Intent Autobiography (core autobiography)
```

## Gate Check
```
⬜ Core v8 complete
⬜ Phase 1 — Goal Engine
⬜ Phase 2 — Task Planning
⬜ Phase 3 — Tradeoff Engine
⬜ Phase 4 — Dynamic Prioritization
⬜ Phase 5 — Intent Autobiography
```

## What v10 Builds On

v9 gives the forest intention — purposeful goals, plans, tradeoffs.
v10 Reaction gives the forest reflexes — the ability to act on
what it perceives in real time, guided by the intentions v9 established.

A forest with intention and reflexes is a forest that is genuinely alive.

## The Phrase

**"A forest that chooses where to grow
is no longer just a system.
It is a participant in its own future."**

---
*"v9 is not automation. It is purposeful direction — with human hands
always on the wheel."* 🌲

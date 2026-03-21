---
id: 140
date: 2026-03-21
type: future
title: "Core v10 — Reaction: The Forest Responds Without Being Asked"
status: planned
tags: [core, v10, reaction, events, reflexes, automation, v13]
version: 13.0.0
priority: medium
depends_on: [133, 120]
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
| v9 | Intent | the forest chooses where to grow |
| **v10** | **Reaction** | **the forest responds without being asked** |

v10 is the moment the forest gains reflexes.
Not just observing, not just proposing — responding.
Guided by the intentions v9 established, the forest acts
when conditions change, without waiting to be asked.

**Critically: reactions are bounded by human-approved goals.**
The forest only reacts within the scope of accepted v9 goals.
No reaction executes outside that boundary.

## The Core Insight

v9 gives the forest purposeful goals.
v10 gives those goals a nervous system.

When health drops — the forest doesn't wait for `d`.
When churn spikes — the forest doesn't wait for `gchurn`.
When a goal becomes achievable — the forest doesn't wait for `core goals list`.

It surfaces the signal. It proposes the response. The human decides.

## The Relationship to faelight-shell Phase 17

v10 and Phase 17 (event system) are the same idea at two layers:
```
faelight-shell Phase 17  — shell-level events
  on file_change run build
  on log_error notify

Core v10                  — forest-level reactions
  on health < 95     → surface stability goals from v9
  on gchurn > 50     → flag hotspot, propose evolution
  on forecast -2.0   → pre-emptive advise before drop hits
  on intent.complete → audit affected tools, emit forest.grew
```

Both feed the same reaction engine. Phase 17 handles local shell
events. v10 handles forest-wide state changes.

## The Five Pillars

### Pillar 1 — Reaction Rules
Human-defined rules that bind forest state to responses.
```bash
core react list              # show all active reaction rules
core react add               # define a new reaction rule
core react enable/disable    # toggle rules
core react history           # log of all triggered reactions
```

Example rules:
```
on health < 95:
  action: surface stability goals
  message: "Health advisory — stability goals activated"

on gchurn(file) > 50 and coupling > 0.4:
  action: flag for evolution proposal
  message: "Hotspot detected: {file} — consider refactor"

on forecast.trend < -1.5:
  action: pre-emptive advise
  message: "Forecast declining — review before it drops"

on intent.complete:
  action: audit affected tools
  emit: forest.grew
```

### Pillar 2 — Event Bus
All forest events flow through a unified bus.
```bash
core events stream           # live event stream
core events subscribe <type> # watch specific event types
core events replay <time>    # replay events from a point in time
```

Event sources:
- health checks (doctor domain)
- git commits (faelight-git events)
- intent transitions (intent domain)
- security findings (security domain)
- forecast changes (doctor domain)
- shell commands (faelight-shell events)

### Pillar 3 — Conditional Intelligence
Reactions are not simple if/then — they use v9 goal context.
```
if health < 95 AND active_goal.type == "stability":
  escalate urgency
  surface relevant plan steps

if gchurn(file) > 50 AND file in active_goal.scope:
  link finding to goal
  propose next plan step
```

The forest doesn't just react — it reacts with purpose.

### Pillar 4 — Reaction Boundaries
The guardrails that keep v10 safe.
```bash
core react bounds            # show current reaction boundaries
core react audit             # audit all reactions against goals
```

Boundaries:
- Reactions only within accepted v9 goal scope
- No filesystem writes from reactions
- No network calls from reactions
- No reactions while health < 80% (stability gate)
- All reactions logged to intent ledger

### Pillar 5 — Reaction Narrative
The forest explains why it reacted.
```bash
core react explain <id>      # why did this reaction fire?
core react story             # today's reaction narrative
```

Example:
```
Reaction 042 fired at 14:23
  Trigger: forecast.trend dropped to -1.8
  Goal context: Goal 003 — maintain 95%+ health
  Action: surfaced advise
  Human response: ran d, found schema issue, fixed
  Outcome: forecast recovered to +0.4
```

## The Three Guardrails

**Rule 1** — Reactions never execute outside accepted v9 goal scope.
The goal boundary is the reaction boundary.

**Rule 2** — Reactions propose, never act.
The forest surfaces signals and suggestions. Humans execute.

**Rule 3** — Stability gates all reactions.
Health < 80%: only stability reactions fire.
Health < 95%: expansion reactions suspended.
Health ≥ 95%: full reaction engine active.

## Relationship to Other Intents
```
INT-133 Core v9    — goal engine that scopes reactions
INT-120 Phase 17   — shell event system (sister layer)
INT-126 Core v8    — evolution proposals that reactions can surface
INT-135 Shell v11  — shell personality that voices reactions
```

## Structural Changes Required
```
engine/src/domains/
  reaction/
    mod.rs          — reaction engine
    rules.rs        — rule definitions and evaluation
    bus.rs          — event bus integration
    bounds.rs       — boundary enforcement
    narrative.rs    — reaction story generation

runtime/
  reactions/        — active rules
  reaction-log/     — triggered reaction history
```

## Build Order
```
Phase 1 — Event Bus (unified event stream)
Phase 2 — Reaction Rules (define + evaluate)
Phase 3 — Conditional Intelligence (goal-scoped reactions)
Phase 4 — Reaction Boundaries (safety enforcement)
Phase 5 — Reaction Narrative (explain + story)
```

## Gate Check
```
⬜ Core v9 complete — all 5 phases done
⬜ faelight-shell Phase 17 complete — event system ready
⬜ Phase 1 — Event Bus
⬜ Phase 2 — Reaction Rules
⬜ Phase 3 — Conditional Intelligence
⬜ Phase 4 — Reaction Boundaries
⬜ Phase 5 — Reaction Narrative
```

## The Phrase

**"A forest with intention knows where it wants to go.
A forest with reflexes knows when to move.
Together: a forest that is genuinely alive."**

---
*"v10 is not automation. It is guided instinct — always within
the boundaries the human authorized."* 🌲

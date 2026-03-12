---
id: 116
date: 2026-03-08
type: future
title: "Core v6 — The Judgment Layer"
status: in-progress
tags: [core, v6, judgment, decisions, outcomes, heuristics, advise, simulate, rust, architecture]
---
## Vision

**v2 gave structure. v3 gave awareness. v4 gave discipline. v5 gave intelligence. v6 gives judgment — the forest that remembers what decisions led to outcomes.**

v5 can see patterns. v6 helps you decide better when the moment arrives.

The progression:
```
v2  Structure    — the forest has shape
v3  Awareness   — the forest sees itself
v4  Discipline  — the forest holds itself accountable
v5  Intelligence — the forest understands patterns
v6  Judgment    — the forest remembers what worked
```

Right now the system tracks events, causes, patterns, and forecasts.
It does not yet track **decisions and their outcomes** as first-class entities.
That is the missing layer.

**The core insight:** Intelligence without memory of outcomes is forecasting.
Intelligence with memory of outcomes is judgment.

---

## The Five Pillars

### Pillar 1 — Decision Ledger
Decisions become first-class entities in the forest.
```
core decide "upgrade rust toolchain"
core decide "refactor compositor input pipeline"
core decide "large dependency update before release"
```

Each decision is stored as a structured object:
- `decision_id` — DEC-001, DEC-002...
- `timestamp` — when the decision was made
- `context_snapshot` — health, active intents, git churn, recent events
- `related_intent` — optional INT-XXX reference
- `confidence` — low / medium / high (operator-set)
- `expected_outcome` — free text
- `actual_outcome` — set later via `core outcome`

Storage: `runtime/decisions/` directory — one TOML per decision.
Also indexed in `state.db` for correlation queries.

### Pillar 2 — Outcome Tracking
Close the loop on every decision.
```
core outcome DEC-014 success
core outcome DEC-014 failure
core outcome DEC-014 partial "improved performance but broke notifications"
```

Outcome types: `success`, `partial`, `failure`, `unknown`

Over time the system builds correlation:
- decision conditions → outcome
- "architectural decisions during health < 90% correlated with partial/failure 4/5 times"
- "decisions with checkpoints created beforehand → 90% success rate"

### Pillar 3 — Operator Judgment Assist
The payoff feature. Reads current state + decision history and advises.
```
core advise
core advise --decision "upgrade dependencies"
core advise --decision "large refactor"
```

Example output:
```
🧭 Judgment Advisory
  Context:
    Health: 91%  Git churn: elevated  Active intents: 4

  Historical patterns (8 observations):
    Large changes during elevated churn → regression 3/4 times
    Decisions without checkpoint → failure 2/3 times

  Suggestion:
    Consider cpc before proceeding.
    Wait for health ≥ 95% — current trend reaches it in ~2 runs.
```

The system never acts. It only informs. Philosophy intact.

### Pillar 4 — Heuristics Engine
Distilled lessons from the decision ledger.

Raw events → aggregates → patterns → **heuristics**

Storage: `runtime/heuristics/heuristics.toml`

Each heuristic:
```toml
[[heuristic]]
id = "H-001"
description = "Large refactors during elevated git churn increase regression risk"
confidence = 0.82
observations = 9
domain = "git"
last_updated = "2026-03-08"
```

Confidence-gated: never surfaces with fewer than 5 observations.
Built incrementally — new outcomes trigger re-evaluation.
```
core heuristics          # list all learned heuristics
core heuristics --domain git
core lessons             # human-readable summary of what the forest has learned
```

### Pillar 5 — Scenario Simulation (Extended)
Now that the system has forecasts, patterns, and decisions, it can simulate.
```
core simulate "large dependency upgrade"
core simulate "major refactor before release"
```

Output:
```
🔮 Simulation: "large dependency upgrade"
  Risk signals:
    · Security scan lag (last scan 3d ago)
    · Git churn elevated (15 files changed today)
    · 3 active intents — context switching risk

  Historical match: 3 similar decisions
    2 partial, 1 success
    Common factor in success: checkpoint created first

  Estimated risk: medium
  Recommendation: checkpoint + security scan before proceeding
```

---

## New Command Set
```
core decide "<description>"    # record a decision with context snapshot
core outcome <id> <result>     # record outcome of a decision
core advise                    # judgment advisory for current state
core advise --decision "<x>"   # advisory for a specific planned decision
core heuristics                # list learned heuristics
core heuristics --domain <x>   # domain-filtered heuristics
core lessons                   # human-readable wisdom summary
core simulate "<scenario>"     # risk simulation for a planned action
core decisions                 # list recent decisions
core decisions --open          # decisions without outcomes yet
```

---

## What Stays The Same

v6 does not replace v5. It extends it.

- Health forecasting: still available, now informs advise
- Causality engine: still available, feeds heuristic building
- Pattern recognition: still available, provides baseline for simulation
- Event ledger: still the source of truth — v6 only reads and appends, never rewrites

**The philosophy stays the same: manual control over automation.**
v6 makes you better informed. It never acts without you.

---

## Build Order

### Phase 1 — Decision Ledger Foundation
`core decide`, `core outcome`, `core decisions`
- `runtime/decisions/` directory structure
- Decision schema with context snapshot
- Outcome recording
- List and show commands

### Phase 2 — Outcome Correlation
Connect decisions to conditions and outcomes.
- Cross-reference decision context with actual outcome
- Build correlation table in state.db
- `core decisions --open` for pending outcomes

### Phase 3 — Judgment Assist
`core advise` — the flagship feature.
- Read current state (health, churn, intents)
- Query decision history for similar conditions
- Surface relevant patterns and heuristics
- Render advisory without prescribing action

### Phase 4 — Heuristics Engine
`core heuristics`, `core lessons`
- Auto-build heuristics from decision+outcome corpus
- Confidence scoring (min 5 observations)
- Human-readable lessons summary

### Phase 5 — Extended Simulation
`core simulate` extended with decision patterns
- Pattern-aware risk signals
- Historical decision match
- Checkpoint and scan recommendations

---

## Session Rules
```
1. One phase per session.
2. Every session ends at 95%+ health with a clean commit.
3. Judgment features are read-only — they advise, never act.
4. No phase starts without the previous phase tested and stable.
5. Heuristics require minimum 5 observations before surfacing.
6. core advise never prescribes — it informs and suggests.
```

---

## Gate Check
```
✅ Core v5 complete — event ledger rich with real data
✅ runtime/state.db has 250+ events across 5 domains
✅ Causality engine operational — patterns queryable
✅ Philosophy alignment confirmed — advise only, never act
⬜ Phase 1 — Decision ledger foundation
⬜ Phase 2 — Outcome correlation
⬜ Phase 3 — Judgment assist (core advise)
⬜ Phase 4 — Heuristics engine
⬜ Phase 5 — Extended simulation
```

---

## Stats Context (at time of writing)
```
System:       v10.6.0 — The Judgment Layer
Health:       95% (22 checks)
Commits:      1365
Intents:      80 complete
Event ledger: 260+ events, domains: doctor, git, security, update, compositor
Tools:        50 custom Rust binaries
Decisions:    0 (ledger not yet built)
Heuristics:   0 (requires decision corpus)
```

---

## The Phrase

**"The best advisor isn't the one who knows the most facts.
It's the one who remembers what happened last time
and what conditions surrounded it."**

*"The forest doesn't just see ahead.
It remembers the path that led here — and advises the next step wisely."* 🌲

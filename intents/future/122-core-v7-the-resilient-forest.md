---
id: 122
date: 2026-03-12
type: future
title: "Core v7 — The Resilient Forest"
status: planned
tags: [core, v7, resilience, bootstrap, reproducibility, security, architecture]
version: 11.0.0
priority: medium
---

## Vision

**v2 gave structure. v3 gave awareness. v4 gave discipline.
v5 gave intelligence. v6 gave judgment. v7 gives resilience —
the forest that can rebuild itself from its own memory.**

The progression:
```
v2  Structure    — the forest has shape
v3  Awareness   — the forest sees itself
v4  Discipline  — the forest holds itself accountable
v5  Intelligence — the forest understands patterns
v6  Judgment    — the forest remembers what worked
v7  Resilience  — the forest can rebuild itself
```

## The Core Insight

Right now the forest knows itself deeply.
But if the system fails catastrophically, recovery is manual.

v7 makes the forest self-healing and self-reproducible.
Given the intent ledger, decision history, and registry —
the forest should be able to guide its own reconstruction.

## The Five Pillars (Draft)

### Pillar 1 — Bootstrap Intelligence
```bash
core bootstrap plan      # What would it take to rebuild this forest?
core bootstrap verify    # Is the current state consistent with history?
core bootstrap diff      # What diverged from the canonical state?
```

The forest reads its own intent ledger and registry to
generate a reconstruction plan. Not automation — guidance.

### Pillar 2 — Anomaly Detection
```bash
core anomaly scan        # Detect unexpected system changes
core anomaly history     # What changed without an intent?
core anomaly alert       # Surface changes that lack decision records
```

If a file changes without a corresponding decision or intent,
the forest notices. Not blocking — observing.

### Pillar 3 — Security Intelligence (Extended)
```bash
core security advise     # INT-119 — judgment for security decisions
core security trend      # How has security posture changed over time?
core security simulate   # What would happen if we applied this patch?
```

### Pillar 4 — Dependency Intelligence
```bash
core deps graph          # Visual dependency map of all forest tools
core deps risk           # Which dependencies carry the most risk?
core deps audit          # Cross-reference with decision history
```

The forest understands its own dependency tree and can
reason about the risk of changes.

### Pillar 5 — Forest Narrative (Extended)
Building on Core v6's `core story`, v7 adds:
```bash
core narrative           # Long-form forest history
core narrative --since v10.0.0   # From a version
core narrative --intent 109      # Story of a specific intent
```

The forest becomes a historian of its own evolution.

## What This Unlocks
```
core bootstrap plan      → "here's how to rebuild me"
core anomaly scan        → "something changed without a decision"
core deps risk           → "this dependency is your biggest risk"
core narrative           → "here's the story of how I became what I am"
```


## Structural Considerations Before v7

Organization is a top priority. As the forest grows, structure must
mature alongside it. These decisions should be made deliberately,
not reactively.

### Current Structure — Holding Strong
The Numbered Gravity architecture (00-meta through 04-runtime) is
sound and v7 fits naturally into the existing pattern.
```
00-meta/          identity, VERSION, CHANGELOG, PHILOSOPHY
01-registry/      declarations — tools.toml, sandbox-policies.toml
02-rules/         constraints, no execution
03-interfaces/    dotfiles via GNU Stow
engine/           core orchestrator binary
rust-tools/       53+ custom Rust tools
runtime/          all mutable state — state.db, checkpoints
intents/          architectural decision records
scripts/          deployed binaries (PATH priority)
```

### What v7 Adds — No Structural Change Required
```
engine/src/domains/anomaly/     — Phase 1 anomaly detection
engine/src/domains/bootstrap/   — Phase 2 bootstrap intelligence
runtime/anomalies/              — anomaly records (new subdir)
runtime/dependency-graph/       — dependency snapshots (new subdir)
```
Same domain pattern as v6. No restructuring needed.

### Domain Grouping — Consider Before v7 Grows Large
engine/src/domains/ currently has 20+ domains. As v7 adds more,
consider logical grouping to maintain clarity:
```
engine/src/domains/
├── intelligence/   # audit, anomaly, bootstrap, narrative
├── judgment/       # decisions, simulate, advise
├── system/         # doctor, security, update, checkpoint
├── compositor/     # compositor events (future)
└── ...             # other existing domains
```

This is cosmetic but important — the codebase should be as readable
as the forest it manages.

### faelight-shell Scripts — Future Home
When .fsh scripting language matures (Phase 6), scripts need a home:
```
04-scripts/       # NEW — .fsh forest scripts (v12+ territory)
```
Not urgent now — but the numbered directory slot should be reserved.
Document this decision when it becomes relevant.

### The Organizing Principle Going Forward
Every new file, directory, domain, or tool must answer:
- Where does this live in the numbered gravity structure?
- Does it have a clear, single responsibility?
- Is it documented in the intent ledger?
- Does it follow the existing naming pattern?

**Nothing grows without intention. Nothing is added without a home.**

## Build Order (Tentative)
```
Phase 1 — Anomaly Detection (most immediately useful)
Phase 2 — Bootstrap Intelligence
Phase 3 — Security Intelligence Extended (builds on INT-119)
Phase 4 — Dependency Intelligence
Phase 5 — Forest Narrative Extended
```

## Gate Check
```
⬜ Core v6 complete and stable (✅ done)
⬜ faelight-compositor stable (INT-109 in progress)
⬜ faelight-shell Phase 1 complete (INT-120)
✅ Phase 1 — Anomaly Detection (core anomaly scan/history/alert)
⬜ Phase 2 — Bootstrap Intelligence
⬜ Phase 3 — Security Intelligence Extended
⬜ Phase 4 — Dependency Intelligence
⬜ Phase 5 — Forest Narrative Extended
```

## Stats Context (at time of writing)
```
System:    v10.7.0 — The Forest Remembers
Health:    100% (22 checks)
Commits:   1400
Decisions: 4 recorded (ledger just born)
Intents:   83 complete
Tools:     52 custom Rust binaries
```

## The Phrase

**"A resilient forest doesn't fear the storm.
It knows how to grow back."**

*"The forest that remembers its past
can reconstruct its future."* 🌲

## Additional Pillars (Added 2026-03-15)

### Pillar 6 — Snapshot Narrative
The forest writes its own autobiography at a point in time.
```bash
core snapshot narrative          # human-readable markdown
core snapshot narrative --json   # machine-readable reconstruction seed
core snapshot narrative --save   # writes both to runtime/snapshots/
```

The snapshot captures:
- version and identity
- tool inventory with versions and scores
- active policies
- recent decisions and outcomes
- dependency graph
- system health and forecast
- condensed story of key milestones

**Two voices, same data:**
- Human version — markdown, readable without tooling
- Machine version — JSON, feeds `core bootstrap plan`

The narrative becomes the seed of reconstruction.
The bootstrap reads the JSON and guides rebuilding.
The human reads the markdown and understands why.

### Pillar 7 — Deterministic Rebuild
```bash
core doctor rebuild   # reconstruct environment from first principles
```

Reads:
- 01-registry/tools.toml     → what should exist
- intents/complete/          → why decisions were made
- 03-interfaces/stow/        → what the environment looks like
- schema/                    → what is valid
- runtime/events/            → what happened

Produces a step-by-step reconstruction plan.
Not automated — guided. Every step traceable to an intent or decision.

This is what makes Faelight Forest different from NixOS:
NixOS can reproduce state.
Faelight can reproduce state AND reasoning.

## Updated Gate Check

- ✅ Core v6 complete and stable
- ✅ Phase 1 — Anomaly Detection (core anomaly scan/history/alert)
- ⬜ Phase 2 — Bootstrap Intelligence
- ⬜ Phase 3 — Security Intelligence Extended
- ⬜ Phase 4 — Dependency Intelligence
- ⬜ Phase 5 — Forest Narrative Extended
- ⬜ Phase 6 — Snapshot Narrative (core snapshot narrative)
- ⬜ Phase 7 — Deterministic Rebuild (core doctor rebuild)

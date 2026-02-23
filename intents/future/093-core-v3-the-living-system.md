---
id: 093
date: 2026-02-22
type: future
title: "Core v3 — The Living System"
status: planned
tags: [v11, architecture, causality, event-driven, self-aware]
version: 11.0.0
---

## Vision

**v2 gave 0-Core control. v3 gives it memory and foresight.**

Core v2 solved structure — one binary, 15 domains, capability model, everything
organized and intentional. v3 solves a different class of problem entirely:
the system becomes aware of itself over time.

Not automation for its own sake. Not removing human control.
The forest watches, remembers, and advises. You still decide everything.
It just knows more than it did before.

---

## The Three Pillars

### Pillar 1 — Causality Engine
**"Why is the system in this state?"**

Every change to the system is traced to its cause. Not just git commits —
configuration changes, package updates, service state changes, health
fluctuations. The system builds a causal chain.
```
core why                    # Why is the system in its current state?
core why health 95          # Why is health 95% right now?
core why drift              # What caused this config drift?
core trace last-change      # Full causal trace of the last change
```

Implementation:
- Every domain operation writes a structured event to runtime/events/
- Events carry: timestamp, domain, action, before/after state, actor (command)
- Causality graph built from event stream
- `core why` traverses the graph backward from current state

### Pillar 2 — Simulation Engine
**"What will happen if I do this?"**

Before any significant operation, core can simulate the outcome and show
you exactly what would change — without touching anything.
```
core simulate update             # What would a system update change?
core simulate link deploy        # What symlinks would be created?
core simulate security scan      # Preview findings before running
core simulate doctor             # Predict health score after changes
```

Implementation:
- Each domain implements a `simulate()` trait alongside `execute()`
- Simulation runs in a read-only sandbox (no writes to runtime/)
- Output shows diff: current state → predicted state
- User confirms or cancels before real execution

### Pillar 3 — Event Bus
**"The forest reacts to itself."**

Right now everything is pull-based — you run doctor, it checks.
v3 introduces a push-based event system. The system watches itself
and surfaces changes as they happen.
```
core events watch               # Live event stream
core events since 1h            # Events in the last hour
core events filter domain=git   # Events by domain
```

Events emitted by:
- inotify watches on key config files → config drift detected
- systemd journal tail → service state changes
- git fsmonitor → repository changes
- health cache writes → health fluctuations

The bar, palette, and prompt react to events without polling.
CPU usage drops. Latency drops. Everything is live.

---

## Supporting Features

### Plugin Boundary
```
core plugin add <name>          # Register an external tool
core plugin list                # Installed plugins
core plugin remove <name>       # Deregister
```

The individual GitHub repos (faelight-fm, faelight-git, faelight-update,
faelight-palette) become first-class plugins. They declare their capabilities,
register with core, and participate in the event bus. The ecosystem grows
without modifying core itself.

### Intent-Aware Operations
Every core operation is linked to an active intent if one exists.
```
core intent context             # What intent is currently active?
core doctor run --intent 093    # Run health check in context of intent
```
Operations taken while an intent is active are recorded in the intent's
history automatically.

### Health Forecasting
```
core doctor forecast            # Predict health 24h from now
core doctor trend               # Health trend over last 30 days
```
Based on drift history, package update cadence, and event patterns.

---

## What v3 Is NOT

- **Not autonomous** — the system never acts without human confirmation
- **Not AI** — pure deterministic Rust, no models, no inference
- **Not Skynet** — the forest watches and advises, you decide everything
- **Not scope creep** — each pillar is a discrete, shippable unit

The philosophy stays the same: **manual control over automation**.
v3 makes you more informed, not less in control.

---

## Migration Path from v2

v3 is fully backward compatible. Every v2 command works unchanged.
New capabilities are additive:
```
Phase 1 — Event Bus foundation (runtime/events/, inotify watchers)
Phase 2 — Causality Engine (event graph, core why)
Phase 3 — Simulation Engine (simulate() trait, per-domain)
Phase 4 — Plugin Boundary (core plugin, external tool registration)
Phase 5 — Health Forecasting (trend analysis, drift prediction)
Phase 6 — Intent Integration (auto-context, operation linking)
```

---

## Timing

⚠️ DO NOT START BEFORE:
1. ✅ v10.1.0 released (complete)
2. ✅ Intent 092 fully closed (complete)
3. ⬜ Individual repos branched (faelight-fm, faelight-git, etc.)
4. ⬜ faelight-browser WIP resolved
5. ⬜ faelight-term WIP resolved or scoped

Start condition: individual repos complete + v11 planning session.

---

## The Phrase

**"v2 gave you control. v3 gives the forest memory and foresight."**


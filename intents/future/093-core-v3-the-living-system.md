---
id: 093
date: 2026-02-22
updated: 2026-02-27
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

## Build Order (revised 2026-02-27)

The original phase order was logically correct but wrong for safe incremental
delivery. Reordered from least risk to most — each phase fully usable before
the next begins.

### Phase 1 — Event Ledger ← START HERE
**Additive only. Zero risk. No daemon.**

Every domain operation appends a structured JSONL event to runtime/events/.
No watchers, no async, no daemon. Pure write path.
```
runtime/events/
  2026-02-27.jsonl
  2026-02-28.jsonl
  ...
```

Event schema:
```json
{
  "ts": 1740614400,
  "domain": "doctor",
  "action": "run",
  "actor": "dot-doctor",
  "result": "ok",
  "before": { "health": 95 },
  "after":  { "health": 100 }
}
```

Commands unlocked:
```
core events list              # all events today
core events since 1h          # last hour
core events filter domain=git # by domain
```

Session scope: 1 session. Implement EventWriter in runtime/,
wire into 3-4 domains (doctor, git, update, lock). Ship.

---

### Phase 2 — Causality Engine
**Reads the ledger. No new writes.**

Pure read — traverses events backward from current state.
`core why` is the Linus wow-factor command.
```
core why                    # current state explained
core why health 95          # why is health 95%?
core trace last-change      # full causal chain
```

Session scope: 1 session. EventGraph struct, backward traversal,
formatted output. No new infrastructure.

---

### Phase 3 — Simulation Engine
**Per-domain dry-run with diff output.**

Add `Simulatable` trait. Implement for doctor + update first —
safest domains, no destructive ops.
```
core simulate doctor        # predicted health after pending changes
core simulate update        # what packages would change
```

Session scope: 1-2 sessions. Trait definition + 2 domain impls.
--dry-run becomes --simulate with structured diff.

---

### Phase 4 — Event Bus (daemon)
**⚠️ Requires dedicated planning session before coding.**

inotify watchers, push-based events, bar + prompt subscribe.
By Phase 4 the event schema is proven — safe to build the bus on top.
```
core events watch           # live stream
```

Session scope: Plan first. Then 2-3 sessions minimum.

---

### Phase 5 — Plugin Boundary
faelight-git, faelight-update, faelight-fm register with core.
Participate in event bus. Ecosystem grows without modifying core.
```
core plugin list
core plugin add faelight-git
```

---

### Phase 6 — Health Forecasting + Intent Integration
Trend analysis, drift prediction, auto-context linking.
```
core doctor forecast
core doctor trend
core intent context
```

---

## Session Rules
```
1. One phase per session. Stop when it compiles and doctor passes.
2. Every session ends with a commit and doctor at 95%+.
3. No phase starts until the previous has run for at least one day.
4. Phase 4 (daemon) gets its own planning session before any code.
5. No v3 work on days with other system work pending.
```

---

## Gate Check
```
✅ v10.1.0 released
✅ Intent 092 closed
✅ faelight-term — scoped (intent 094, WIP acceptable)
✅ faelight-browser — scoped (intent 095, WIP acceptable)
⬜ Individual repos branched — not blocking Phase 1-3
```

Phase 1 is unblocked. Can start today.

## Progress (updated 2026-02-27)
- Phase 1 — Event Ledger        ✅ complete
- Phase 2 — Causality Engine    ✅ complete  
- Phase 3 — Simulation Engine   ✅ complete
- Phase 4 — Planning session    ✅ complete — ready to implement
- Phase 4 — Implementation      ⬜ next dedicated session

---

## Migration Path

v3 is fully backward compatible. Every v2 command works unchanged.
All new capabilities are additive.

---

## Timing

Start condition for Phase 1: gates above satisfied. ✅ Met.
Start condition for Phase 4: dedicated planning session completed.

---

## The Phrase

**"v2 gave you control. v3 gives the forest memory and foresight."**

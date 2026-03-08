---
id: 113
date: 2026-03-06
type: future
title: "Core v5 — The Intelligent System"
status: complete
tags: [core, v5, intelligence, forecasting, causality, learning, rust, architecture]
---

## Vision

**v2 gave structure. v3 gave awareness. v4 gave discipline. v5 gives intelligence — the forest that learns.**

v4 made the system disciplined. It recovers from failure, holds you to your
intent, and tracks security debt over time. The event ledger now contains
months of real operational data — doctor runs, git patterns, intent velocity,
security trends, health fluctuations.

v5 mines that data and turns it into foresight.

The system stops being reactive and becomes predictive. Not through automation
— through pattern recognition that surfaces what the forest already knows but
hasn't said aloud. The operator remains in full control. The system simply
becomes a better advisor.

**The core insight:** The event ledger is not a log. It is a memory.
v5 is the system learning to read its own memory.

---

## The Four Pillars

### Pillar 1 — Health Forecasting Engine

**"The forest sees trouble before it arrives."**

The event ledger contains every doctor run. Health scores over time form a
signal. v5 reads that signal and projects forward.
```
core forecast                    # current health trajectory
core forecast --days 7           # 7-day health projection
core forecast --domain security  # security-specific trend
```

Implementation:
- Read last N doctor events from `runtime/state.db`
- Compute weighted moving average + slope
- If slope is negative over 3+ consecutive runs → warning
- Correlate with recent activity (git churn, package updates, intent changes)
- Output: projected health in 24h, 72h, 7 days with confidence range

Example output:
```
📈 Health Forecast
  Current:    95%
  24h:        94% (stable)
  72h:        91% (slight decline — 3 security findings aging)
  7d:         88% (attention recommended)
  
  Signal: Security debt aging without upstream patches
  Suggestion: review core security trend
```

### Pillar 2 — Causality Engine (Deep)

**"The forest knows why, not just what."**

v3 introduced the event bus. v5 makes it answer questions.
```
core why                         # why is health at current level?
core why health                  # causal chain for health score
core why health --since 2026-03-01  # what changed health this month?
core why intent 098              # what events surrounded this intent?
core why git                     # what patterns precede risky commits?
```

The causality engine reads across domains:
- When did health last drop? What preceded it?
- Which intents correlate with high git churn?
- What system state precedes security findings aging?

Implementation:
- Cross-domain event correlation in `runtime/state.db`
- Time-window analysis: "what changed in the 24h before this event?"
- Causal chain renderer — ordered list of contributing events
- Stored as `runtime/causality/` — built incrementally, not on demand

### Pillar 3 — Pattern Recognition

**"The forest remembers what works."**

After months of operation, patterns emerge. v5 surfaces them.
```
core correlate                   # show discovered cross-domain patterns
core correlate git health        # correlation between git activity and health
core correlate intent velocity   # what conditions produce fast intent completion?
core suggest                     # proactive suggestions based on current state
```

Patterns the system discovers automatically:
- "git churn above 15 files/day preceded health drops 4 of the last 5 times"
- "security scan within 48h of package update correlates with finding reduction"
- "intents completed faster when checkpointed at start"
- "health drifts after 3+ days without a doctor run"

`core suggest` reads current state against known patterns:
```
💡 Suggestions (based on 847 events)
  · No doctor run in 18h — health drift risk elevated
  · 3 security findings aging past 30 days — debt accumulating
  · Last checkpoint was 8 sessions ago — consider cpc before next release
  · INT-103 has been in-progress for 12 days — longest active intent
```

Implementation:
- Pattern store: `runtime/patterns/patterns.toml`
- Computed incrementally — new events trigger pattern re-evaluation
- Confidence score per pattern (requires N observations to surface)
- Never acts — only surfaces. Operator decides.

### Pillar 4 — Compositor Intelligence (When INT-109 Lands)

**"The forest knows where attention lives."**

This pillar activates when faelight-compositor (INT-109) joins the family.
Until then it is designed but dormant.

When the compositor emits events to the ledger — workspace switches, window
focus, layout changes — v5 gains a new data stream: visual attention.
```
core why workspace              # what drove workspace switches today?
core correlate focus health     # does attention fragmentation precede health drift?
core attention --today          # visual attention map for today's session
```

The causality engine can answer questions it never could before:
- What visual topology correlates with high git churn?
- Does focus fragmentation (many workspace switches) precede intent drift?
- When does deep focus (single workspace for 2h+) correlate with fast completion?

This is the payoff of the family model — tools that share data produce
intelligence no individual tool could generate alone.

---

## What Stays The Same

v5 does not replace v4. It extends it.

- Recovery engine: still available, now informed by forecasting
- Intent discipline: still enforced, now enriched with velocity patterns
- Security debt: still tracked, now projected forward
- Event ledger: still the source of truth — v5 only reads, never rewrites

**The philosophy stays the same: manual control over automation.**
v5 makes you better informed. It never acts without you.

---

## Build Order

### Phase 1 — Data Foundation
**Make the ledger queryable for intelligence.**

The event ledger has data but no analytical layer. Phase 1 builds the
foundation all other phases read from.
```
core ledger stats               # event counts, domains, date range
core ledger query <domain>      # filtered event retrieval
core ledger export              # full export for offline analysis
```

- Add indexed views to `runtime/state.db` for time-window queries
- Build `runtime/analytics/` — pre-computed aggregates updated incrementally
- Wire into `core doctor` — analytics rebuilt after each doctor run

### Phase 2 — Health Forecasting
**The first intelligence feature. Low risk, high value.**
```
core forecast
core forecast --days N
```

- Read doctor event history
- Compute trend with weighted moving average
- Render forecast with confidence range
- Wire into `core doctor` output — show forecast line at bottom of health check

### Phase 3 — Causality Engine (Deep)
**Answer "why" across domains.**
```
core why health
core why git
core why intent <id>
```

- Cross-domain time-window correlation
- Causal chain renderer
- Stored incrementally in `runtime/causality/`

### Phase 4 — Pattern Recognition + Suggestions
**The system surfaces what it has learned.**
```
core correlate
core suggest
```

- Pattern store built from event history
- Confidence-gated surfacing (min N observations)
- `core suggest` as daily awareness tool

### Phase 5 — Compositor Intelligence
**Dormant until INT-109. Designed now.**

- WM abstraction layer emits attention events
- Visual topology correlation with health and intent data
- `core attention`, `core why workspace`

---

## Session Rules
```
1. One phase per session.
2. Every session ends at 95%+ health with a clean commit.
3. Intelligence features are read-only — they observe, never act.
4. No phase starts without the previous phase tested and stable.
5. Patterns must be validated against real data before surfacing.
6. Never surface a suggestion with fewer than 5 supporting observations.
```

---

## Gate Check
```
✅ Core v4 complete — event ledger populated with real data
✅ runtime/state.db has 1000+ events across 4+ domains
✅ faelight-pulse built — ledger is readable and queryable
✅ Philosophy alignment confirmed — read-only intelligence
⬜ Phase 1 unblocked — data foundation
⬜ Phase 2 — health forecasting
⬜ Phase 3 — causality engine
⬜ Phase 4 — pattern recognition
⬜ Phase 5 — compositor intelligence (blocked on INT-109)
```

---

## Stats Context (at time of writing)
```
System:      v10.4.0 — Niri Version
Health:      95% (22 checks)
Commits:     1320
Intents:     72 complete
Event ledger: 1000+ events, domains: doctor, git, security, update
Tools:       42 custom Rust binaries
Rust %:      ~95%
Goal:        100% Rust
```

---

## The Version Arc
```
v1-v8    Learning, tools, structure
v9       Production-ready tools, 100% path resilience
v10      Core v2/v3 — self-aware system
v11      Core v4 — reliable, disciplined
v12      faelight-compositor — 100% Rust (INT-109)
v13      Core v5 — intelligent, learns from itself (this intent)
v14      Faelight Forest — complete, self-aware, self-improving
```

---

## The Phrase

**"The best system isn't the one that never breaks.
It's the one that sees the break coming, remembers why it happened last time,
and knows what conditions led there — before you have to ask."**

*"The forest doesn't just watch itself breathe.
It learns the rhythm."* 🌲

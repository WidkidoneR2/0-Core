---
id: 093
date: 2026-02-22
updated: 2026-02-27
type: complete
title: "Core v3 — The Living System"
status: complete
tags: [v10.3, architecture, causality, event-driven, self-aware]
version: 10.3.0
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

### Pillar 1 — Causality Engine ✅
**"Why is the system in this state?"**

Every domain operation writes a structured event to the SQLite ledger at
\`~/0-core/runtime/state.db\`. Commands traverse the ledger backward from
current state to explain why things are the way they are.

\`\`\`
cw      # core why summary — today's activity across all domains
cwh     # core why health — health trajectory with deltas
cwd     # core why domain — domain-specific activity with payloads
ctr     # core trace last — last 10 events chronological
ctrd    # core trace domain — domain-specific trace
\`\`\`

### Pillar 2 — Simulation Engine ✅
**"What will happen if I do this?"**

Before any significant operation, core simulates the outcome and shows
exactly what would change — without touching anything.

\`\`\`
csd     # core simulate doctor — predicted health after pending changes
csu     # core simulate update — what packages would change
\`\`\`

### Pillar 3 — Event Bus ✅
**"The forest reacts to itself."**

faelight-daemon v3.0.0 extended with SQLite polling (2s interval) and
broadcast channel. Any subscriber receives live events as they happen.

\`\`\`
cew     # core events watch — live event stream via daemon
\`\`\`

---

## Supporting Features

### Plugin Registry ✅
\`\`\`
cpl     # core plugin list — all registered plugins with status
cpa     # core plugin add <name> — register a plugin
cps     # core plugin status <name> — plugin detail
\`\`\`

5 plugins registered: faelight-git, faelight-update, faelight-fm,
faelight-bar, faelight-fetch. Domain mappings declared. Versions detected.

### Health Forecasting ✅
\`\`\`
cdt     # core doctor trend — sparkline + pattern analysis over all readings
cdf     # core doctor forecast — predicted trajectory + risk factors
\`\`\`

Based on event ledger history — 20 doctor readings, 93% average health,
+5% drift since first run, 70% of readings at 95%+ health.

---

## What v3 Is NOT

- **Not autonomous** — the system never acts without human confirmation
- **Not AI** — pure deterministic Rust, no models, no inference
- **Not Skynet** — the forest watches and advises, you decide everything
- **Not scope creep** — each pillar is a discrete, shippable unit

The philosophy stays the same: **manual control over automation**.
v3 makes you more informed, not less in control.

---

## Phase Completion

| Phase | Name | Commands | Shipped |
|-------|------|----------|---------|
| 1 | Event Ledger | ce, ces, cef | 2026-02-27 |
| 2 | Causality Engine | cw, cwh, cwd, ctr, ctrd | 2026-02-27 |
| 3 | Simulation Engine | csd, csu | 2026-02-27 |
| 4 | Event Bus | cew | 2026-02-27 |
| 5 | Plugin Registry | cpl, cpa, cps | 2026-02-27 |
| 6 | Health Forecasting | cdt, cdf | 2026-02-27 |

All 6 phases shipped in a single day. v10.3.0 tagged and live.

---

## Implementation Notes

- Event ledger: SQLite at \`~/0-core/runtime/state.db\`, \`events\` table
- Daemon: extended faelight-daemon v3.0.0 with \`broadcast::channel<EventBroadcast>\`
- Poll interval: 2s SQLite read, \`timestamp > last_ts\` cursor
- Plugin registry: TOML at \`~/0-core/00-meta/plugins.toml\`
- Forecasting: linear trend from last 5 doctor readings, risk factor analysis
- All new commands: pure reads, zero side effects except \`plugin add/remove\`

---

## Stats at Completion

\`\`\`
Version:   v10.3.0
Commits:   1281
Aliases:   343 across 43 tools
Health:    95%
New cmds:  16 (ce ces cef cw cwh cwd ctr ctrd csd csu cew cpl cpa cps cdt cdf)
Plugins:   5 registered
Daemon:    v3.0.0 — 17 tasks, 6.1M RSS
\`\`\`

---

## Gate Check

\`\`\`
✅ v10.2.0 released (gate entry)
✅ Intent 092 closed
✅ All 6 phases complete
✅ v10.3.0 released
✅ Doctor at 95%+ throughout
✅ Zero regressions
\`\`\`

---

## The Phrase

**"v2 gave you control. v3 gives the forest memory and foresight."**

*"The forest doesn't just run — it thinks, remembers, and forecasts."* — Intent 093

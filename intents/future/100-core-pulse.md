---
id: 100
date: 2026-03-02
type: future
title: "core pulse — Live Event Stream Terminal View"
status: planned
tags: [core, events, tui, rust, observability, glow]
version: TBD
priority: high
---

## Vision

A live terminal view that streams the event ledger in real time.
Every doctor check, every git operation, every intent transition,
every checkpoint — scrolling past like a heartbeat monitor.

The forest watching itself breathe.
```
core pulse
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
12:31:36  doctor     ✅ health: 95%
12:32:48  checkpoint 📸 auto-intent-098-start
12:32:49  intent     🚀 098 → in-progress
12:33:01  git        ✓  commit 8c71af5
12:35:22  doctor     ✅ health: 95%
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  🌲 Forest is breathing. Focus: INT-098
```

## Why This Glows

The event ledger exists. The broadcast channel exists.
`core pulse` is just a window into what the forest already knows.
It makes the invisible visible. The forest becomes self-illuminating.

## Approach

- Tail `runtime/state.db` events in real time
- Color-code by domain (doctor=green, git=blue, intent=yellow)
- Show current focus and health inline
- Optional: `core pulse --domain git` for filtered stream
- Optional: `core pulse --json` for machine-readable output

## Success Criteria

- [ ] Live event stream with <100ms latency
- [ ] Color-coded by domain
- [ ] Shows current focus intent inline
- [ ] Filterable by domain
- [ ] Graceful exit on Ctrl+C

---

*"The forest watching itself breathe."* 🌲

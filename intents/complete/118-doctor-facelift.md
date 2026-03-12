---
id: 118
date: 2026-03-12
type: future
title: "doctor facelift — cockpit-style health dashboard"
status: complete
tags: [doctor, ui, ratatui, health, dashboard, v10.8]
version: 10.8.0
priority: high
---

## Vision

The doctor command is the most-run command in the forest.
It should feel like a cockpit instrument panel, not a log file.

Right now it's functional but plain — plain borders, flat colors,
sequential output. It should feel like faelight-fm after its overhaul.

## What Changes

- Rounded borders per health section
- Color-coded severity per check (green/yellow/red)
- Summary dashboard line at the top — one glance tells the story
- Forecast integrated inline, not appended at the bottom
- Compact mode for daily use, verbose mode for debugging
- Section grouping: System / Git / Security / Tools / Protection

## Example Vision
```
╭─ 🏥 Faelight Forest v10.8.0 — Health Dashboard ─────────────────╮
│  ✅ 21/22  🟢 95%  📈 +2.1  🔒 locked  1400 commits             │
╰──────────────────────────────────────────────────────────────────╯
╭─ System ──────────────╮ ╭─ Git & Tools ─────────╮ ╭─ Security ──╮
│ ✅ Stow    12/12      │ │ ✅ Git     clean       │ │ ✅ UFW      │
│ ✅ Services 2/2       │ │ ✅ Tools   52/52       │ │ ✅ fail2ban │
│ ✅ Symlinks clean     │ │ ✅ Paths   100%        │ │ ✅ Audit    │
╰───────────────────────╯ ╰───────────────────────╯ ╰─────────────╯
```

## Success Criteria

- [ ] Cockpit-style layout with grouped sections
- [ ] Color severity per check
- [ ] Summary line at top
- [ ] Forecast inline
- [ ] Compact and verbose modes
- [ ] Still runs in < 2 seconds

---
*"The forest should know its own health at a glance."* 🌲

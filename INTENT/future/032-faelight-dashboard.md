---
id: 032
date: 2026-01-10
type: future
title: "faelight-dashboard - TUI System Overview"
status: planned
tags: [rust, tui, dashboard, monitoring]
---

## The Vision
A beautiful TUI dashboard that shows everything about your system at a glance.

## Why
- Currently need multiple commands to check system status
- No unified view of 0-Core health, git, intents
- Want a "home base" when opening terminal

## Features

### Panels
```
┌─ 🌲 Faelight Dashboard v6.5.0 ─────────────────────────────────┐
│                                                                 │
│  ┌─ Health ──────┐  ┌─ Git Status ─────┐  ┌─ Profile ────────┐ │
│  │ ✅ 12/12 pass │  │ 🔓 Unlocked      │  │ 🏠 default       │ │
│  │ 100% healthy  │  │ ✅ Clean         │  │ VPN: Atlanta     │ │
│  │               │  │ ⬆ 0 to push      │  │ CPU: balanced    │ │
│  └───────────────┘  └──────────────────┘  └──────────────────┘ │
│                                                                 │
│  ┌─ Recent Intents ─────────────────────────────────────────┐  │
│  │ 026 ✅ v6.4.0 Architectural Milestone                    │  │
│  │ 025 ✅ core-diff policy mode                             │  │
│  │ 027 📋 faelight-launcher fuzzy search                    │  │
│  │ 028 📋 Btrfs automatic snapshots                         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌─ Quick Stats ────────────────────────────────────────────┐  │
│  │ 🦀 9 Rust tools │ 📦 19 stow packages │ 🔧 188 aliases   │  │
│  │ 📜 26 intents   │ 🔒 6/6 security     │ 💾 3.6TB free    │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  [h]ealth  [g]it  [i]ntents  [p]rofile  [q]uit                 │
└─────────────────────────────────────────────────────────────────┘
```

### Interactions
- `h` - Run full health check
- `g` - Open lazygit
- `i` - Browse intents
- `p` - Switch profile
- `q` - Quit
- `r` - Refresh

## Technical
- Use `ratatui` crate for TUI
- Pull data from existing tools (dot-doctor --json, git, etc.)
- Refresh on keypress or auto every 30s
- Sub-100ms render time

## Integration
- Optional: Show on terminal startup
- `faelight dashboard` command
- Add to faelight unified binary

## Success Criteria
- [ ] All panels render correctly
- [ ] Real-time data from system
- [ ] Keyboard navigation works
- [ ] Sub-100ms startup
- [ ] Beautiful Faelight Forest theme

---
_The forest at a glance._ 🌲📊

---
id: 101
date: 2026-03-02
type: future
title: "faelight-login — Rust Display Manager"
status: planned
tags: [login, display-manager, rust, greetd, wayland, rusty]
version: TBD
priority: high
---

## Vision

The first thing you see when the system boots should be the forest.
Not greetd's default. Not SDDM. Faelight.

A minimal Rust display manager built on greetd's PAM backend,
with Faelight Forest theming, health status at login, and
full integration with the session manager.
```
┌─────────────────────────────────────────┐
│  🌲 Faelight Forest                     │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │  christian                      │   │
│  │  ••••••••                       │   │
│  └─────────────────────────────────┘   │
│                                         │
│  Session: [Niri] [Sway]                │
│  Health:  95% ✅  Commits: 1290        │
│                                         │
└─────────────────────────────────────────┘
```

## Why This Matters

greetd is already Rust. Its IPC protocol is simple JSON.
faelight-login speaks greetd's protocol but renders the forest's identity.
The boot sequence becomes the first event in the daily ledger.

## Approach

- Build on greetd IPC (tuigreet as reference)
- Pure Rust TUI using ratatui
- Show system health + last commit at login
- Session picker: Niri / Sway / faelight-compositor (future)
- Login event emits to faelight-daemon on session start

## Success Criteria

- [ ] PAM authentication via greetd
- [ ] Session selection (Niri, Sway)
- [ ] Health display at login screen
- [ ] Forest theming consistent with faelight-bar
- [ ] Login event in event ledger

---

*"The forest greets you first."* 🌲

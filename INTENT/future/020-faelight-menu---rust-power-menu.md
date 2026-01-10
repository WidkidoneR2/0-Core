---
id: 020
date: 2026-01-09
type: future
title: "faelight-menu - Rust Power Menu"
status: complete
tags: [rust, wayland, ui, sway, v6.2]
---
## The Vision
A minimal Rust power/system menu. Clean overlay for shutdown, reboot, logout, lock.

## Why
- Current fuzzel scripts work but feel hacky
- No visual consistency with faelight tools
- Single binary, no shell scripts

## Features
- Power options (shutdown, reboot, suspend)
- Session options (logout, lock)
- Keyboard navigation (hjkl + Enter)
- Quick keys (l/e/s/r/p)
- Escape to cancel
- Faelight Forest colors

## Technical
- wayland-client + smithay-client-toolkit
- Layer-shell overlay (centered)
- fontdue text rendering

## Success Criteria
- [x] All power actions working
- [x] Keyboard navigation
- [x] Renders on Wayland layer-shell
- [x] Matches faelight aesthetics
- [x] Replaces power-menu-fuzzel.sh
- [ ] Confirmation dialogs (future)
- [ ] Mouse support (future)

## Completed
v0.1.0 released in v6.2.0 (2026-01-10)
- Bound to Super+Shift+Escape

---
_The forest knows when to rest._ 🌲⚡

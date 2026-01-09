---
id: 020
date: 2026-01-09
type: future
title: "faelight-menu - Rust Power Menu"
status: planned
tags: [rust, wayland, ui, sway, v6.3]
---

## The Vision

A minimal Rust power/system menu. Clean overlay for shutdown, reboot, logout, lock.

## Why

- Current fuzzel scripts work but feel hacky
- No visual consistency with faelight tools
- Want smooth animations
- Single binary, no shell scripts

## Features

- Power options (shutdown, reboot, suspend, hibernate)
- Session options (logout, lock)
- Confirmation dialogs
- Keyboard navigation (hjkl + Enter)
- Mouse support
- Profile indicator
- Escape to cancel

## Technical

- wayland-client + smithay-client-toolkit
- Layer-shell overlay
- Same font/color rendering as faelight-bar
- Sway IPC for logout

## Success Criteria

- [ ] All power actions working
- [ ] Keyboard + mouse navigation
- [ ] Renders on Wayland layer-shell
- [ ] Matches faelight-bar aesthetics
- [ ] Replaces power-menu-fuzzel.sh

---

_The forest knows when to rest._ 🌲⚡

---
id: 104
date: 2026-03-02
type: future
title: "faelight-wallpaper — Rust Wallpaper Daemon"
status: planned
tags: [wallpaper, wayland, rust, wlr-layer-shell, rusty]
version: TBD
priority: low
---

## Vision

Replace swaybg (C) with a Rust wallpaper daemon that understands
the forest's identity and health state.

Not just a static image. A living backdrop that responds to the forest.

## Approach

- Implement wlr-layer-shell-unstable-v1 in Rust
- Static wallpaper support (day one)
- Health-reactive: subtle visual shift when health drops
- Intent-aware: wallpaper tint changes with focused intent type
- Config lives in 03-interfaces/wallpaper/

## Success Criteria

- [ ] Replaces swaybg
- [ ] Multi-monitor support
- [ ] Static wallpaper rendering
- [ ] Health-reactive visual state
- [ ] No C wallpaper dependencies remain

---

*"The backdrop knows the forest's mood."* 🌲

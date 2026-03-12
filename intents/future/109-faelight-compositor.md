---
id: 109
date: 2026-03-02
type: future
title: "faelight-compositor — Rust Wayland Compositor on Smithay"
status: in-progress
tags: [compositor, wayland, smithay, rust, v12, architecture]
version: 12.0.0
priority: high
depends_on: [099]
---

## Vision

The last sibling comes home.

A Wayland compositor written entirely in Rust on Smithay,
purpose-built for Faelight Forest. Not a fork. Not a config.
A compositor that knows what it is part of.

See INT-099 for full architectural specification.

## What Makes This Different

Every other compositor is substrate.
faelight-compositor is a participant.

It emits events to faelight-daemon.
It writes to the event ledger.
It integrates with core's capability model.
doctor monitors its health.
The causality engine can query its topology.

## Day One Feature Set

- Column-based tiling (informed by Niri study)
- 5 workspaces, keyboard navigation
- Single monitor (AMD laptop)
- Input handling (keyboard + touchpad)
- Lock integration with core-protect
- Event emission: workspace.switch, window.focus, window.open
- doctor health check: compositor state

## Success Criteria

- [ ] Replaces Niri as primary compositor
- [ ] 100% Rust stack achieved
- [ ] Events flowing into ledger
- [ ] doctor monitors compositor health
- [ ] faelight-bar compatible

---

*"The compositor is the last one to come home."* 🌲

## Progress Log

### 2026-03-11 — v0.1.0 shipped
- Crate created in workspace: `rust-tools/faelight-compositor/`
- Smithay 0.7.0 (git) added to workspace dependencies
- `FaelightCompositor` state struct modeled on smallvil
- All required protocol handlers implemented
- `self.emit()` wired into `focus_changed()` and `new_toplevel()`
- Winit backend added — runs nested inside Niri
- **First frame rendered** — forest green `#11140f` background
- Binary deployed to `/usr/local/bin/faelight-compositor`
- Commits: 2732c69, proof of life achieved

## Gate Check
- ✅ Smithay added to workspace dependencies
- ✅ FaelightCompositor state struct complete
- ✅ Protocol handlers implemented
- ✅ Event emission wired (window.focus, window.open)
- ✅ Winit backend — first frame rendered
- ✅ Binary deployed
- ⬜ Events writing to state.db
- ⬜ DRM/udev backend (real hardware)
- ⬜ Column tiling layout
- ⬜ Input handling
- ⬜ doctor health check integration
- ⬜ faelight-bar compatible
- ⬜ Replaces Niri as primary compositor

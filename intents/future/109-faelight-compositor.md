---
id: 109
date: 2026-03-02
type: future
title: "faelight-compositor — Rust Wayland Compositor on Smithay"
status: planned
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

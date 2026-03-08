---
id: 103
date: 2026-03-02
type: future
title: "faelight-idle — Rust Idle Daemon"
status: complete
tags: [idle, lock, wayland, rust, ext-idle-notify, rusty]
version: TBD
priority: medium
---

## Vision

Replace swayidle (C) with a Rust idle daemon that integrates
with core-protect and the event ledger.

Idle is a security boundary. It should be owned by the forest.

## Approach

- Implement ext-idle-notify-v1 Wayland protocol in Rust
- Configurable timeouts via 03-interfaces/
- On idle: emit event, trigger core-protect lock
- On wake: emit event, log duration
- Idle patterns become health signals (long idle = away, frequent idle = distracted)

## Success Criteria

- [x] Replaces swayidle completely
- [x] Integrates with core-protect lock
- [x] Idle/wake events in event ledger
- [x] Configurable timeouts
- [x] No C idle dependencies remain

---

*"Even stillness is an event the forest remembers."* 🌲

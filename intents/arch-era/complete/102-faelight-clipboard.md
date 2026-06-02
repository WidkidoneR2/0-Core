---
id: 102
date: 2026-03-02
type: future
title: "faelight-clipboard — Rust Clipboard Manager"
status: complete
tags: [clipboard, wayland, rust, wlr-data-control, rusty]
version: TBD
priority: medium
---

## Vision

Replace wl-clipboard (C) with a Rust-native clipboard manager
that integrates with the event ledger and respects Wayland's
security model.

## Why This Matters

wl-clipboard is C. It's a dependency we understand but don't own.
faelight-clipboard is a sibling that comes home.

Clipboard history is also useful data — what you copy is a signal
of what you're working on. That signal belongs in the forest.

## Approach

- Implement wlr-data-control-unstable-v1 protocol in Rust
- Clipboard history stored in runtime/ (configurable depth)
- `faelight-clipboard history` — searchable history via faelight-launcher
- Optional: clipboard event emission to event ledger
- Replaces: wl-clipboard, wl-copy, wl-paste

## Success Criteria

- [x] wl-copy / wl-paste compatible interface
- [x] Clipboard history (last 50 entries)
- [x] Searchable via faelight-launcher integration
- [x] Wayland security model respected
- [x] No C clipboard dependencies remain

---

*"Even what you copy is the forest's memory."* 🌲

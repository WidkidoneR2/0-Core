---
id: 270
date: 2026-05-03
type: feature
title: "faelight-login v2 -- slint native Rust greeter"
status: in-progress
tags: [login, faelight-login, slint, wayland, greetd, ui, rust]
---

## Vision

Replace the current ratatui TUI greeter with a native Wayland UI using slint.
Login is a one-shot declarative UI — you see it once, type a password, it disappears.
Slint's declarative DSL compiles to Rust and forces clean separation of UI and business logic.

## Architecture Decision (2026-05-03)

Evaluate slint for the lock screen and login UI. Same time-box rule as INT-239:
if slint + Wayland/greetd handoff proves painful, keep current ratatui approach
and upgrade it instead.

The login screen should feel like the forest — calm, intentional, minimal.
A Faelight tree animation on boot. Password field. Nothing else.

## Gates

- [ ] slint prototype compiles and renders on Niri/Wayland
- [ ] greetd handoff works correctly with slint binary
- [ ] Password entry works with proper security (no echo, clear on submit)
- [ ] Forest aesthetic: tree motif, forest color palette
- [ ] Boot animation (optional but desirable for presentation)
- [ ] Demonstrated: login → desktop in < 2 seconds

## Notes

Time-box: one week. If slint + greetd integration proves painful, upgrade
current ratatui greeter instead. The philosophy wins over the framework.
Evaluate alongside INT-239 (bar v2 with iced) since both share the
"new framework evaluation" pattern.

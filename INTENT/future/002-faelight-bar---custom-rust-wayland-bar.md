---
id: 002
date: 2025-12-31
type: future
title: "faelight-bar - Custom Rust Wayland Bar"
status: planned
tags: [rust, wayland, v5.0, celebration]
---

## Why

v5.0.0 is a CELEBRATION of the journey.

This will be:

- First Rust project (learning + proving the tech)
- Something RARE (custom Wayland bars are uncommon)
- Philosophy embodied (minimal, intentional, beautiful)
- Marks the transition (dotfiles → philosophy → tool builder)

Also: Waybar works, but it's not OURS.
faelight-bar will be 0-Core's bar, built with 0-Core principles.

## What

Custom Wayland status bar written in Rust.

Features (minimal but complete):

- Workspaces (Hyprland socket integration)
- Clock (system time)
- Battery (charge status, icons)
- Network (wifi/ethernet)
- VPN indicator (Mullvad detection)
- Lock status (0-core locked/unlocked)

Design philosophy:

- i3bar aesthetic (minimal, text-focused)
- Modern underneath (Rust, Wayland protocols)
- Configurable via TOML
- Security-first (no unnecessary features)

## Timeline

v5.0.0 (Q2-Q3 2026)

Learning period: Jan-Jun 2026
Build period: Jun-Aug 2026
Polish & ship: Aug-Sep 2026

## Technical Stack

- Language: Rust
- Wayland: smithay-client-toolkit
- Rendering: Cairo/Pango
- Config: TOML
- Async: tokio

## Why Rust for This

Perfect learning project:

- Standalone (doesn't break 0-Core if I struggle)
- Forces me to learn Wayland protocols
- Async patterns (socket reading, polling)
- Graphics rendering
- Real-world complexity

If faelight-bar succeeds, v5.1 rewrites all scripts in Rust.
If it struggles, we learn what NOT to do.

## Success Criteria

- Runs stable (no crashes, memory leaks)
- Feature parity with our Waybar usage
- Daily-driveable
- Code quality (idiomatic Rust)
- Proves Rust is viable for 0-Core

## The Celebration

This marks:

- v1.0 → v5.0 (major milestone)
- Dotfiles → Philosophy → Builder
- Linux user → Linux craftsman
- Following → Leading

The forest grows into something new.

### 2025-12-31

[Initial creation]

---

_Part of the 0-Core Intent Ledger_ 🌲

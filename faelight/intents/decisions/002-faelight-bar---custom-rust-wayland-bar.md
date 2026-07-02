---
id: 002
date: 2025-12-31
type: future
title: "faelight-bar - Custom Rust Wayland Bar"
status: complete
tags: [rust, wayland, bar, hyprland, v5.0]
---

## The Vision

A minimal, intent-aware status bar that reflects system state, not decoration.
Built in Rust. Integrated with 0-Core. Uniquely yours.

## Why

Current Waybar works but:

- Generic configuration
- No integration with profiles/intents
- Not built by us
- Can't extend with our philosophy

faelight-bar will be:

- Intent-aware (bar changes with profiles)
- Diagnostic-first (health at a glance)
- Minimal (information, not decoration)
- Ours (Rust, from scratch)

## Design Principles

1. **Color = State** — Green healthy, Blue connected, Amber attention, Red danger
2. **Animation = Information** — Never decorative, always meaningful
3. **Contextual, not cluttered** — Modules appear/disappear based on intent
4. **Peripheral vision friendly** — Parse state without reading text

## Version Roadmap

### v0.1 - Static Status Bar

**Goal:** Get pixels on screen. Learn Wayland rendering.

Modules:

- Time / date
- Battery
- Network status

No interaction. Read-only. Flat design.

**Rust concepts:** Event loop, rendering lifecycle

### v0.2 - Hyprland IPC

**Goal:** React to window manager state.

Modules:

- Workspaces (clickable later)
- Active window title

**Rust concepts:** IPC listeners, async events

### v0.3 - Intent-Aware Modules ⭐

**Goal:** Bar reflects current profile. THE DIFFERENTIATOR.

Behavior:

- `profile gaming` → shows GPU, performance metrics
- `profile work` → shows VPN status, focus indicator
- `profile low-power` → shows battery prominently
- Modules appear/disappear dynamically

**Rust concepts:** Typed shared state, hot reload, observer pattern

### v0.4 - Clickable Panels

**Goal:** Add minimal interactivity.

Features:

- Click volume → slider panel
- Click profile → profile switcher
- Click updates → package list
- Panels float below bar
- Keyboard-first interaction

**Rust concepts:** Input handling, focus management, message passing

### v0.5 - Diagnostics Integration

**Goal:** dot-doctor in your peripheral vision.

Features:

- Green/amber/red health indicators
- Git repo dirty indicator
- Config drift warning
- Update safety warnings
- Zero noise when healthy

**Rust concepts:** Deterministic logic, structured diagnostics

### v1.0 - Polished & Stable

**Goal:** Daily driver ready.

Features:

- All modules stable
- Smooth transitions (purposeful only)
- Complete documentation
- Profile-driven themes

## Color System — Faelight Forest Palette

### Base (Backgrounds)

| Name         | Hex     | Use                      |
| ------------ | ------- | ------------------------ |
| Forest Night | #0f1411 | Main bar background      |
| Deep Moss    | #161d19 | Module background (idle) |
| Pine Shadow  | #1e2622 | Hover / focused          |

### Primary Accent (Faelight Green)

| Name           | Hex     | Use                      |
| -------------- | ------- | ------------------------ |
| Faelight Green | #6be3a3 | Active icon color        |
| Soft Faelight  | #2e5f49 | Active module background |
| Dim Faelight   | #4fbf8a | Secondary accents        |

### Secondary Accent (Cool Blue)

| Name          | Hex     | Use                     |
| ------------- | ------- | ----------------------- |
| Faelight Blue | #5cc8ff | Network, connectivity   |
| Mist Blue     | #243a44 | Network background tint |
| Deep Water    | #3fa9dd | Strong signal / active  |

### Tertiary Accent (Warnings)

| Name       | Hex     | Use                |
| ---------- | ------- | ------------------ |
| Amber Leaf | #f5c177 | Warnings, updates  |
| Rust Glow  | #d08770 | Errors, alerts     |
| Soft Ember | #4a2e25 | Warning background |

### Neutral

| Name       | Hex     | Use                 |
| ---------- | ------- | ------------------- |
| Fog White  | #d7e0da | Primary text        |
| Ash Grey   | #9aa7a0 | Secondary text      |
| Stone Grey | #6f7f77 | Disabled / inactive |

### Color Semantics (Never Mix)

| Color    | Meaning                   |
| -------- | ------------------------- |
| Green    | Active / healthy          |
| Blue     | Connected / informational |
| Amber    | Attention needed          |
| Red-rust | Danger                    |
| Grey     | Inactive                  |

## Depth Effect (No Shaders)

Fake 3D without complexity:

- Top edge: lighten background ~5%
- Bottom edge: darken ~8%
- Icon glow: same color, 30-40% alpha

This gives the "instrument panel" feel.

## Module Examples

### Wi-Fi Module

**Connected:**

- Background: Mist Blue (#243a44)
- Icon: Faelight Blue (#5cc8ff)
- Subtle glow

**Disconnected:**

- Background: Deep Moss
- Icon: Stone Grey
- No glow

### Profile Indicator

Shows current profile icon in bar:

- 🏠 default
- 🎮 gaming
- 💼 work
- 🔋 low-power

Visual confirmation of system state.

## Dependencies

- Learn Rust fundamentals first (bump-system-version, dotctl, etc.)
- Complete v5.0.0 CLI rewrites before starting faelight-bar
- This is the capstone project, not the starting point

## Success Criteria

**v1.0 is complete when:**

- [x] Replaces Waybar as daily driver
- [x] Profile switching changes bar appearance
- [x] Diagnostics visible at a glance
- [x] No crashes, no memory leaks
- [x] Documented and shareable

## Completion Notes (2026-01-06)

Built in a single day:

- v0.1: Static bar, time, font rendering
- v0.2: Hyprland IPC (workspaces, window title)
- v0.3: Profile-aware theming
- v0.4: Full modules (VPN, Battery, WiFi, Volume)
- v0.5: Diagnostics (health %, lock status)
- v0.6: Click handling

~500 lines of Rust. Zero external dependencies. Daily driver.

_The forest grew its own bar._ 🦀🌲

## Philosophy

> The bar is a health monitor, not decoration.
> Information, not eye candy.
> Systems engineer energy.

---

_The forest glows. State at a glance._ 🌲

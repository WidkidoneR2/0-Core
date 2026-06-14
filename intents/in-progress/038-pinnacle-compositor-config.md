---
id: 038
date: 2026-06-07
type: feature
title: "Pinnacle compositor config: Lua config, layer-shell, lock screen"
status: in-progress
tags: [pinnacle, compositor, lua, layer-shell, lock-screen, config]
priority: medium
---

## Vision
Pinnacle runs as a stable daily-driver compositor.
All forest tools work correctly under it.
niri remains fallback until this is complete.

## Prerequisites (from INT-021 findings)
- Pinnacle v0.2.2 confirmed working on Framework 16
- EGL hardware acceleration confirmed
- alacritty, faelight-bar, faelight-notify work

## Work Items

### 1. Lua config
- Complete init.lua that mirrors niri keybinds
- Auto-starts forest services (bar, notify)
- Workspace management
- Session stays alive

### 2. faelight-bar layer-shell
- Update bar to use wlr-layer-shell properly
- Full width rendering under Pinnacle
- Anchored to top of screen

### 3. Lock screen
- hyprlock does not work under Pinnacle
- Options: swaylock, waylock, or faelight-lock v2
- Must work with greetd/tuigreet

### 4. faelight-menu
- Fix workspace integration under Pinnacle
- Currently exits immediately

## Progress (2026-06-08)
- modules/desktop/pinnacle.nix complete as NixOS module
- modules/desktop/mango.nix complete as NixOS module
- Both compositors enabled via faelight.desktop.*.enable options
- greetd session entries owned by modules (niri.desktop removed)
- mangowc from nixpkgs replacing broken flake input
- Broken mango flake input removed from flake.nix
- Next: Pinnacle Lua config, lock screen, faelight-menu fix

## Progress (2026-06-14)
Decision (2026-06-14): Mango = personal daily driver, Pinnacle = pure-Rust testing profile, both served via the faelight-compositor bridge (INT-055). niri retired / to be removed.
Proof (2026-06-14): Pinnacle launches, stays alive, and manages/tiles Wayland clients nested on NixOS -- compositor confirmed working.
- Custom Pinnacle Lua config is the remaining keystone (closes 006 keybinds + this intent).

## Gate (re-scoped 2026-06-14 -- pure-Rust testing profile)
- [ ] Pinnacle stays alive with Lua config
- [ ] Core keybinds ported to Pinnacle Lua
- [ ] fsh works inside Pinnacle terminal
- [ ] Forest tools (faelight-notify, etc.) work under Pinnacle
- [ ] (stretch, testing-profile optional) lock screen under Pinnacle
- [ ] (stretch, testing-profile optional) faelight-menu under Pinnacle

Moved out: faelight-bar layer-shell rendering -> INT-053. Dropped: "run as daily driver for 1 week nested in niri" (niri retired; Pinnacle is the testing profile, not the daily driver).

## Note
This is a side track -- does not block INT-030 or INT-026.
Work on it in small sessions alongside main shell/Friday work.

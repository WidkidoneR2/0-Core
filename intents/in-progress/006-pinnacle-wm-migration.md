---
id: 006
date: 2026-06-03
type: infrastructure
title: "Pinnacle WM: compositor migration path, i3-style ownership model"
status: in-progress
tags: [pinnacle, compositor, wayland, i3, nixos]
priority: medium
---

## Why

Pinnacle offers i3-style tiling with Lua configuration and full Wayland
native ownership. The forest deserves a compositor it owns completely.
NixOS makes this safe -- niri stays as fallback generation.

## Approach

1. Study Pinnacle Lua config model
2. Port niri keybinds to Pinnacle
3. Wire faelight-bar as Pinnacle bar
4. Test in VM before switching

## Progress (2026-06-08)
- NixOS module created: modules/desktop/pinnacle.nix
- mkEnableOption wired, faelight.desktop.pinnacle.enable = true
- greetd session entry owned by module
- pinnacle package from flake input installing correctly
- xdg-desktop-portal-wlr wired
- Next: Lua config, keybind porting, faelight-bar wiring

## Progress (2026-06-14)
Decision (2026-06-14): Mango = personal daily driver, Pinnacle = pure-Rust testing profile, both served via the faelight-compositor bridge (INT-055). niri retired / to be removed.
Proof (2026-06-14): Pinnacle launches, stays alive, and manages/tiles Wayland clients nested on NixOS -- compositor confirmed working.
- Default Pinnacle config validated; custom Lua config (INT-038) is the remaining work.

## Gate (re-scoped 2026-06-14 -- pure-Rust testing profile, not daily driver)
- [x] Pinnacle launches cleanly on NixOS
- [x] Pinnacle manages and tiles Wayland clients (confirmed nested 2026-06-14)
- [ ] Core keybinds drive it (full suite lives in INT-038 Lua config)
- [ ] Forest toolchain (fsh, faelight-notify) runs under Pinnacle

Dropped: "all 102 keybinds" (daily-driver overkill for a testing profile); faelight-bar rendering moved to INT-053.

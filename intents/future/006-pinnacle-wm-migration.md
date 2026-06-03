---
id: 006
date: 2026-06-03
type: infrastructure
title: "Pinnacle WM: compositor migration path, i3-style ownership model"
status: planned
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

## Gate

Pinnacle session starts cleanly. All 102 keybinds working.
faelight-bar renders correctly.

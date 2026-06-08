---
id: 005
date: 2026-06-03
type: feature
title: "faelight-login + faelight-menu: proper NixOS login flow with greetd"
status: in-progress
tags: [faelight-login, faelight-menu, greetd, tuigreet, nixos]
priority: high
---

## Why

Current login flow: LUKS decrypt → login prompt → manually type niri-session.
This is three steps that should be one clean forest greeting.

## Approach

1. Add services.greetd with tuigreet to configuration.nix
2. Configure greetd to auto-start niri session
3. faelight-login becomes the greeter face
4. faelight-menu works properly as power/session menu

## Gate

Boot → LUKS → faelight-login greeting → niri session starts automatically.
faelight-menu opens cleanly with Mod+Escape.

## Progress (2026-06-04)

### What works
- greetd wired correctly in configuration.nix
- faelight-login launches as greeter
- Authentication works
- niri-session starts automatically after auth -- no manual typing needed
- Plymouth added for clean LUKS prompt

### What needs VM work before closing
- faelight-login box sizing -- slightly too large, username field visibility issues
- Need VM to iterate on ratatui layout without rebooting real machine
- Specific fixes needed:
  - Terminal size detection on greetd VT
  - Box dimensions responsive to actual terminal size
  - Username/password field alignment

### Generation notes
- gen 45: stable baseline (before faelight-login changes)
- gen 46: greetd + faelight-login attempt (sizing issues)
- Rolled back to gen 45

### Next step
INT-021 (Pinnacle VM) sets up the VM infrastructure.
Use that VM to iterate on faelight-login sizing before rebuilding real system.

### Update (2026-06-08)
- greetd session entries now module-owned (pinnacle.nix, mango.nix)
- niri.desktop session entry removed from system (was hardcoded, now gone)
- Remaining gate: faelight-menu Mod+Escape under Pinnacle/Mango
- Blocked by: Pinnacle Lua config (INT-038)

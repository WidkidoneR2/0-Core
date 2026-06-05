---
id: 021
date: 2026-06-03
type: study
title: "Pinnacle VM study: prove compositor in nixos-lab before touching real system"
status: in-progress
tags: [pinnacle, vm, study, compositor, nixos-lab, safety]
priority: high
---

## Why

Pinnacle changes everything downstream -- bar, notify, menu, login, terminal.
Migrating to Pinnacle on the real system without VM proof is too risky.
nixos-lab VM is the safe proving ground.

## Approach

1. Install Pinnacle in nixos-lab VM
2. Port current niri keybinds to Pinnacle Lua config
3. Test faelight-bar renders correctly
4. Test faelight-notify works
5. Test faelight-lock works
6. Document what breaks and what needs changes
7. Only then plan real system migration

## What Pinnacle Needs From The Forest

- faelight-bar: layer-shell protocol support
- faelight-notify: compositor-native notifications
- faelight-menu: Pinnacle workspace integration
- faelight-login: session lifecycle ownership
- All visual tools speaking the same language

## Gate

Pinnacle session runs in VM.
All 6 core forest tools work under Pinnacle.
Written migration plan for real system.

---
id: 019
date: 2026-06-03
type: feature
title: "faelight-notify v5: NixOS-native, noti research, layer-shell ready"
status: in-progress
tags: [faelight-notify, nixos, noti, layer-shell, pinnacle, wayland]
priority: medium
---

## Why

faelight-notify v4 works on NixOS but was built for Arch assumptions.
v5 should be compositor-native, layer-shell aware, and Pinnacle-ready.

## Research: noti

noti (https://github.com/variadico/noti) is a notification tool that
triggers system notifications when commands finish. Study patterns for:
- How it integrates with desktop notification systems
- D-Bus patterns worth adopting
- Whether faelight-notify should absorb noti-style functionality

## Approach

- Study noti source and patterns
- Add layer-shell support for Pinnacle compatibility
- Keep niri compatibility during transition
- D-Bus interface improvements
- Forest-aware: notify on intent complete, health drop, Friday events

## Gate

faelight-notify v5 works on niri AND is Pinnacle-ready.
noti research documented in labs/graduated/.

---
id: 014
date: 2026-06-03
type: feature
title: "faelight-dashboard v2: full NixOS replacement, ratatui, forest-native"
status: cancelled
tags: [dashboard, ratatui, nixos, forest-native]
priority: medium
---

## Why

faelight-dashboard was an Arch script. On NixOS it needs a full rethink.
The dashboard should show forest health, active intents, Friday status,
system resources, and recent commits -- all in one ratatui TUI.

## Approach

- ratatui TUI, standalone binary in workspace
- Live health score, integrity, forecast
- Active intents with progress
- Friday state: patterns, facts, last prediction
- Recent git activity
- System: CPU, RAM, disk, battery

## Gate

Mod+Alt+m launches dashboard. All panels render correctly.

## Gate Check
🚫 014 -- cancelled: Scoped as a full NixOS replacement dashboard. There is no NixOS to replace. -- approved by: christian 2026-08-27

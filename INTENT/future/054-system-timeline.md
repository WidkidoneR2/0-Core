---
id: 054
date: 2026-01-14
type: future
title: "System Timeline - Time Travel for Configs"
status: planned
tags: [v8.0, infrastructure, history, recovery]
version: 8.0.0
---

## Vision
Visual timeline of system changes with ability to view state at any point in time.

**Philosophy:** "Recovery requires history."

## Features

### 1. Timeline View
```bash
$ timeline show

2025-12-01  Omarchy system (pre-incident)
2025-12-15  [INCIDENT 001] Password auth failure
2025-12-16  [INTENT 035] Arch + Sway migration
2025-12-20  [INTENT 002] faelight-bar v1.0
2026-01-10  [INTENT 047] faelight-core created
2026-01-14  [INTENT 048] Tool migrations (v7.0.0)
            ↑ YOU ARE HERE
```

### 2. Point-in-Time Query
```bash
$ timeline at 2025-12-01

System state on 2025-12-01:
  OS: Omarchy (Arch derivative)
  WM: Hyprland v0.35
  Bar: Waybar
  Launcher: walker
  Intents: 25 complete

Configs:
  ~/.config/hypr/hyprland.conf (sha256:...)
  ~/.config/waybar/config (sha256:...)
```

### 3. Diff Between Points
```bash
$ timeline diff 2025-12-01 2026-01-14

Added:
  + faelight-core
  + faelight-bar
  + faelight-menu
  + faelight-notify
  + faelight-launcher
  
Removed:
  - waybar
  - walker
  - hyprland
  
Changed:
  ~/.config/sway/config (35 modifications)
  
Intents: +23 (25 → 48)
```

### 4. Config Snapshots
```bash
$ timeline configs-at 2025-12-01

All configs on 2025-12-01:
  ~/.config/hypr/hyprland.conf
  ~/.config/waybar/config
  ~/.zshrc
  [... full list ...]

$ timeline restore-config ~/.config/sway/config 2025-12-20
# Restores single config to state at date
```

## Data Sources
1. **Git commits** (primary timeline)
2. **Intent metadata** (context)
3. **Config registry** (snapshots)
4. **Btrfs snapshots** (if available)

## Success Criteria
- [ ] CLI tool: `timeline`
- [ ] Show visual timeline
- [ ] Query state at any date
- [ ] Diff between two dates
- [ ] Restore individual configs
- [ ] Integration with git history
- [ ] Integration with config-registry
- [ ] Timeline visualization in terminal

## Timeline
**v8.0.0**

---

_"History is not just what happened. It's why it happened."_ 🌲

---
id: 169
date: 2026-03-28
type: future
title: "Niri Autostart Audit — Everything That Starts Must Start Correctly"
status: planned
tags: [niri, autostart, systemd, startup, reliability, services]
version: 11.5.0
priority: medium
---

## The Problem
The autostart chain has never been formally audited since:
- Migration from Sway to Niri
- faelight-notify migration to systemd service
- faelight-term now launching fsh instead of zsh
- faelight-bar multiple rewrites
- New tools added to autostart without removal of old ones

Currently spawning at startup (from config.kdl):
```
faelight-wallpaper
faelight-clipboard watch
launch-bar
faelight-niri-bridge
faelight-idle
```

Plus systemd user services:
```
faelight-notify (systemd)
```

Questions that have no answers:
- What starts first? Does order matter?
- What happens if faelight-bar crashes?
- What happens if faelight-niri-bridge fails?
- Are there race conditions between services?
- Is anything starting that shouldn't be?
- Are any old Sway-era things still running?

## The Audit Process

### Step 1 — Document current startup chain
```bash
# What does Niri spawn?
grep "spawn-at-startup" ~/.config/niri/config.kdl

# What systemd user services are enabled?
systemctl --user list-units --type=service --state=running

# What processes are running that shouldn't be?
ps aux | grep faelight
```

### Step 2 — Verify each service

For each autostart item, answer:
```
✅ Still needed?
✅ Starts correctly?
✅ Has --health check?
✅ Restarts on crash?
✅ Logs errors somewhere?
✅ No Sway/old references?
```

### Step 3 — Define correct start order
Some services depend on others:
```
Niri compositor starts
  → faelight-wallpaper (needs compositor)
  → faelight-niri-bridge (needs compositor)
  → launch-bar (needs compositor + niri socket)
  → faelight-clipboard watch (needs compositor)
  → faelight-idle (needs compositor)
  
systemd (independent):
  → faelight-notify (D-Bus, independent of compositor)
```

### Step 4 — Add health checks to all autostart tools
Every tool that autostarts should support:
```bash
faelight-wallpaper --health  # "faelight-wallpaper v1.0.0 — healthy"
faelight-clipboard --health  # etc
launch-bar --health
faelight-niri-bridge --health
faelight-idle --health
```

This allows doctor to verify autostart tools are running.

### Step 5 — Systemd migration consideration
Should more tools move from Niri spawn to systemd user services?

Benefits of systemd:
- Auto-restart on crash
- Proper logging (journalctl)
- Dependency ordering
- Status monitoring

Candidates:
```
faelight-idle      → systemd (already pattern established by notify)
faelight-clipboard → systemd (watch mode = daemon)
faelight-niri-bridge → keep Niri spawn (needs compositor)
faelight-wallpaper   → keep Niri spawn (needs compositor)
launch-bar           → keep Niri spawn (needs compositor)
```

## Phase 1 — Audit and Document
Run the audit. Answer all questions. Document findings.
Create: docs/AUTOSTART-MAP.md

## Phase 2 — Clean Up Sway Remnants
Remove any swaymsg, sway-*, or sway references from:
- .zshrc
- aliases.zsh  
- config files
- Scripts

## Phase 3 — Add Health Checks
Every autostart tool gets --health flag.
Doctor check "System Services" expanded to verify all autostart tools.

## Phase 4 — Systemd Migration
Move appropriate daemons to systemd user services.
Follow the faelight-notify pattern exactly.

## Phase 5 — Crash Recovery
Each service has restart policy:
```ini
[Service]
Restart=on-failure
RestartSec=3s
```

Test crash recovery: kill each service, verify restart.

## Gate Check
```
⬜ docs/AUTOSTART-MAP.md — full startup chain documented
⬜ All Sway remnants removed from configs
⬜ Every autostart tool has --health check
⬜ Doctor "System Services" verifies all autostart tools
⬜ faelight-idle moved to systemd (if appropriate)
⬜ faelight-clipboard moved to systemd (if appropriate)
⬜ Crash recovery tested for all services
⬜ Start order documented and verified
⬜ No race conditions on fresh boot
```

## The Phrase
**"A forest whose trees fall silently
in the night — unnoticed, unreported —
is a forest that does not know itself.
Every service that starts
must be accounted for."**

---
*"Autostart audit is not glamorous work.
It is the foundation work that makes everything else reliable."* 🌲

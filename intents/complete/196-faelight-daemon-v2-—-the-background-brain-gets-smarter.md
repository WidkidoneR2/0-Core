---
id: 196
date: 2026-04-05
type: planned
title: "faelight-daemon v2 — The Background Brain Gets Smarter"
status: complete
tags: [daemon, background, intelligence, socket, neovim, v2]
---
## Current State
faelight-daemon v1 runs as a systemd service and provides a socket
at /tmp/faelight-daemon.sock for neovim integration. It does basic
background operations but its intelligence is minimal.

## What v2 Becomes
The daemon is the always-on brain of the forest. While you work,
it watches, learns, and prepares. It should become the bridge between
all intelligence layers — contextd, predictions, partner proposals.

## Improvements

### Neovim Integration Upgrade
Currently: socket exists, basic communication.
v2: rich protocol — neovim can query forest state, get intent context,
receive suggestions based on the file being edited.
"You are editing faelight-shell/src/commands/mod.rs — INT-194 is active"

### Health Watchdog
Daemon monitors system health in background.
If health drops below 95%, daemon queues a notification.
No more discovering health issues only when you run d.

### Prediction Pre-computation
Between commands, daemon pre-computes likely next predictions.
When fsh reads the prediction, it is already ready — zero latency.

### Event Aggregation
faelight-contextd detects signals. faelight-daemon acts on them.
Daemon becomes the action layer for contextd insights.

### Memory Pressure Monitoring
Daemon monitors memory usage of all forest tools.
Alerts if any tool is leaking memory over time.
Logs growth trends to state.db.

## Commands
daemon status         — current daemon health and activity
daemon log            — recent daemon activity
daemon neovim         — neovim integration status
daemon watchdog       — health watchdog status
daemon predictions    — pre-computed prediction cache

## Gate Check
✅ daemon neovim integration v2 — GetNeovimContext protocol live
✅ health watchdog — 60s polling, alerts to engine_signals on drop below 95%
✅ prediction pre-computation — 30s cache in shell_state.daemon_prediction
✅ event aggregation — 30s signal count summary
⬜ memory pressure monitoring — deferred to v3
✅ core daemon status/signals/watchdog/context/neovim all live

## The Phrase
"The daemon that only runs is infrastructure.
The daemon that watches, learns, and acts
is intelligence." 🌲

---
id: 053
date: 2026-06-09
type: feature
title: "faelight-bar v2: i3-style wlr-layer-shell bar for MangoWM and Pinnacle"
status: planned
tags: [bar, wayland, layer-shell, mango, pinnacle, rust]
priority: high
---
## Why
faelight-bar v1 was a niri prototype. MangoWM and Pinnacle need a proper
wlr-layer-shell bar built for tiling compositors. i3-style: workspaces,
system stats, clock, forest status.

## Vision
- wlr-layer-shell anchored top
- Workspace indicators (MangoWM tags)
- System: CPU, RAM, battery, wifi
- Forest: Health %, active intent, Friday status
- Clock with neon candy colors
- Works under both MangoWM and Pinnacle

## Gate
- [ ] Renders correctly under MangoWM
- [ ] Renders correctly under Pinnacle
- [ ] Workspace indicators update live
- [ ] Forest status shows health and active intent
- [ ] Autostart via compositor config

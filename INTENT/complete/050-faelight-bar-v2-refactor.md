---
id: 050
date: 2026-01-14
type: future
title: "faelight-bar v2.0 - Architectural Refactor"
status: complete
completed: 2026-01-14
tags: [rust, bar, architecture, performance, v7.0]
---

## ✅ COMPLETE - Cached Model Architecture

**Shipped:** faelight-bar v2.0 with complete cached model architecture

## What We Built

### Cached Model
All system state cached in `BarModel`:
- Workspaces (JSON parsed)
- Volume, WiFi, VPN, Battery
- Profile, Health, Lock status
- Time

### Per-Module Update Frequencies
- **Fast (500ms):** workspaces, time, volume, wifi, vpn, locked
- **Medium (10s):** battery
- **Slow (30s):** health (expensive git operations)

### JSON Parsing
- Replaced brittle string matching with `serde_json`
- Robust against JSON formatting changes
- Handles named workspaces, scratchpad, multi-output

## Performance Gains

**Before:**
- ~200ms blocking I/O every 500ms
- Brittle string parsing
- Wasteful polling (battery 2x/sec, health 2x/sec)

**After:**
- <1ms draw time (pure rendering)
- Efficient polling (battery 1x/10s, health 1x/30s)
- Smooth, non-blocking render loop

## Architecture Benefits

✅ Clean separation: data collection ↔ rendering  
✅ Extensible: easy to add new cached state  
✅ Maintainable: clear update logic  
✅ Performant: smart polling, no blocking  

---

_"Like a clean kitchen - everything in its place, nothing wasted."_ 🧹🌲

---
id: 057
date: 2026-01-15
type: future
title: "Workspace Intelligence - Sway X-ray Vision"
status: complete
tags: [rust, tools, v7.3, sway, workspace]
---

## Vision
Know exactly what's happening in your Sway workspaces without switching between them. X-ray vision for your window manager.

**Philosophy:** "Understanding over convenience" - See your workspace state at a glance.

## The Problem
- "What apps are on workspace 2?"
- "Where did I leave that terminal?"
- "Which workspace is active?"
- Switching workspaces just to check what's there

## The Solution
### workspace-view v0.1

**Multiple viewing modes:**
```bash
ws              # Full detailed view
wsa             # Active workspace only
wss             # Compact summary
workspace-view --json   # Machine-readable
```

## Features
- [x] Real-time workspace status
- [x] Window detection with app_id
- [x] Terminal working directories (cwd detection)
- [x] Focused window highlighting
- [x] Window counts per workspace
- [x] Multiple output modes (default, active, summary, json)
- [x] Color-coded active/visible workspaces
- [x] Tree-structured display
- [x] Empty workspace detection

## Output Examples

**Default view:**
```
🌲 Workspace Overview
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Workspace 1 [ACTIVE] 🟢 (2 windows)
  ├─ foot - ~/0-core/rust-tools/workspace-view ●
  └─ brave-browser - Claude - Brave

Workspaces 2-5
  └─ (empty)
```

**Summary view (wss):**
```
1 → 2 windows (foot, brave-browser) [ACTIVE]
2-5 → (empty)
```

**JSON output:**
```json
{
  "num": 1,
  "focused": true,
  "window_count": 2,
  "windows": [
    {"app_id": "foot", "cwd": "~/0-core", "focused": true}
  ]
}
```

## Technical Details
- Uses `swaymsg -t get_tree` for window hierarchy
- Detects terminal working directories via `/proc/{pid}/cwd`
- Finds shell child processes with `pgrep -P`
- Supports path shortening (~ for HOME)
- JSON serialization for scripting integration

## Success Criteria
- [x] CLI tool: `workspace-view`
- [x] Aliases: ws, wsa, wss
- [x] Multiple output modes
- [x] Terminal cwd detection
- [x] Window counts
- [x] Focused window highlighting
- [x] Clean, readable output

## Impact
- Instant workspace awareness
- No more "where did I put that terminal?"
- Perfect for workspace-heavy workflows
- Scriptable with JSON output
- Foundation for future workspace automation

---
_"See all, switch less."_ 🌲

---
id: 047
date: 2026-06-09
type: feature
title: "faelight-menu v2: NixOS upgrade with Pinnacle and MangoWM support"
status: planned
tags: [launcher, menu, wayland, pinnacle, mangowm, rofi, fsh, nixos, rust]
priority: high
---
## Why
faelight-menu v1 was built for Niri.
MangoWM is now the daily driver. Pinnacle is the compositor target.
The menu needs to work under both without compositor-specific hacks.

A launcher is the gateway to the forest.
It should speak forest-native language:
  -- Launch apps by name
  -- Run fsh vocabulary commands directly
  -- Show active intents
  -- Switch forest context

## Vision
  Mod+Space       -- open faelight-menu
  type app name   -- fuzzy match installed apps
  type intent     -- shows active intents, select to focus
  type fsh verb   -- runs fsh vocabulary command
  Esc             -- close without action

Visual:
  Neon candy colors matching forest theme (INT-033)
  Centered floating overlay
  Works under MangoWM and Pinnacle via wlr-layer-shell
  JetBrainsMono Nerd Font at 14px

## What Already Exists
faelight-menu v1: wlr-layer-shell, basic app launcher, Niri-era
faelight-launcher: separate tool, basic fuzzy launch
MangoWM confirmed working on Framework 16
Pinnacle compositor working with Lua config

## Approach
- wlr-layer-shell overlay (compositor-agnostic)
- ratatui or custom renderer for input + results
- App list from XDG desktop entries
- fsh vocabulary list from registry
- Intent list from intents/in-progress/
- Fuzzy match across all three sources
- Keybind registration via compositor config (not hardcoded)

## Phases

Phase 1 -- wlr-layer-shell overlay
  Create faelight-menu v2 crate from scratch
  Overlay window via wlr-layer-shell (works on both compositors)
  Basic text input, Esc to close
  Gate: overlay opens and closes under MangoWM

Phase 2 -- App launching
  Read XDG desktop entries from standard paths
  Fuzzy match on app name and description
  Launch selected app
  Gate: can launch brave, alacritty, fm from menu

Phase 3 -- Forest integration
  Add fsh vocabulary commands as menu entries
  Add active intents as menu entries (prefix: intent:)
  Add vm list entries (prefix: vm:)
  Gate: can run intent list, vm start from menu

Phase 4 -- Pinnacle integration
  Verify overlay works under Pinnacle compositor
  Register Mod+Space keybind in Pinnacle Lua config
  Gate: menu works identically under MangoWM and Pinnacle

Phase 5 -- Neon candy styling
  Apply INT-033 color palette
  JetBrainsMono Nerd Font
  Selection highlight in neon green
  Gate: menu matches forest aesthetic

## Gates
- [ ] wlr-layer-shell overlay opens under MangoWM
- [ ] wlr-layer-shell overlay opens under Pinnacle
- [ ] Fuzzy app launching from XDG desktop entries
- [ ] fsh vocabulary commands appear as menu entries
- [ ] Active intents appear as menu entries (intent: prefix)
- [ ] vm list appears as menu entries (vm: prefix)
- [ ] Mod+Space keybind works under MangoWM
- [ ] Mod+Space keybind works under Pinnacle
- [ ] Neon candy colors match INT-033 palette
- [ ] JetBrainsMono Nerd Font renders correctly

## Depends On
- INT-055 (faelight-compositor bridge) -- shared Wayland layer
- INT-033 (color system) -- neon candy palette
- INT-030 (fsh semantic domains) -- vocabulary as menu entries

## The Rule
"The launcher is the forest's front door.
 It should know the forest's vocabulary
 and speak it back to you." 🌲

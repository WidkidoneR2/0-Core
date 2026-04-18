---
id: 239
title: "faelight-bar v2 -- Modernized, Intelligent, Friday-Aware"
status: planned
date: 2026-04-18
tags: [faelight-bar, bar, modernize, friday, intelligence, ui, wayland, v2]
---
The current bar works. v2 makes it speak.
Three zones. No noise. Friday visible when it matters.
Inspired by the clean pill-badge aesthetic of the Omarchy/Hyprland era --
but grounded in forest intelligence, not generic system stats.
[Left]                    [Center]                     [Right]
🔒 Core: LOCKED    🌲 INT-201 · faelight-term v11    Fri Apr 18 · 14:23
When Friday has a signal (confidence >= 0.85):
🔒 Core: LOCKED    🌲 Friday: 3 deploys without commit    Fri Apr 18 · 14:23
Center returns to active intent after 10 seconds.
Red lock icon + "Core: UNLOCKED" when unlocked -- impossible to miss
Green lock icon + "Core: LOCKED" when locked
No other information -- this zone is a warning system
Default: active intent number + title (truncated cleanly)
When no active intent: forest version + health percentage
When Friday has signal: Friday message (10s display, then returns)
When health drops below 95%: health warning
Clean date format: "Fri Apr 18 · 14:23"
No seconds -- reduce visual noise
Font matches bar aesthetic
Pill/badge style for left and right zones
Subtle separator between zones
Dark background matching forest theme
Faelight visual language -- green for healthy/locked, amber for warning, red for critical
Matches the aesthetic of the old Omarchy bar but with forest intelligence
Reads from faelight-daemon for forest context (active intent, health, Friday signal)
Updates every 5 seconds for intent/health
Updates every 60 seconds for time
Friday signal: reads from synthesis_snapshots and friday_contradictions
Wayland layer-shell protocol via smithay-client-toolkit
⬜ Left zone: core protection status -- red/green lock icon
⬜ Center zone: active intent displayed cleanly
⬜ Center zone: Friday signal displays when confidence >= 0.85
⬜ Center zone: returns to intent after 10 seconds
⬜ Center zone: health warning when < 95%
⬜ Right zone: date and time in clean format
⬜ Pill/badge visual style matching forest aesthetic
⬜ Updates every 5s for forest state, 60s for time
⬜ Reads from faelight-daemon IPC
⬜ No visual corruption on workspace switch
⬜ Performance: < 1% CPU usage idle
⬜ Replaces current faelight-bar as primary bar
"The bar is Friday's face on the desktop.
Three zones. Forest intelligence made visible." 🌲

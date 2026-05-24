---
id: 337
title: "Study -- Pinnacle compositor Smithay patterns for faelight-compositor v3"
status: planned
date: 2026-05-25
tags: [study, pinnacle, smithay, compositor, wayland, lua, rust, awesomewm]
---

## What Is Pinnacle

Pinnacle (https://github.com/pinnacle-comp/pinnacle) is a Smithay-based Wayland
compositor inspired by AwesomeWM. Configured in Lua or Rust. Actively maintained.

This is the most relevant external project to faelight-compositor because:
1. Same foundation: Smithay
2. Same goal: custom Wayland compositor in Rust
3. Further along: they have solved problems we are still working through
4. AwesomeWM inspiration: keyboard-driven, programmable -- aligns with Forest Candy + i3 vision

## Why Study It

faelight-compositor v2 is running on real hardware. The next step (INT-323,
faelight-compositor v3) is full session authority -- replacing Niri permanently.

Pinnacle has already solved:
- Full XDG shell protocol implementation
- Window management with tiling and floating layouts
- Lua configuration API (reconfigure without recompile)
- Input handling and keybind system
- Multi-monitor support
- Layer shell (for bar, notifications)

Studying Pinnacle prevents us from reinventing what they have already proven.

## What To Study

1. **How Pinnacle structures state** -- the main compositor state struct and
   how it organizes windows, outputs, inputs
2. **The Lua API design** -- how they expose compositor behavior to configuration
   without sacrificing safety
3. **Window management** -- tiling algorithm, focus management, workspace model
4. **Input handling** -- how keybinds are registered and dispatched
5. **Layer shell implementation** -- how faelight-bar would run as a layer surface
6. **VBlank and frame timing** -- how they handle the render loop
7. **Multi-output** -- how they handle multiple monitors

## What We Build (After Study)

faelight-compositor v3 gains:
- Stable window management (tiling + floating, forest-colored borders)
- Layer shell for faelight-bar and faelight-notify
- Keybind system configurable from state.db (not hardcoded)
- Multi-monitor awareness
- VT switch stability

The Lua configuration approach is interesting but not forest-aligned.
The forest configures through state.db and fsh commands, not Lua scripts.
However the API design pattern -- separating policy from mechanism -- is worth borrowing.

## Gates

⬜ Pinnacle source cloned and studied -- architecture documented in docs/pinnacle-patterns.md
⬜ State struct design understood -- how Pinnacle organizes compositor state
⬜ Window management algorithm documented -- tiling, focus, workspace model
⬜ Layer shell implementation understood -- how bar/notify surfaces work
⬜ Input and keybind system documented -- registration and dispatch pattern
⬜ VBlank and frame timing pattern documented
⬜ Multi-output pattern documented
⬜ At least 3 patterns directly applied to faelight-compositor v3
⬜ faelight-compositor v3 scaffolded with Pinnacle-informed architecture
⬜ Layer shell working -- faelight-bar runs as layer surface inside compositor
⬜ Keybind system wired to state.db -- forest configures via commands not hardcode

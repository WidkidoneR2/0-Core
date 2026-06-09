---
id: 052
date: 2026-06-09
type: feature
title: "MangoWM: daily driver configuration, keybinds, and autostart"
status: planned
tags: [mango, compositor, wayland, keybinds, autostart]
priority: high
---
## Why
MangoWM is now the primary compositor. Faster than niri, dwl-based,
proven working on Framework 16. Needs proper configuration as daily driver.

## Work Items
- Complete keybind mapping for all forest tools
- Autostart faelight-notify, faelight-bar-v2 when ready
- Workspace rules for common apps
- Trackpad tuning for Framework 16
- Lock screen integration
- Move config to users/christian/mango.nix properly

## Gate
- [ ] All keybinds working and documented
- [ ] Autostart working for forest services
- [ ] Config tracked in users/christian/mango.nix
- [ ] Run as daily driver for 2 weeks

## Notes
- broot: needs Faelight Forest neon candy colors theme
- bind=SUPER,e for broot

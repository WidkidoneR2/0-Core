---
id: 052
date: 2026-06-09
type: feature
title: "MangoWM: daily driver configuration, keybinds, and autostart"
status: planned
tags: [mango, compositor, wayland, keybinds, autostart, framework, dwl]
priority: high
---
## Why
MangoWM is confirmed working on Framework 16 as of 2026-06-10.
It is faster than Niri and Pinnacle, dwl-based, Wayland-native.
It is the daily driver but config is not yet fully formalized.

Keybinds are partially set. Autostart is not complete.
Config is not fully tracked in the Nix flake.
This intent locks in MangoWM as the daily driver properly.

## What Already Exists
MangoWM installed and working on Framework 16
greetd tuigreet launches MangoWM via --cmd mango (commit b0b397dd)
Alacritty launches on Mod+Return
Basic keybinds working
Config partially in users/christian/mango.nix

## Vision
MangoWM fully configured as daily driver:
  All forest tool keybinds mapped and documented
  Autostart: faelight-bar, faelight-notify, pipewire, NetworkManager
  Workspace rules for common apps
  Trackpad tuned for Framework 16
  Lock screen via faelight-lock v2 (INT-046)
  Config fully tracked in users/christian/mango.nix
  Run as daily driver for 2 weeks without issues

## Keybind Map (target)
  Mod+Return       -- alacritty
  Mod+Alt+Return   -- faelight-ade
  Mod+Space        -- faelight-menu v2 (INT-047)
  Mod+/            -- faelight-cheatsheet (Ctrl+/)
  Mod+e            -- faelight-fm
  Mod+g            -- faelight-git
  Mod+d            -- d (health check in terminal)
  Mod+l            -- faelight-lock v2 (INT-046)
  Mod+Shift+q      -- close window
  Mod+Shift+e      -- exit MangoWM
  Mod+1..9         -- switch workspace
  Mod+Shift+1..9   -- move window to workspace
  Mod+f            -- fullscreen
  Mod+Shift+f      -- floating toggle

## Autostart Services
  faelight-notify  -- notification daemon
  faelight-bar     -- status bar (INT-053, when ready)
  pipewire         -- audio
  NetworkManager   -- network (already running as systemd service)
  Mullvad          -- VPN (already running as systemd service)

## Phases

Phase 1 -- Keybind audit and completion
  Document all current keybinds from mango.nix
  Map missing forest tool keybinds
  Test all keybinds in live session
  Gate: all keybinds in table above working and documented

Phase 2 -- Autostart hardening
  Verify faelight-notify autostarts cleanly
  Add faelight-bar autostart (when INT-053 complete)
  Gate: all autostart services running after login

Phase 3 -- Trackpad tuning
  libinput config for Framework 16 trackpad
  Natural scroll, tap-to-click, palm detection
  Gate: trackpad behaves correctly for daily use

Phase 4 -- Nix config cleanup
  All config in users/christian/mango.nix
  No hardcoded paths, no manual config files
  Gate: fresh rebuild produces identical MangoWM config

Phase 5 -- Daily driver validation
  Run MangoWM as sole compositor for 2 weeks
  No compositor switches, no fallback to Niri
  Log any issues to state.db
  Gate: 2 weeks daily driver with no blocking issues

## Gates
- [ ] All keybinds in target map working
- [ ] Keybind cheatsheet updated (INT-260)
- [ ] faelight-notify autostarts on login
- [ ] Trackpad natural scroll and tap-to-click working
- [ ] Framework 16 palm detection tuned
- [ ] All config in users/christian/mango.nix
- [ ] Fresh rebuild produces correct MangoWM config
- [ ] 2 weeks daily driver with no blocking issues
- [ ] faelight-bar autostart ready (pending INT-053)

## Depends On
- INT-053 (faelight-bar v2) -- bar autostart
- INT-046 (faelight-lock v2) -- lock screen keybind
- INT-047 (faelight-menu v2) -- Mod+Space launcher

## The Rule
"MangoWM is not a temporary measure.
 It is the forest's window to the world.
 Configure it properly.
 Run it daily.
 Trust it completely." 🌲

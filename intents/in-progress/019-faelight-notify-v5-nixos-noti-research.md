---
id: 019
date: 2026-06-03
type: feature
title: "faelight-notify v5: NixOS-native, noti research, layer-shell ready"
status: in-progress
tags: [faelight-notify, nixos, noti, layer-shell, pinnacle, wayland, colors]
priority: medium
---
## Why
faelight-notify v4 works on NixOS but has two gaps:
1. render.rs uses old Arch-era colors -- not INT-033 neon candy palette
2. No forest-aware notifications -- health drops, intent completions,
   Friday events are not surfaced as system notifications

## Research: noti
noti (github.com/variadico/noti) is a SENDER tool -- it wraps a command
and sends a notification when it finishes. It is not a daemon.
faelight-notify is a DAEMON -- it receives and displays notifications.
They serve different purposes. noti patterns already absorbed:
- fsh already notifies on commands > 30 seconds (main.rs line 2616)
- D-Bus org.freedesktop.Notifications is the right interface
- No noti code needs adopting -- architecture is already correct

## What v4 Already Has
- org.freedesktop.Notifications D-Bus compliant (dbus.rs)
- wlr-layer-shell Wayland overlay (main.rs)
- fontdue text renderer (render.rs)
- Urgency levels: low/normal/critical
- Broken pipe graceful exit (INT-019 fix)
- Stderr silenced (nix::libc::dup2 fix)
- Works under MangoWM and Pinnacle (wlr-layer-shell agnostic)

## What v5 Adds

1. Neon candy colors (INT-033 palette)
   render.rs currently uses old Arch-era greens
   Update to match INT-033 semantic tokens:
   BG: forest night (0x0f, 0x14, 0x11)
   BORDER_NORMAL: neon green (57, 255, 20)
   BORDER_CRITICAL: neon red (255, 80, 80)
   BORDER_LOW: muted green (100, 180, 100)
   TEXT_SUMMARY: neon green (57, 255, 20)
   TEXT_BODY: fog white (215, 224, 218)

2. Forest-aware notifications
   Subscribe to forest events via state.db
   Notify on: intent complete, health drop below 90,
              Friday new insight, VM snapshot complete
   fsh sends these via notify-send or direct D-Bus call

3. NixOS font path
   render.rs hardcodes assets/fonts/HackNerdFont-Regular.ttf
   NixOS binary has different path -- use pkgdatadir or env var

## Phases

Phase 1 -- Neon candy colors (safe, no system impact)
  Update render.rs color constants to INT-033 palette
  Commit, rebuild, verify notification colors
  Gate: notifications show neon candy colors

Phase 2 -- NixOS font path
  Fix hardcoded font path for NixOS derivation
  Gate: font loads correctly in NixOS build

Phase 3 -- Forest-aware notifications
  Add forest event subscription to fsh
  Gate: intent complete fires desktop notification

## Gates
- [x] Broken pipe graceful exit
- [x] Stderr silenced -- no terminal noise
- [x] Works under MangoWM (wlr-layer-shell)
- [x] Neon candy colors match INT-033 palette
- [x] Font path works in NixOS derivation
- [ ] Intent complete fires desktop notification
- [ ] Health drop below 85 fires desktop notification
- [ ] Integrity drop below 80 fires desktop notification
- [x] noti research documented (done above -- not applicable)

## The Rule
"The forest speaks to itself through notifications.
 Health drops, intent wins, Friday insights --
 all surfaced without you having to ask." 🌲

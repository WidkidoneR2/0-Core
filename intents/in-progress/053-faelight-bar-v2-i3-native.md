---
id: 053
date: 2026-06-09
type: feature
title: "faelight-bar v2: i3-style wlr-layer-shell bar for MangoWM and Pinnacle"
status: in-progress
tags: [bar, wayland, layer-shell, mango, pinnacle, rust, ratatui, cosmic-text]
priority: high
---
## Why
faelight-bar v1 was a Niri prototype built around Niri-specific IPC.
MangoWM is now the daily driver. Pinnacle is the compositor target.
v1 does not work under MangoWM or Pinnacle.

faelight-bar v2 is built compositor-agnostic from the ground up:
wlr-layer-shell only, no compositor-specific IPC.
It reads forest state from state.db and /proc -- not from the compositor.

## What Already Exists
faelight-bar v1: cosmic-text renderer, wlr-layer-shell, health/git/intent display
INT-033: neon candy palette applied to v1 (green/amber/red/purple)
MangoWM confirmed working on Framework 16
Pinnacle confirmed working with Lua config

## Vision
  Top bar, anchored via wlr-layer-shell
  Left:   🔒 lock state · H:95% health · branch* git
  Center: active intent title (neon purple) or Friday message (neon cyan)
  Right:  CPU% · RAM% · battery% · wifi · clock

  Colors:
    health >= 95  → neon green
    health >= 80  → neon amber
    health < 80   → neon red
    active intent → neon purple
    friday msg    → neon cyan
    git dirty     → neon amber
    git clean     → neon green

  Updates every 2 seconds from state.db and /proc
  No compositor IPC dependency -- reads forest state directly

## Approach
- Build on faelight-bar v1 codebase (cosmic-text + wlr-layer-shell)
- Replace Niri IPC workspace reader with MangoWM/Pinnacle-agnostic source
- Workspace indicators from wlr-foreign-toplevel-management protocol
- System stats from /proc/stat, /proc/meminfo, /sys/class/power_supply
- Forest state from state.db (health, intent, Friday)
- Git state from .git/HEAD + git status --porcelain

## Pre-flight (INT-056 required)
Any compositor-touching change must pass INT-056 pre-flight:
TTY2 hardened, fallback session defined, VM tested first.

## Phases

Phase 1 -- Strip Niri IPC, verify wlr-layer-shell baseline
  Remove all Niri-specific code from v1
  Verify wlr-layer-shell overlay renders under MangoWM
  Gate: blank bar renders under MangoWM, does not crash

Phase 2 -- Forest state display
  Health from state.db with neon candy colors (INT-033)
  Active intent from state.db
  Git branch and dirty state from .git/HEAD
  Gate: left and center sections show correct forest data

Phase 3 -- System stats
  CPU from /proc/stat (delta calculation)
  RAM from /proc/meminfo
  Battery from /sys/class/power_supply
  Wifi SSID from iwctl or /proc/net/wireless
  Clock: HH:MM format
  Gate: right section shows live system stats

Phase 4 -- Pinnacle verification
  Test bar under Pinnacle compositor
  Gate: bar renders identically under Pinnacle

Phase 5 -- Autostart integration
  Add faelight-bar to MangoWM autostart in mango.nix
  Add faelight-bar to Pinnacle autostart in Lua config
  Gate: bar autostarts on compositor launch

## Gates
- [ ] INT-056 pre-flight complete before any live compositor work
- [ ] wlr-layer-shell overlay renders under MangoWM
- [ ] wlr-layer-shell overlay renders under Pinnacle
- [ ] Health shows with neon candy semantic colors
- [ ] Active intent shows in neon purple
- [ ] Git state shows branch and dirty indicator
- [ ] CPU, RAM, battery, wifi, clock all rendering
- [ ] Updates every 2 seconds without flicker
- [ ] Autostart in MangoWM config
- [ ] Autostart in Pinnacle config
- [ ] No Niri-specific code remaining

## Depends On
- INT-056 (Forest Recovery Protocol) -- pre-flight gate
- INT-055 (compositor bridge) -- shared layer-shell infrastructure
- INT-033 (color system) -- neon candy palette already applied

## The Rule
"The bar is the forest's pulse.
 It should show the forest's health at a glance --
 not the compositor's internal state." 🌲

## Pre-flight Gate -- INT-056 (Forest Recovery Protocol)
This intent changes the login/compositor surface. Per INT-056, NOTHING
here lands on the real machine until it has passed the pre-flight
checklist in INT-024's VM:
  [ ] change tested in VM via INT-024 pipeline
  [ ] VM snapshot taken before test (before-INT-NNN)
  [ ] TTY2 verified reachable in VM
  [ ] greetd fallback session verified in VM
  [ ] recovery from a broken session demonstrated in VM
  [ ] all of the above documented before graduating
Door is always open: docs/recovery-runbook.md · TTY2 via Ctrl+Alt+F2.

## Known Issue -- in progress (2026-06-14)
faelight-bar v3 exits with rc=1 "Io error: Broken pipe (os error 32)" after
~8 min under MangoWM: the compositor closes the Wayland connection and the next
eq.flush()? in main() propagates the IoError and exits. Confirmed NOT memory
(RSS flat ~34 MB across deaths) and NOT a panic.
Workaround in place: supervised auto-restart loop respawns the bar in ~2s,
mirroring a deployed systemd service with Restart=always.
TODO next session: run with WAYLAND_DEBUG=1, read the compositor's fatal error
just before the broken pipe, decide bar-protocol-bug vs Mango behavior, then
either fix the bar or have main() reconnect instead of exiting.
Status: in-progress. Do not close until the connection drop is understood.

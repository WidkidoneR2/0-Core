---
id: 055
date: 2026-06-09
type: architecture
title: "faelight-compositor: shared Wayland bridge for MangoWM and Pinnacle"
status: planned
tags: [compositor, wayland, layer-shell, mango, pinnacle, smithay, rust, ipc]
priority: medium
---
## Why
faelight-compositor is a custom Smithay-based Wayland compositor
built over 6 months. It has deep protocol coverage:
xdg-shell, layer-shell, wlr-foreign-toplevel, IPC.

Rather than retiring it when MangoWM became daily driver,
it becomes infrastructure: the shared Wayland bridge beneath
MangoWM and Pinnacle.

faelight-bar v2, faelight-notify v5, faelight-lock v2 all need:
  -- layer-shell protocol (rendering overlays)
  -- IPC (reading compositor state)
  -- Protocol abstractions (not reimplementing per compositor)

One implementation serves all tools and both compositors.

## What Already Exists
faelight-compositor: Smithay 0.7.0, working on Framework 16
Phases 1-3 complete: XDG protocols, fsh + faelight-term inside compositor,
auto-tiling, layer-shell handlers
faelight-core: shared library crate for all forest tools
MangoWM: running as daily driver, honors wlr-layer-shell
Pinnacle: installed, honors wlr-layer-shell

## Vision
faelight-compositor transitions from standalone compositor to bridge library:
  faelight-core/src/wayland/ -- shared protocol implementations
  faelight-bar v2 imports layer-shell from faelight-core
  faelight-notify v5 imports layer-shell from faelight-core
  faelight-lock v2 imports layer-shell from faelight-core
  IPC socket at /run/user/1000/faelight-compositor.sock
  MangoWM session starts bridge first, registers compositor type
  Pinnacle session starts bridge first, registers compositor type
  Forest tools connect to bridge -- not directly to compositor

## Pre-flight (INT-056 required)
Any compositor change must pass INT-056 pre-flight:
TTY2 hardened, fallback session, VM-tested first.

## Phases

Phase 1 -- Audit
  Read faelight-compositor/src/handlers/ -- document protocol coverage
  Identify what faelight-bar v1 needed vs what is available
  Identify what faelight-notify v4 uses vs what layer-shell provides
  Output: docs/compositor-bridge-audit.md
  Gate: audit doc written, gaps identified

Phase 2 -- Extract layer-shell to faelight-core
  Move layer-shell implementation from faelight-compositor to faelight-core
  faelight-bar v2 imports from faelight-core
  faelight-notify v5 imports from faelight-core
  Gate: both tools build using shared layer-shell from faelight-core

Phase 3 -- IPC socket
  faelight-compositor exposes Unix socket: /run/user/1000/faelight.sock
  Events: compositor_type, workspace_change, window_focus, health_update
  faelight-bar v2 subscribes to workspace events via socket
  Gate: IPC socket working under MangoWM

Phase 4 -- Compositor profiles
  MangoWM session: start bridge, register compositor=mango
  Pinnacle session: start bridge, register compositor=pinnacle
  Forest tools read compositor type from bridge
  Gate: forest tools detect compositor type correctly

Phase 5 -- Pinnacle verification
  All phases verified under Pinnacle
  Gate: IPC socket working under Pinnacle

## Gates
- [ ] INT-056 pre-flight complete before any compositor work
- [ ] docs/compositor-bridge-audit.md written
- [ ] layer-shell extracted to faelight-core
- [ ] faelight-bar v2 uses faelight-core layer-shell
- [ ] faelight-notify v5 uses faelight-core layer-shell
- [ ] IPC socket working under MangoWM
- [ ] IPC socket working under Pinnacle
- [ ] Compositor type detectable from bridge
- [ ] Forest tools connect via bridge not direct compositor calls

## Depends On
- INT-056 (Forest Recovery Protocol) -- pre-flight gate
- INT-053 (faelight-bar v2) -- first consumer of bridge
- INT-019 (faelight-notify v5) -- second consumer of bridge

## The Rule
"faelight-compositor is not retired.
 It becomes the forest's Wayland nervous system.
 One implementation. Every tool benefits." 🌲

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

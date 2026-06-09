---
id: 055
date: 2026-06-09
type: architecture
title: "faelight-compositor: shared Wayland bridge for MangoWM and Pinnacle"
status: planned
tags: [compositor, wayland, layer-shell, mango, pinnacle, smithay, rust]
priority: medium
---
## Why
faelight-compositor is a custom Smithay-based Wayland compositor built
over 6 months. Rather than retiring it, it becomes the shared protocol
and IPC layer beneath MangoWM and Pinnacle. One implementation serves both.

## Vision
faelight-compositor transitions from standalone compositor to a Wayland
bridge library -- providing layer-shell, IPC, and protocol implementations
that MangoWM and Pinnacle consume. faelight-bar v2, faelight-notify v5,
and faelight-lock v2 all talk to this bridge instead of each compositor
separately.

## Achievable Goals (using existing tools)

### Phase 1 -- Audit (faelight-compositor + faelight-bar + faelight-notify)
- Read faelight-compositor/src/handlers/ to understand current protocol coverage
- Document which protocols are implemented: xdg-shell, layer-shell, wlr-foreign-toplevel
- Identify what faelight-bar v1 needed vs what is available
- Identify what faelight-notify v4 uses vs what layer-shell provides
- Output: docs/compositor-bridge-audit.md

### Phase 2 -- Extract layer-shell as shared crate
- Move layer-shell implementation from faelight-compositor into faelight-core
- faelight-bar v2 imports from faelight-core instead of reimplementing
- faelight-notify v5 imports from faelight-core instead of reimplementing
- Verify both MangoWM and Pinnacle honor wlr-layer-shell protocol

### Phase 3 -- IPC bridge
- faelight-compositor exposes a Unix socket for forest tool communication
- faelight-bar v2 subscribes to workspace events via socket
- faelight-notify v5 sends notifications via socket
- Friday can query compositor state via socket

### Phase 4 -- Compositor profiles
- MangoWM session starts faelight-compositor bridge first
- Pinnacle session starts faelight-compositor bridge first
- Both compositors register with the bridge on startup
- Forest tools connect to bridge, not directly to compositor

## Dependencies
- faelight-compositor (existing -- Smithay 0.7.0)
- faelight-core (existing -- shared library crate)
- faelight-bar v2 (INT-053)
- faelight-notify v5 (INT-019)
- MangoWM (pkgs.mangowc -- running)
- Pinnacle (flake input -- installed)

## Gate
- [ ] Audit complete -- docs/compositor-bridge-audit.md written
- [ ] layer-shell extracted to faelight-core
- [ ] faelight-bar v2 uses faelight-core layer-shell
- [ ] faelight-notify v5 uses faelight-core layer-shell
- [ ] IPC socket working under MangoWM
- [ ] IPC socket working under Pinnacle
- [ ] Forest tools connect via bridge not direct compositor calls

## Note
This does not require replacing MangoWM or Pinnacle.
Both compositors remain. faelight-compositor becomes infrastructure,
not a daily driver compositor.

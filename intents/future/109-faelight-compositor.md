---
id: 109
date: 2026-02-15
type: future
title: "faelight-compositor — Rust Wayland Compositor on Smithay"
status: in-progress
tags: [compositor, wayland, smithay, drm, rust, architecture, v11]
version: 11.0.0
priority: high
---

## Vision

The last sibling comes home.

Every other tool in Faelight Forest was built from scratch in Rust.
The compositor was borrowed — first Sway, then Niri.
faelight-compositor ends that. A Wayland compositor built on Smithay,
native to the forest, aware of its own ecosystem.

## Philosophy

Every other compositor is substrate.
faelight-compositor is a participant.

It doesn't just manage windows — it knows it's part of a forest.
Every window open, every focus change, every workspace switch
flows into state.db. The compositor is a ledger participant.

## Current Status (v0.1.0)

- ✅ Smithay workspace dependency
- ✅ FaelightCompositor state struct
- ✅ Protocol handlers: Compositor, XdgShell, Seat, DataDevice, Output
- ✅ Event emission wired (window.focus, window.open)
- ✅ Winit backend — first frame rendered (forest green #11140f)
- ✅ Events writing to state.db
- ✅ DRM/udev backend — libseat session, libinput, real hardware
- ✅ Input handling wired
- ✅ 43/43 path resilience — compositor fully deployed
- ✅ fc alias (winit) and fc --drm alias (real hardware)

## DRM Rendering — 8 Session Plan

The path to pixels on real hardware. Each session has a clear
goal and a "fun" side piece so no session ends empty-handed.

### Session 1 — Read & Understand (no coding)
Goal: Full comprehension of device_added in anvil/src/udev.rs
- Read device_added (lines 763-865) completely
- Read connector_connected completely  
- Read render function completely
- Document every dependency and its purpose
- Write notes in this intent
Fun: Add a forest quote system to faelight-fetch

### Session 2 — Open DRM Device
Goal: Open GPU device file, create DrmDeviceFd
- session.open() with correct OFlags
- DrmDevice::new() from fd
- GbmDevice::new() from fd clone
- Wire DrmEvent::VBlank handler
- Test: device opens without error
Fun: faelight-login visual polish

### Session 3 — GPU Renderer
Goal: EGL context and GpuManager initialized
- EGLDisplay::new() from GBM device
- EGLDevice::device_for_display()
- GlesRenderer::new() with EGL context
- GpuManager::add_node() for render node
- Test: renderer created, no EGL errors
Fun: core audit Phase 2 — expected_usage in tools.toml

### Session 4 — Connector Scanning
Goal: Detect monitors and read their modes
- DrmScanner::new() and scan_connectors()
- connector_connected() implementation
- Read connector name (HDMI-1, eDP-1, etc.)
- Read preferred mode (resolution + refresh)
- Test: prints monitor name and resolution to log
Fun: doctor gains optional audit score summary line

### Session 5 — Output Creation
Goal: Wayland output object registered
- DrmOutputManager construction
- Output::new() with physical properties
- Mode registration and output global
- Test: output registered, wlr-randr sees it
Fun: faelight-fetch shows audit health score

### Session 6 — First Frame ⭐ THE MILESTONE
Goal: Forest green pixels on real hardware
- Render loop wired to VBlank
- GlesRenderer clears to #11140f
- DrmOutput::queue_frame()
- Switch to TTY, run fc --drm
- See forest green fill the screen
Fun: core story captures the milestone

### Session 7 — Surface Rendering
Goal: Wayland clients connect and display windows
- WaylandSurfaceRenderElement integration
- Damage tracking with OutputDamageTracker
- weston-simple-shm renders inside compositor
- Test: a window appears on screen
Fun: core audit Phase 3 — core advise integration

### Session 8 — Polish & Stability
Goal: Production-ready DRM backend
- VT switching (pause/resume session)
- Proper cleanup on exit
- doctor health check integration
- faelight-bar compatibility
- Test: switch VTs and back, compositor recovers
Fun: v10.8.0 release — "The Compositor Wakes"

## Gate Check

- ✅ Smithay added to workspace dependencies
- ✅ FaelightCompositor state struct complete
- ✅ Protocol handlers implemented
- ✅ Event emission wired (window.focus, window.open)
- ✅ Winit backend — first frame rendered
- ✅ Binary deployed — 43/43 path resilience
- ✅ Events writing to state.db
- ✅ DRM/udev backend — libseat + libinput on real hardware
- ✅ Session 2 — DRM device enumeration, AMD Radeon 780M identified
  - GPU: AMD Radeon 780M (radeonsi, phoenix, ACO)
  - DRM: 3.64, Mesa: 26.0.2
  - Devices: /dev/dri/card1, card2, renderD128, renderD129
  - DrmDevice code written and compiles correctly
  - CRITICAL: must run from TTY2, not inside Niri
    (Niri owns libseat — running --drm from within Niri causes session conflict)
  - Session 3: Switch to TTY2, run --drm, get DRM device opened log
- ✅ Session 3 — DRM device opened from TTY2
  - card2: 9 connectors, 4 CRTCs (main GPU — AMD Radeon 780M)
  - card1: 2 connectors, 4 CRTCs
  - Error: "Failed to restore previous state" — expected, Niri owns atomic state
  - Compositor reached ready state successfully
  - Session 4: Select correct CRTC/connector, create GBM device, attempt first render
- ⬜ Session 4 — GBM device, connector selection, first render attempt ← NEXT
- ⬜ Session 3 — GPU renderer initialized
- ⬜ Session 4 — Connector scanning
- ⬜ Session 5 — Output creation
- ⬜ Session 6 — First frame on real hardware ⭐
- ⬜ Session 7 — Surface rendering
- ⬜ Session 8 — Polish and stability
- ⬜ Replaces Niri as primary compositor

## Stats Context
```
System:    v10.7.0 — The Forest Remembers
Health:    100% (22 checks)
Commits:   1417
Tools:     52 custom Rust binaries
Backend:   DRM initialized, input enumerated, events flowing
```

## The Phrase

**"Every other compositor is substrate.
faelight-compositor is a participant."**

*"The last sibling comes home — and the forest becomes complete."* 🌲

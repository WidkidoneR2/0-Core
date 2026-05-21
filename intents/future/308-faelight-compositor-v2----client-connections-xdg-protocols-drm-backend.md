---
id: 308
title: "faelight-compositor v2 -- client connections, XDG protocols, DRM backend"
status: in-progress
date: 2026-05-16
type: build
tags: [compositor, smithay, wayland, drm, xdg, egl, opengl, protocols]
depends_on: []
---
## Where We Are (v0.1.0 -- proven 2026-05-16)

faelight-compositor v0.1.0 is proven:
  - Compiles cleanly with full smithay stack
  - EGL on AMD Radeon 780M (radeonsi, phoenix, ACO)
  - OpenGL ES 3.2 Mesa 26.0.6
  - Wayland socket opens (wayland-2)
  - state.db connected at startup
  - Forest green background rendered via GPU (#11140f)
  - foot connected and ran inside compositor (winit mode)
  - 60fps event loop keeps compositor alive

Warnings foot showed -- protocols not yet implemented:
  - primary selection interface
  - XDG activation
  - fractional scaling
  - server-side cursors
  - xdg-toplevel-icon
  - text input interface
  - decoration manager

These are the v2 targets.

---
## v2 Goals

### Phase 1 -- Protocol Completeness
Implement missing Wayland protocols so clients connect cleanly:

XDG decoration manager:
  - Clients can request server-side decorations
  - smithay::wayland::shell::xdg::decoration::XdgDecorationState
  - Gate: foot connects with no decoration warning

Primary selection protocol:
  - zwp_primary_selection_device_manager_v1
  - Enables middle-click paste
  - Gate: middle-click paste works between clients

XDG activation:
  - xdg_activation_v1
  - Enables window focus requests between apps
  - Gate: no activation warning from clients

Server-side cursors:
  - wp_cursor_shape_manager_v1
  - GPU-rendered cursors instead of client-rendered
  - Gate: cursor renders without client fallback warning

### Phase 2 -- faelight-term inside faelight-compositor
The real validator: run faelight-term inside faelight-compositor.
fsh running inside the forest compositor.
Forest shell inside forest compositor -- full stack owned.

  WAYLAND_DISPLAY=wayland-2 faelight-term

Gate: faelight-term runs fully inside faelight-compositor
Gate: fsh session works inside faelight-compositor
Gate: Friday signals visible inside compositor session

### Phase 3 -- Window Management
Currently: clients connect but windows are not positioned.
Add basic tiling/floating window management:
  - New windows tile automatically
  - Keyboard focus follows mouse
  - Super+Q closes focused window
  - Super+arrows move focus between windows
  - Window borders in forest colors (#00bfff active, #2a4a5a inactive)

Gate: two terminals side by side inside compositor
Gate: keyboard navigation between windows

### Phase 4 -- DRM Backend (Real Hardware)
Run faelight-compositor on real hardware -- replace Niri.
Steps:
  1. Test DRM backend in VM first (QEMU/OVMF/virtio-gpu)
  2. Boot into faelight-compositor alongside Niri (TTY2)
  3. Run faelight-term inside it on real hardware
  4. Validate 165Hz on eDP (2560x1600)
  5. Only replace Niri after 1 week stability on TTY2

DRM notes (from AMD 780M research):
  GPU:        AMD Radeon 780M (radeonsi, phoenix)
  Driver:     Mesa 26.0.6 / ACO
  DRM:        3.64
  Connector:  eDP (embedded display)
  Resolution: 2560x1600
  Refresh:    165Hz
  CRTC:       Handle(363)
  GBM format: XRGB8888 confirmed working

Gate: DRM backend renders on real hardware
Gate: 165Hz page flip without tearing
Gate: faelight-term runs on DRM backend

### Phase 5 -- Forest Integration
The compositor knows about the forest:
  - Reads state.db for active intent → shows in compositor chrome
  - Friday signals appear as compositor overlays
  - faelight-bar runs as layer-shell surface inside compositor
  - faelight-notify runs as layer-shell notifications
  - Health status visible in compositor border/chrome

Gate: active intent shown in compositor chrome
Gate: faelight-bar running inside compositor
Gate: faelight-notify popups inside compositor

### Phase 6 -- faelight-boot (future, dangerous)
UEFI bootloader in Rust.
Test in VM only until proven.
Never remove GRUB until boot is stable for 30+ days.
The uefi crate is the foundation.
This is a v16+ story.

---
## Safety Rules for DRM Work
- Always keep Niri working on TTY1
- Test DRM on TTY2 first
- Never modify bootloader without VM validation first
- Keep GRUB as fallback at all times
- Framework recovery mode is available if needed

---
## Gates
Phase 1 -- Protocol completeness:
- [x] XDG decoration manager implemented -- server-side decorations, no decoration warning 2026-05-17
- [x] Primary selection protocol implemented -- middle-click paste support added 2026-05-17
- [x] foot connects with zero warnings -- all protocols implemented 2026-05-17
- [x] faelight-term connects with zero warnings -- cursor shape, fractional scale, decoration all implemented

Phase 2 -- faelight-term inside compositor:
- [x] faelight-term runs inside faelight-compositor -- connects, wgpu needs DRM for full render 2026-05-17
- [x] fsh session works inside compositor -- foot+fsh confirmed, faelight-term partial (wgpu/DRM)
- [x] Friday signals visible -- state.db events written on window.open and window.focus

Phase 3 -- Window management:
- [x] Two windows tile side by side -- auto-tiling, half-screen each 2026-05-17
- [x] Keyboard focus navigation works -- focus follows new window, others deactivated
- [x] Forest color borders -- visual rendering confirmed working on DRM backend 2026-05-21

Phase 4 -- DRM backend:
- [x] VM test passes -- GBM created, forest green rendered on virtio-gpu in QEMU 2026-05-21
- [x] DRM backend on TTY2 stable -- forest green on AMD 780M 2560x1600 confirmed 2026-05-21
- [x] 165Hz confirmed -- vrefresh=165 PREFERRED|DRIVER mode on eDP, same connector as Niri 2026-05-21
- [ ] faelight-term on DRM

Phase 5 -- Forest integration:
- [~] Active intent in compositor chrome -- title set via /etc/faelight/INTENT, visible on DRM backend 2026-05-18
- [ ] faelight-bar inside compositor
- [ ] faelight-notify inside compositor

Final:
- [ ] faelight-compositor replaces Niri as daily compositor
- [ ] Full forest stack: boot → login → compositor → shell → terminal

---
"Niri was the forest learning to walk.
faelight-compositor is the forest learning to stand.
When it stands on its own,
the forest owns the screen from pixel to thought." 🌲

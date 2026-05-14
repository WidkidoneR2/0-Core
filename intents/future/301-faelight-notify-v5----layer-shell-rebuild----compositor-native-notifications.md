---
id: 301
title: "faelight-notify v5 -- Layer-Shell Rebuild -- Compositor-Native Notifications"
status: planned
date: 2026-05-14
type: arch
tags: [faelight-notify, layer-shell, cosmic-text, wgpu, wayland, notifications]
depends_on: [287]
---
## The Problem
faelight-notify v4 works but it is not compositor-native.
It uses xdg-shell surfaces -- the same surface type as application windows.
This means:
  - No guaranteed z-ordering above everything
  - No blur or transparency from the compositor
  - No gesture dismissal
  - Niri cannot treat notifications as a distinct layer
  - Positioning is approximate, not monitor-aware

A notification daemon that uses xdg-shell is fundamentally limited.
The correct protocol is zwlr_layer_shell_v1.
Niri handles layer-shell exceptionally well.
This is the rebuild that makes notifications feel native.

---
## The Vision
Notifications that feel like they belong to the system.
Not windows pretending to be notifications.
Actual compositor-native overlay surfaces that:
  - Appear above everything, always
  - Anchor to monitor edges correctly on multi-monitor setups
  - Animate in/out under compositor control
  - Support blur and transparency (Niri handles this natively)
  - Dismiss on gesture or click
  - Stack properly when multiple arrive

Friday-aware notifications:
  - Deploy completed: bar pulses, notification appears, Friday signal fires
  - Long command finished: notification shows command name + duration
  - Health check warning: notification uses warning color from color DNA
  - Intent completed: brief celebration -- the forest acknowledges its own work

---
## Architecture
### Stack
  smithay-client-toolkit -- Wayland surface management (already in faelight-term)
  zwlr_layer_shell_v1 -- layer-shell protocol for overlay surfaces
  cosmic-text + glyphon -- text rendering (shared pattern with faelight-term v3)
  wgpu -- GPU rendering pipeline (shared pattern with faelight-term v3)
  wayland-client -- raw Wayland protocol
  zbus -- D-Bus for receiving notifications (org.freedesktop.Notifications)

### Surface model
  One layer surface per notification
  Layer: overlay (above everything)
  Anchor: top-right (configurable)
  Margin: 16px from edges
  Size: content-driven width, fixed max 380px
  Exclusive zone: 0 (does not push other content)

### Notification lifecycle
  Receive via D-Bus (org.freedesktop.Notifications.Notify)
  Create layer surface
  Animate in: slide from right, 200ms ease-out
  Display: timeout or until dismissed
  Animate out: fade, 150ms
  Destroy surface

### Color DNA integration
  Background: #0a0f14 at 92% opacity (Abyss Black-Blue)
  Border: #00bfff (Neon Azure) -- 1px
  Text: #a9dfff (Soft Ice Blue)
  Success: #2affd5 (Aqua Mint)
  Warning: #ffd43b (Soft Amber)
  Error: #ff4c4c (Signal Red)
  Friday signal: #00ff88 (Sharp Forest Green)

---
## Build Phases

Phase 1 -- Layer surface foundation
  Get a zwlr_layer_shell_v1 surface rendering in Niri
  Anchor to top-right, correct margin
  Render a colored rectangle (wgpu)
  Gate: colored box appears in top-right corner, above all windows

Phase 2 -- Text rendering
  Integrate cosmic-text + glyphon (same pattern as faelight-term v3)
  Render notification title and body text
  Correct font, correct color, correct wrapping
  Gate: "Hello from the forest" renders correctly in notification surface

Phase 3 -- D-Bus receiver
  Implement org.freedesktop.Notifications interface via zbus
  Receive Notify calls from any application
  Parse: app_name, summary, body, urgency, timeout
  Gate: notify-send "test" "hello" appears as a layer-shell notification

Phase 4 -- Animation
  Slide in from right (translate X, 200ms ease-out)
  Fade out (alpha, 150ms)
  wgpu handles this via transform uniforms
  Gate: smooth animation on arrival and dismissal

Phase 5 -- Stacking + multi-monitor
  Multiple notifications stack vertically with 8px gap
  Each notification is its own layer surface
  Monitor-aware: anchor to the monitor the cursor is on
  Gate: three rapid notifications stack correctly, oldest at top

Phase 6 -- Friday integration
  Friday signal color (#00ff88) for forest events
  fsh deploy completion fires a Friday-colored notification
  Intent completion fires a brief acknowledgment
  Gate: fg done triggers a Friday-green notification

Phase 7 -- Gesture dismissal
  Click to dismiss
  Swipe right to dismiss (pointer gesture)
  Gate: click and swipe both dismiss correctly

---
## Gates
Phase 1:
- [ ] layer surface renders in Niri top-right corner
- [ ] surface appears above all application windows
- [ ] correct anchor and margin

Phase 2:
- [ ] title text renders correctly
- [ ] body text renders with wrapping
- [ ] color DNA applied

Phase 3:
- [ ] org.freedesktop.Notifications implemented via zbus
- [ ] notify-send triggers a notification
- [ ] urgency levels map to correct colors

Phase 4:
- [ ] slide-in animation smooth
- [ ] fade-out animation smooth
- [ ] no tearing or flashing

Phase 5:
- [ ] multiple notifications stack correctly
- [ ] monitor-aware positioning

Phase 6:
- [ ] Friday-colored notifications for forest events
- [ ] fg done triggers notification

Phase 7:
- [ ] click dismisses
- [ ] swipe dismisses

Final:
- [ ] faelight-notify v5 replaces v4 completely
- [ ] All existing notification sources work
- [ ] foot is no longer needed for testing

---
"The notification is not an interruption.
It is the system speaking.
It should sound like itself." 🌲

---
id: 080
date: 2026-06-23
type: feature
title: "vm gui single-window: drop leftover egl-headless GL surface"
status: complete
tags: [vm, faelight-vm, spice, remote-viewer, vm-gui, lane0]
priority: medium
---
## Why
Follow-up from INT-079 / INT-077 Gate 5-6. `vm gui` opens TWO windows: the working
SPICE console AND a second window stuck at an unfinished boot. Diagnosed read-only
(2026-06-23): NOT two VMs, NOT two viewers, NOT a Mango layout quirk -- it is qemu
emitting TWO display surfaces at once. Probe of the live qemu cmdline showed BOTH
`-display egl-headless,rendernode=/dev/dri/renderD128` AND `-spice unix=on,...` active
on a single virtio-gpu-gl device. remote-viewer attaches to the SPICE surface (the good
console); the egl-headless surface renders the raw framebuffer into a second window that
stalls at boot. One qemu, one viewer process -- confirmed via vm debug (alive:1) and a
viewer probe (1 process).

The egl-headless + virtio-gpu-gl combo is LEFTOVER from the INT-077 Gate-6 Pinnacle
GL-render experiment, which was explicitly punted to real hardware (INT-067). It does
nothing for the everyday "watch the console" loop. Removing it = one clean window.

## What
Simplify cmd_gui's QOPTS for the everyday watching path: drop egl-headless +
virtio-gpu-gl, use a single simple SPICE display surface (e.g. -vga qxl or -device
virtio-gpu + -spice). Result: exactly ONE window. Script-only edit to pkgs/faelight/
scripts/vm. No VM rebuild, no fsh rebuild. Gate-5 no-fullscreen rule preserved.

## Gates
- [x] G1: cmd_gui QOPTS simplified to a single SPICE display surface (egl-headless +
      virtio-gpu-gl removed from the everyday gui path). bash -n clean. Backed up.
- [x] G2: `vm gui` opens EXACTLY ONE window (the SPICE console) -- verified live, with
      a read-only probe confirming one qemu, one viewer, one display surface.
- [x] G3: the window is windowed/resizable, NEVER fullscreen (INT-077 Gate-5 safety
      preserved -- Shift+F12 frees mouse, closeable, no screen-trap); vm down cleans up.

## Notes
- GL-accel render surface (egl-headless/virtio-gpu-gl) belongs to the Pinnacle-on-780M
  test, which is INT-067 / real-hardware territory -- NOT the everyday vm gui loop. If a
  GL path is ever wanted again, add it as an opt-in flag (e.g. `vm gui --gl`), not the default.
- Confirmed NOT a Mango tiling issue (probe showed two qemu display outputs, not one
  surface tiled oddly). So no INT-052 window-rule change needed.


## Evidence log
### 2026-06-23 -- G1+G2+G3 DEMONSTRATED
cmd_gui QOPTS changed `-device virtio-gpu-gl -display egl-headless,...` -> `-vga qxl`
(kept -spice unix socket + virtio-serial vdagent channel). Backup vm.bak-20260623T171912.
bash -n OK. Gate-5 fullscreen check: none in code.
PROVEN live: `vm gui` -> EXACTLY ONE window (the SPICE console), windowed. spice_channels
probe: qemu shows only `-vga qxl` + `-spice` (no egl-headless), 1 display device, 1 viewer
process. `vm down` closed it cleanly. The prior second window (egl-headless framebuffer
stuck at boot) is gone. Note: qxl first-boot init is a few seconds slower than the GL path
-- expected for the device swap, benign.
All 3 gates met.

## Outcome
`vm gui` now opens one clean console window. The GL render surface (for Pinnacle-on-780M)
was correctly identified as INT-077 Gate-6 scaffolding belonging to real hardware (INT-067);
if ever needed again it should return as an opt-in `vm gui --gl`, not the default.

## The Rule
"Watching the guest = one window. GL render = a different machine." 🌲

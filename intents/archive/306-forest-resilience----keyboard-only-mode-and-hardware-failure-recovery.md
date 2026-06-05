---
id: 306
title: "forest resilience -- keyboard-only mode and hardware failure recovery"
status: complete
date: 2026-05-15
type: arch
tags: [niri, keybinds, resilience, recovery, bluetooth, mouse, keyboard]
depends_on: []
---
## The Problem
On 2026-05-15 the Bluetooth adapter failed (Intel AX210, -22 error).
No mouse. No Logi Bolt receiver initially found.
Session was significantly impacted because:
  - No documented keyboard-only navigation mode
  - No quick reference for Niri window switching without mouse
  - No recovery runbook for common hardware failures
  - Cursor got stuck off-screen with no keyboard way to reset it

A system built for resilience must be resilient even when hardware fails.
The forest should never be blocked by a single point of hardware failure.

---
## The Vision
When the mouse dies, the forest keeps working.
When Bluetooth fails, there is a documented recovery path.
When any single hardware component fails, the workflow continues.

---
## Keyboard-Only Mode
### Niri navigation without mouse
Document and verify all window navigation keybinds:
  - Focus next/previous window
  - Switch workspaces
  - Move windows between workspaces
  - Launch applications (faelight-launcher keybind)
  - Close windows
  - Resize windows with keyboard

### fsh keyboard-only workflow
  - All forest vocabulary words work without mouse
  - Terminal copy/paste via keyboard (xclip/wl-clipboard keyboard shortcuts)
  - File navigation via fsh vocabulary (list, find, search)
  - Browser navigation via keyboard shortcuts

### Cursor recovery
If cursor goes off-screen or gets stuck:
  - Keyboard shortcut to reset cursor to center
  - Or: niri command to warp cursor
  - Document: how to kill a stuck input device

---
## Hardware Recovery Runbooks
### Bluetooth failure (the 2026-05-15 incident)
Symptoms: bluetoothctl says "no default controller available"
Root cause: Intel AX210 -22 firmware error after sleep/wake
Fix sequence:
  1. journalctl -b | grep -i bluetooth -- check for -22 error
  2. sudo rfkill block bluetooth && sudo rfkill unblock bluetooth
  3. sudo systemctl restart bluetooth
  4. If still failing: sudo rmmod btusb btintel && sudo modprobe btintel btusb
  5. If still failing: check /lib/firmware/intel/ibt-* exists
  6. If missing: sudo pacman -S linux-firmware-whence
  7. Nuclear option: sudo reboot
  8. Permanent fix: /etc/modprobe.d/btusb.conf with enable_autosuspend=n

### Logi Bolt pairing
If Bolt receiver needs re-pairing:
  - Install solaar: sudo pacman -S solaar
  - Run: solaar pair
  - When discovered: follow button sequence shown
  - If cursor drifts: unplug receiver immediately
  - Try different USB port if pairing fails

### Mouse alternatives
  - Logi Bolt receiver: plug into USB port near charging port
  - Laptop trackpad: always available as fallback
  - Keyboard-only mode: documented above

---
## Niri Keybind Audit
Audit current keybinds for keyboard-only completeness:
  - Every action that requires mouse should have a keyboard alternative
  - Document gaps and add missing keybinds
  - Test keyboard-only workflow for 30 minutes

---
## Gates
- [x] Keyboard-only navigation documented in docs/forest-resilience.md 2026-05-26
- [x] All Niri keybinds verified -- 100 unique, no conflicts 2026-05-26
- [x] Super+Ctrl+Space added -- center-column cursor recovery 2026-05-26
- [x] Bluetooth recovery runbook documented (2026-05-15 incident pattern) 2026-05-26
- [x] Logi Bolt pairing runbook documented in forest-resilience.md 2026-05-26
- [x] fsh keyboard-only workflow documented -- all vocabulary works without mouse 2026-05-26
- [x] Keyboard-only session validated -- forest fully navigable without mouse 2026-05-26
- [x] docs/forest-resilience.md created -- all runbooks documented 2026-05-26

---
"A system that fails when one component breaks
is not a system -- it is a dependency chain.
The forest must work in the rain.
It must work without the mouse.
It must work when the hardware argues.
Resilience is not a feature.
It is the foundation." 🌲

# Faelight Forest — Autostart Map
Last updated: 2026-03-31 (v11.5.0)
Audited by: INT-169

## Niri Compositor Spawns (config.kdl)

| Tool | Status | Notes |
|------|--------|-------|
| faelight-wallpaper | ✅ Running | Needs --health flag |
| faelight-niri-bridge | ✅ Running | Daemon mode, no --health |
| launch-bar | ✅ Running | Spawns faelight-bar |
| faelight-idle | ✅ Running | Needs --health flag |
| faelight-clipboard watch | ❌ DISABLED | Panics: zwlr_data_control_device_v1 Wayland error |

## Systemd User Services

| Service | Status | Notes |
|---------|--------|-------|
| faelight-notify.service | ✅ Running | D-Bus, auto-restart enabled |
| faelight-daemon.service | ✅ Running | Forest daemon, auto-restart enabled |

## Start Order
```
Niri compositor starts
  → faelight-wallpaper    (needs compositor)
  → faelight-niri-bridge  (needs compositor + niri socket)
  → launch-bar            (needs compositor + niri socket)
  → faelight-idle         (needs compositor)
  → faelight-clipboard    (DISABLED — Wayland bug)

systemd (independent of compositor):
  → faelight-daemon       (starts early, state.db access)
  → faelight-notify       (D-Bus, independent of compositor)
```

## Known Issues

### faelight-clipboard watch — Wayland panic
```
Error: Missing event_created_child specialization for event opcode 0
       of zwlr_data_control_device_v1
```
Root cause: wayland-client crate version incompatible with current compositor.
Fix: Update wayland-client dependency or implement missing event handler.
Status: DISABLED in niri config until fixed.

## Health Check Status

| Tool | --health flag |
|------|--------------|
| faelight-wallpaper | ❌ Missing |
| faelight-niri-bridge | ❌ Missing |
| faelight-idle | ❌ Missing |
| faelight-bar | ✅ Has --health |
| faelight-notify | ✅ Has --health |
| faelight-daemon | ✅ Has --health |

## Sway Remnants
Zero. Migration to Niri is complete. No swaymsg or sway references found.

## Recommendations
1. Add --health to faelight-wallpaper, faelight-niri-bridge, faelight-idle
2. Fix faelight-clipboard Wayland compatibility
3. Consider moving faelight-idle to systemd (follows notify pattern)
4. Doctor "System Services" should verify all autostart tools are running

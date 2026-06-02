---
id: 012
date: 2026-01-07
type: future
title: "faelight-notify - Rust Notification Daemon"
status: complete
tags: [rust, wayland, notifications, dbus, v6.2]
---
## The Vision
A minimal Rust notification daemon replacing mako. Same layer-shell foundation as faelight-bar.

## Why
- mako works but isn't ours
- No integration with profiles/intents
- Can't match faelight-bar aesthetics perfectly
- DBus + layer-shell experience gained

## Features
- DBus notification protocol (org.freedesktop.Notifications)
- Layer-shell rendering (like faelight-bar)
- Faelight Forest color palette
- Click to dismiss
- Auto-expire (5s default)
- Multiple notification badge (+N)

## Technical
- wayland-client + smithay-client-toolkit
- zbus for DBus (tokio async)
- fontdue for text rendering
- Shared color constants with faelight-bar

## Success Criteria
- [x] Receives DBus notifications
- [x] Renders on Wayland layer-shell
- [x] Matches faelight-bar aesthetics
- [x] Replaces mako as daily driver
- [ ] Profile-aware (future: work = minimal, gaming = silent)
- [ ] Action buttons (future)
- [ ] Notification history (future)

## Completed
v0.2.0 released in v6.2.0 (2026-01-10)
- Working D-Bus server on org.freedesktop.Notifications
- Layer-shell overlay rendering
- Click to dismiss, auto-expire
- Integrated into Sway autostart

---
_The forest whispers._ 🌲

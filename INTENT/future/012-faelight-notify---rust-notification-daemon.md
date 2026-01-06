---
id: 012
date: 2026-01-07
type: future
title: "faelight-notify - Rust Notification Daemon"
status: planned
tags: [rust, wayland, notifications, dbus, v5.2]
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
- Profile-aware styling
- Faelight Forest color palette
- Action buttons (clickable)
- Notification history (optional)
- Do-not-disturb mode per profile

## Technical

- wayland-client + smithay-client-toolkit
- zbus for DBus
- Same font rendering as faelight-bar
- Shared color constants

## Success Criteria

- [ ] Receives DBus notifications
- [ ] Renders on Wayland layer-shell
- [ ] Matches faelight-bar aesthetics
- [ ] Profile-aware (work = minimal, gaming = silent)
- [ ] Replaces mako as daily driver

---

_The forest whispers._ 🌲

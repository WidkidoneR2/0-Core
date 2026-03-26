---
id: 141
date: 2026-03-21
type: future
title: "faelight-notify v4 — Freedesktop Spec, zbus, Wayland Native"
status: in-progress
tags: [notify, wayland, dbus, zbus, freedesktop, rust, v11]
version: 11.0.0
priority: high
---

## The Problem

faelight-notify v3 uses a custom Unix IPC socket.
This means system apps (brave, pipewire, systemd) cannot reach it.
The font/text rendering has been broken for months.
Two problems that require a clean rewrite, not patches.

## The Solution

Rewrite from scratch using the correct stack:
- `zbus` — D-Bus, implementing org.freedesktop.Notifications spec
- `smithay-client-toolkit` — Wayland surface rendering (already used in faelight-bar)
- `fontdue` — font rendering (same as faelight-bar, known working)
- Faelight Forest palette — consistent visual identity

## Why zbus + freedesktop spec matters

When spec-compliant:
- Brave browser notifications work
- systemd notifications work
- Any app using libnotify works
- Core v10 reactions can send notifications via standard D-Bus calls
- faelight-shell can notify via `notify-send`

## Architecture
```
faelight-notify v4
├── dbus.rs       — org.freedesktop.Notifications D-Bus server (zbus)
├── render.rs     — Wayland surface rendering (smithay-client-toolkit)
├── font.rs       — fontdue text rendering (copied from faelight-bar)
├── config.rs     — position, timeout, colors, font size
├── queue.rs      — notification queue (stack/fifo)
└── main.rs       — event loop
```

## Visual Design

Single notification popup — top-right corner, Faelight Forest palette:
```
╭─────────────────────────────────────╮
│  🌲  Application Name               │
│  Notification message text here     │
│  ─────────────────────  3s ████░░  │
╰─────────────────────────────────────╯
```

Features:
- Urgency levels (low/normal/critical) — different colors
- Timeout progress bar
- Action buttons (optional)
- Click to dismiss
- Stack multiple notifications

## Dependencies
```toml
zbus = "4"
smithay-client-toolkit = "0.18"
fontdue = "0.9"
```

## Reference

Study runst source for D-Bus implementation pattern.
faelight-bar for rendering pattern.
Keep it minimal — no animation, no shadows, clean text.

## Gate Check
```
✅ zbus D-Bus server — org.freedesktop.Notifications (2026-03-26)
✅ Wayland surface rendering (2026-03-26) — smithay-client-toolkit, layer shell
✅ fontdue text rendering (2026-03-26) — HackNerdFont, matches faelight-bar
✅ Urgency levels (2026-03-26) — low/normal/critical, different colors and timeouts
✅ Timeout (2026-03-26) — display_start expiry fix, low 5s normal 6s critical 10s
⬜ Click to dismiss — planned
✅ notify-send works (2026-03-26) — all three urgency levels verified
⬜ Brave browser notifications — test pending
✅ Core v10 can trigger via D-Bus (2026-03-26) — notify-send from core reactions
```

## The Phrase

**"A forest that cannot be heard
is a forest that grows alone.
v4 gives the forest a voice
the whole system can hear."**

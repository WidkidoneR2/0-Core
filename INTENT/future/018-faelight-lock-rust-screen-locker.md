---
id: 018
date: 2026-01-09
type: future
title: ""faelight-lock - Rust Screen Locker""
status: planned
tags: [[rust, wayland, security, sway, v6.2]]
---

## The Vision

A minimal Rust screen locker replacing swaylock. Same layer-shell foundation as faelight-bar.

## Why

- swaylock works but isn't ours
- No integration with profiles/themes
- Can't match faelight aesthetics perfectly
- Consistent Rust toolchain

## Features

- PAM authentication
- Layer-shell exclusive overlay
- Faelight Forest color palette
- Profile-aware (work = quick unlock, gaming = extended timeout)
- Clock display while locked
- Battery status visible
- Blur/dim background option

## Technical

- wayland-client + smithay-client-toolkit
- pam crate for authentication
- Same font rendering as faelight-bar
- Shared color constants

## Success Criteria

- [ ] Locks screen on demand (Super+L)
- [ ] PAM authentication working
- [ ] Renders on Wayland layer-shell
- [ ] Matches faelight-bar aesthetics
- [ ] Idle timeout integration
- [ ] Replaces swaylock as daily driver

---

_The forest sleeps safely._ 🌲🔒

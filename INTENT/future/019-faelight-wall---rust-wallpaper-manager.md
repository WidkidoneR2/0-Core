---
id: 019
date: 2026-01-09
type: future
title: "faelight-wall - Rust Wallpaper Manager"
status: planned
tags: [rust, wayland, theming, sway, v6.2]
---

## The Vision

A minimal Rust wallpaper manager replacing swaybg. Dynamic wallpapers with profile integration.

## Why

- swaybg is static
- No profile-aware wallpaper switching
- Can't tie wallpapers to time-of-day or mood
- Want rotation/slideshow capability

## Features

- Static wallpaper display
- Profile-aware switching (work = calm, gaming = dynamic)
- Time-based rotation (morning/evening themes)
- Smooth fade transitions
- Directory-based slideshow
- Solid color fallback

## Technical

- wayland-client + smithay-client-toolkit
- image crate for decoding
- Layer-shell background layer
- IPC for runtime wallpaper changes

## Success Criteria

- [ ] Displays wallpaper on Wayland
- [ ] Profile-aware switching
- [ ] Smooth transitions
- [ ] Low memory footprint
- [ ] Replaces swaybg as daily driver

---

_The forest changes with the seasons._ 🌲🖼️

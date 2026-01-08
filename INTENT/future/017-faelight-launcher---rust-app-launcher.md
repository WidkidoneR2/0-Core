---
id: 017
date: 2026-01-07
type: future
title: "faelight-launcher - Rust App Launcher"
status: planned
tags: [rust, wayland, launcher, v5.5]
---

## The Vision

A minimal, profile-aware app launcher built from scratch in Rust. Replace fuzzel with something truly ours.

## Why

Fuzzel works but:

- Not ours
- No profile integration
- Generic appearance

faelight-launcher will be:

- Wayland-native (layer-shell)
- Profile-aware (work apps vs gaming apps)
- Faelight Forest themed
- Built on faelight-bar foundation

## Features

### Core

- Keyboard text input
- Fuzzy matching
- .desktop file discovery
- App execution

### Profile-Aware

- `profile work` → Shows productivity apps first
- `profile gaming` → Shows games, Steam, etc.
- `profile default` → Everything

### Appearance

- Layer-shell popup (center screen)
- Faelight Forest colors
- Same font rendering as faelight-bar
- Minimal, no bloat

## Architecture

```
┌─────────────────────────────────┐
│      faelight-launcher          │
├─────────────────────────────────┤
│  [_____search input_____]       │
├─────────────────────────────────┤
│  > Firefox                      │
│    Foot                         │
│    Files                        │
│    ...                          │
└─────────────────────────────────┘
```

## Technical

### Reuse from faelight-bar

- Wayland connection
- Layer-shell surface
- Font rendering (fontdue)
- Color palette

### New Components

- `zwp_text_input_v3` — Keyboard input protocol
- `nucleo` or `fuzzy-matcher` — Fuzzy search
- `.desktop` parser — App discovery
- `std::process::Command` — App execution

## Version Roadmap

### v0.1 - Static List

- Layer-shell popup
- Hardcoded app list
- Click to launch

### v0.2 - Keyboard Input

- Text input field
- Filter list as you type

### v0.3 - Fuzzy Matching

- Proper fuzzy search
- Ranked results

### v0.4 - Desktop Integration

- Parse ~/.local/share/applications/
- Parse /usr/share/applications/
- Icons (optional)

### v0.5 - Profile Awareness

- Different app ordering per profile
- Category filtering

## Success Criteria

- [ ] Wayland layer-shell popup
- [ ] Keyboard input works
- [ ] Fuzzy matching filters apps
- [ ] Launches selected app
- [ ] Profile-aware ordering
- [ ] Replaces fuzzel as daily driver

## Keybinds

```
Enter     → Launch selected
Escape    → Close
Up/Down   → Navigate
Tab       → Autocomplete
```

## Why Not Wrap Rofi?

Wrapping isn't building. We build from scratch because:

- Full control
- Full understanding
- Matches philosophy
- Profile integration native

---

_The forest guides your path._ 🌲

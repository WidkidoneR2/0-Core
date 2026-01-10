---
id: 018
date: 2026-01-09
type: future
title: "faelight-lock - Rust Screen Locker"
status: complete
tags: [rust, wayland, security, sway, v6.2]
---
## The Vision
A minimal Rust screen locker with Faelight Forest aesthetics.

## Why
- Consistent visual theming with other faelight tools
- Single command, Faelight colors
- Foundation for future native PAM implementation

## Features (v0.1)
- Wraps swaylock with Faelight Forest colors
- Green ring indicator
- Matching background/text colors
- Error state in red

## Technical
- Wrapper around swaylock (PAM handled by swaylock)
- Custom color configuration
- Future: native PAM auth, clock display

## Success Criteria
- [x] Locks screen on demand (Super+Ctrl+Escape)
- [x] Faelight Forest colors
- [x] PAM authentication (via swaylock)
- [x] Replaces raw swaylock command
- [ ] Native PAM implementation (future v0.2)
- [ ] Clock display (future)
- [ ] Battery status (future)

## Completed
v0.1.0 released in v6.2.0 (2026-01-10)
- Swaylock wrapper with Faelight theming

---
_The forest sleeps safely._ 🌲🔒

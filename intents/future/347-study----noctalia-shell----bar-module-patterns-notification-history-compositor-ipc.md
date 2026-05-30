# INT-347 -- Study: Noctalia Shell
# Status: future
# Tags: study, bar, notifications, compositor, quickshell, qml

## Purpose
Study Noctalia Shell for patterns applicable to faelight-bar v4 and faelight-notify v5.
Do NOT adopt Noctalia. Study it.

## What to Study
- Bar module architecture -- how modules are declared and composed
- Notification system with history and Do Not Disturb
- Niri IPC integration -- how it reads workspace/window state
- Quickshell/QML widget declaration patterns
- Theme system -- color scheme switching

## What NOT to Take
- The Quickshell/QML stack itself -- forest uses Rust
- The lavender aesthetic -- forest has its own palette
- Any component that doesn't understand intent/Friday

## Applies To
- faelight-bar v4 (INT-344) -- module system patterns
- faelight-notify v5 (INT-301) -- notification history patterns
- faelight-compositor v3 (INT-343) -- IPC patterns

## Source
https://github.com/noctalia-dev/noctalia-shell
5.2k stars, native Niri support, v5 in progress

---
## Also Study: noti-rs/noti

A Wayland notification daemon in pure Rust (GPL-3.0).
53 stars, actively developed, 100% Rust.
https://github.com/noti-rs/noti

### Why It Matters
- Pure Rust -- same stack as the forest
- Custom .noti layout format -- declarative notification layout
- Per-app configuration -- matches forest's intent-aware philosophy
- Hot-reload -- no restart needed for config changes
- Already uses JetBrainsMono Nerd Font as default

### Study Focus
- Wayland layer-shell implementation in Rust
- D-Bus notification protocol handling
- Per-app configuration architecture
- Custom layout system (.noti format)
- How it handles urgency levels

### The Question for faelight-notify v5
Build from scratch or build on noti-rs patterns?
The forest owns its tools -- but studying noti-rs before v5
is the difference between reinventing vs understanding.

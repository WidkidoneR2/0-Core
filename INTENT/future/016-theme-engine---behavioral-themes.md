---
id: 016
date: 2026-01-07
type: future
title: "theme-engine - Behavioral Themes"
status: planned
tags: [rust, themes, behavior, system-state, v5.5]
---

## The Vision

Themes are not skins. Themes are states.

A theme expresses how the system should **behave**, not just how it looks.

## Philosophy

Current themes: colors, GTK styles, terminal palettes.

Next evolution: A theme defines:

- **Attention** — How aggressively the system demands focus
- **Contrast** — Visual intensity
- **Motion** — Animation presence
- **Notification behavior** — What gets through
- **Visual noise tolerance** — Information density

## Theme Definitions

```toml
[theme.dark]
contrast = "high"
motion = "minimal"
attention = "normal"
opacity = 1.0
notifications = "standard"

[theme.light]
contrast = "medium"
motion = "normal"
attention = "high"
opacity = 1.0
notifications = "prominent"

[theme.ghost]
contrast = "low"
motion = "none"
attention = "suppressed"
opacity = 0.85
notifications = "queued"
```

## Ghost Theme — Conscious Disengagement

Ghost is not pale colors. Ghost is **low presence**.

### Ghost Behaviors

| Component     | Behavior                    |
| ------------- | --------------------------- |
| Notifications | Queued, only critical shown |
| Bar           | Minimal indicators only     |
| Windows       | Reduced opacity             |
| Animations    | None                        |
| Sounds        | Muted or filtered           |
| Cursor        | Subtle, low-contrast        |

Ghost is for when you need the computer but don't want it demanding attention.

## Architecture

```
                    theme-engine
                         │
        ┌────────────────┼────────────────┐
        │                │                │
   faelight-bar    faelight-notify    Hyprland
        │                │                │
     (adapts)        (adapts)         (adapts)
```

### theme-engine Responsibilities

1. **Define** — Parse theme behavioral profiles
2. **Query** — `theme-engine current` returns active theme
3. **Signal** — Notify components on theme change
4. **Validate** — Strong typing prevents invalid themes

### Component Integration

Each component queries theme-engine:

```rust
let theme = theme_engine::current();
match theme.notifications {
    "queued" => queue_notification(n),
    "suppressed" => drop_notification(n),
    "standard" => show_notification(n),
    "prominent" => show_urgent(n),
}
```

## spectral — Theme-Aware Notification Logic

Built into faelight-notify, not separate:

```bash
spectral status
# Output: "Ghost mode: notifications queued"

spectral explain
# Output: "3 notifications suppressed. Run `spectral show` to review."

spectral show
# Output: Queued notification summary
```

## Commands

```bash
theme-engine status          # Current theme + behaviors
theme-engine set ghost       # Switch to ghost
theme-engine explain         # Why system behaves this way
theme-engine list            # Available themes
theme-engine validate        # Check theme definitions
```

## Integration with Existing Systems

| System          | Integration                                |
| --------------- | ------------------------------------------ |
| profile         | `profile work` can set theme automatically |
| faelight-bar    | Queries theme for display mode             |
| faelight-notify | Queries theme for notification behavior    |
| Hyprland        | Receives animation/opacity settings        |

## Dependencies

- faelight-notify must exist first (to have something to control)
- Unix socket or file-based IPC
- Theme definitions in `~/0-core/themes/`

## Success Criteria

- [ ] Theme definitions as behavioral profiles
- [ ] Components query theme-engine for behavior
- [ ] Ghost theme suppresses visual noise
- [ ] Notification queuing in low-attention modes
- [ ] Profile system can trigger theme changes
- [ ] `theme-engine explain` shows current behavior

## Why Rust

- Strong typing prevents nonsense themes
- Fast queries from all components
- Type-safe IPC between components

---

_The forest changes mood, not just color._ 🌲

---
id: 136
date: 2026-03-17
type: future
title: "Faelight Forest — Visual Identity & Niri Cosmetics"
status: in-progress
tags: [niri, cosmetics, visual, borders, corners, gaps, aesthetics, v11]
version: 11.1.0
priority: medium
depends_on: []
---

## The Vision

The forest should look like it feels — alive, intentional, and yours.

Right now the compositor is functional. It renders pixels.
But the *aesthetic* of the forest hasn't been expressed visually.

Every window should feel like it belongs to Faelight Forest.
Not a generic Wayland desktop. A living, breathing environment
with its own visual language.

## The Forest Visual Language

The palette is already established:
```
Forest green:    #a3e36b  — bright growth
Dark ground:     #11140f  — the earth beneath
Bark brown:      #3d2b1f  — structure and depth
Canopy shadow:   #1a2410  — depth in the dark
Morning mist:    #c8d4b8  — soft light
Amber glow:      #d4a843  — warmth and focus
```

## Pillar 1 — Window Borders

Forest-themed borders that breathe with the system.
```kdl
// Active window — forest green glow
focus-ring {
    enable
    width 2
    active-color "#a3e36b"
    inactive-color "#2d3b1f"
}

// Subtle border on all windows
border {
    enable
    width 1
    active-color "#a3e36baa"
    inactive-color "#1a2410"
}
```

**Ideas to explore:**
- Active window gets bright forest green ring
- Inactive windows fade to dark canopy shadow
- Health-aware borders — border pulses amber when health drops below 95%
- Urgent windows glow warm amber #d4a843

## Pillar 2 — Rounded Corners

Soft corners make the forest feel organic, not mechanical.
```kdl
window-rule {
    geometry-corner-radius 8
}
```

**Ideas to explore:**
- 8px radius — soft but not bubbly
- Terminal windows: slightly less radius (6px) — structured
- Floating windows: more radius (12px) — gentle
- Dialogs: maximum radius (16px) — approachable

## Pillar 3 — Gaps & Breathing Room

The forest needs space to breathe.
```kdl
gaps 12

// Screen edge gaps — the forest has margins
struts {
    left 4
    right 4
    top 4
    bottom 4
}
```

**Ideas to explore:**
- 12px between windows — comfortable without wasteful
- 4px screen edge gaps — the forest has a margin
- Larger gaps when fewer windows — adapts to context

## Pillar 4 — Window Opacity

Depth through transparency.
```kdl
// Inactive windows slightly transparent
window-rule {
    match is-focused false
    opacity 0.92
}

// Terminal: slightly transparent always
window-rule {
    match app-id "foot"
    opacity 0.95
}
```

**Ideas to explore:**
- Inactive windows: 92% opacity — present but not demanding
- Active window: full opacity — clear focus
- faelight-bar: 88% opacity — part of the background
- Terminals: 95% — slightly see-through, feels airy

## Pillar 5 — Health-Aware Visual Feedback

This is unique to Faelight Forest. No other desktop does this.

The compositor knows about forest health via state.db.
We could make the visual environment respond to system state.

**Concepts:**
```
Health 100%  → forest green borders, full brightness
Health 95%   → borders shift slightly toward amber
Health < 90% → subtle amber tint on inactive windows
Health < 80% → border pulses slowly — the forest is unwell
```

This requires a small faelight-niri-bridge update to write
current theme to a file that Niri config can reference.
Or alternatively: a dynamic config rewriter.

## Pillar 6 — Shadow & Depth

Shadows make windows feel grounded in the forest floor.
```kdl
shadow {
    enable
    softness 30
    spread 5
    offset-x 0
    offset-y 4
    color "#00000055"
}
```

**Ideas to explore:**
- Soft drop shadows — forest green tinted (#0b1a0844)
- Stronger shadow on focused window — draws the eye
- No shadow on bar — it floats separately

## Pillar 7 — Animation

Movement that feels like the forest.
```kdl
animations {
    // Window open — rises like a tree growing
    window-open {
        duration-ms 200
        curve "ease-out-expo"
    }
    // Window close — falls like a leaf
    window-close {
        duration-ms 150
        curve "ease-in-quad"
    }
    // Workspace switch — gentle slide
    workspace-switch {
        duration-ms 250
        curve "ease-out-cubic"
    }
}
```

**Ideas to explore:**
- Window open: fast ease-out — snappy but not jarring
- Workspace switch: gentle horizontal slide
- Window movement: smooth cubic — feels physical

## Pillar 8 — Wallpaper Integration

faelight-wallpaper already exists. Make it seasonal.

**Concepts:**
- Morning session: lighter forest tones
- Evening session: darker, warmer canopy
- Health-aware: healthy forest = lush green, degraded = autumn tones
- Commit milestones: special wallpaper at 1000, 1500, 2000 commits

## Implementation Plan
```
Phase 1  — Core cosmetics (borders, corners, gaps)
           config.kdl changes only — 30 minutes
Phase 2  — Opacity and depth
           per-app window rules
Phase 3  — Shadows and animation tuning
           polish and feel
Phase 4  — Health-aware visuals
           requires faelight-niri-bridge integration
Phase 5  — Seasonal wallpaper system
           faelight-wallpaper upgrade
```

## Success Criteria
- ⬜ Forest green active borders (#a3e36b)
- ⬜ Rounded corners (8px default)
- ⬜ Comfortable gaps (12px between windows)
- ⬜ Screen edge margins (4px)
- ⬜ Inactive window opacity (92%)
- ⬜ Drop shadows on windows
- ⬜ Smooth animations (open/close/switch)
- ⬜ Health-aware border color
- ⬜ Per-app window rules (terminal, bar, dialogs)
- ⬜ Seasonal wallpaper system

## The Phrase

**"A forest that looks alive
 is a forest that feels alive.
 The visual language is not decoration.
 It is the forest expressing itself."**

---
*"Every pixel intentional. Every animation purposeful.
The forest has a face — and it is green."* 🌲

---
id: 075
title: "Hybrid Bar/Dmenu - The Wayland Solution"
status: research
created: 2026-01-27
implements_after: "Linus validation Friday"
---

# Hybrid Bar/Dmenu - The Undiscovered Approach

## The Problem

Wayland's security model prevents traditional dmenu:
- No global keyboard grabs (prevents keyloggers)
- No arbitrary overlays (prevents fake dialogs)

Traditional approach: Separate bar + dmenu processes = Wayland violations

## The Solution

**Bar and dmenu are the SAME window with mode switching.**

### Technical Implementation

Layer-shell supports THREE keyboard modes:
1. `None` - No keyboard (bar mode)
2. `Exclusive` - Full keyboard grab (dmenu mode)  ← THIS IS KEY
3. `OnDemand` - Compositor decides (not useful)

### Mode Switching
```rust
// Bar Mode (current)
layer_surface.set_size(0, 32);
layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
layer_surface.set_exclusive_zone(32);

// Toggle to Dmenu Mode
layer_surface.set_size(0, 400);
layer_surface.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
layer_surface.set_exclusive_zone(400);
layer_surface.commit();
```

## Why This Works

1. **Single window** = No IPC, no separate processes
2. **Exclusive keyboard mode** = Legitimate for dmenu (just like lock screens)
3. **Dynamic sizing** = Smooth transitions
4. **Layer-shell native** = No protocol violations

## Why Nobody Found This

Traditional thinking: "Bar is bar, dmenu is dmenu"
Reality: "They can be the same thing in different modes"

## Next Steps

1. **Friday:** Present concept to Linus, get validation
2. **Post-Friday:** Implement with his feedback
3. **Release:** faelight-bar v2.0.0 (hybrid mode)
4. **Document:** Share solution with Wayland community

## Status

Research complete. Architecture validated.
Awaiting Linus feedback before implementation.

Discovery date: 2026-01-27
Researchers: Christian (with Claude's help)

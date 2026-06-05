---
id: 111
date: 2026-03-03
type: future
title: "faelight-bar — Fractional Scaling Support (wp_fractional_scale_v1)"
status: complete
tags: [bar, wayland, niri, scaling, hidpi, rust]
---

## Vision

faelight-bar currently renders at 1x logical resolution and gets upscaled
by Niri at 1.5x — resulting in a soft/blurry bar on eDP-2 @ 2560x1600.

The fix requires implementing the Wayland fractional scaling protocol:
- `wp_fractional_scale_v1` — receives the exact fractional scale factor
- `wp_viewport` — tells the compositor the logical destination size

Together these allow the bar to render at the correct physical pixel
density and display crisp at any fractional scale.

## Current State

- eDP-2: 2560x1600 @ 165Hz, 1.5x scale
- Bar renders at: 1707x32 logical pixels (correct)
- Buffer resolution: 1707x32 (1x — too low for 1.5x display)
- Result: compositor upscales 1x → 1.5x = blurry

## Target State

- Buffer resolution: 2560x48 (1.5x physical)
- wp_viewport destination: 1707x32 (logical)
- Result: crisp at native pixel density

## Approach
```
1. Add wp-fractional-scale to Cargo.toml
   wayland-protocols = { workspace = true, features = ["staging"] }

2. Bind wp_fractional_scale_manager_v1 from registry

3. Create fractional scale object for the bar surface

4. Handle preferred_scale event — store scale_factor (e.g. 144 = 1.44x)

5. Add wp_viewporter + wp_viewport to registry binding

6. On each draw:
   phys_w = (logical_w * scale_factor / 120).ceil() as u32
   phys_h = (BAR_HEIGHT * scale_factor / 120).ceil() as u32
   render at phys_w × phys_h
   viewport.set_destination(logical_w, BAR_HEIGHT)

7. Rebuild pool sized for max physical resolution
```

## Protocol Note

wp_fractional_scale_v1 reports scale as integer × 120:
- 1.0x = 120
- 1.5x = 180
- 2.0x = 240

## Success Criteria

- [ ] wp_fractional_scale_v1 bound from registry
- [ ] wp_viewport set on bar surface
- [ ] Buffer rendered at correct physical resolution
- [ ] Bar visually crisp on eDP-2 @ 1.5x
- [ ] No regression on 1x displays

## References

- https://wayland.app/protocols/fractional-scale-v1
- https://wayland.app/protocols/viewporter
- Niri handles this correctly for all its own surfaces

---

*"The forest should be sharp all the way to the pixel."* 🌲

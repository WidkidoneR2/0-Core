---
id: 344
title: "faelight-bar v4 -- Layer-Shell Upgrade -- Compositor-Native i3 Bar"
status: planned
date: 2026-05-26
tags: [faelight-bar, layer-shell, i3, wayland, iced, sctk, compositor]
depends_on: [337, 343, 321]
---

## Why This Intent Exists

faelight-bar v2 uses xdg-shell -- same surface type as application windows.
Snowcap (Pinnacle's bar system) proves the correct pattern:
smithay-client-toolkit + wlr-layer-shell + Iced.

faelight-bar already has Iced. Adding layer-shell makes it compositor-native.
The result: a true i3-style bar that knows its place in the compositor stack.

## What Changes

### Add to Cargo.toml
```toml
smithay-client-toolkit = { version = "0.19", features = ["wlr-layer-shell"] }
```

### Layer Surface Creation (from Snowcap pattern)
```rust
let layer_shell = LayerShell::bind(&globals, &queue_handle)?;
let layer = layer_shell.create_layer_surface(
    &queue_handle,
    surface,
    Layer::Top,           // above windows, below overlay
    Some("faelight-bar"),
    None,                 // all outputs
);
layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
layer.set_size(0, BAR_HEIGHT);
layer.set_exclusive_zone(BAR_HEIGHT as i32);  // push windows down
layer.commit();
```

## i3 Bar Feature Set

### Zone Layout (from INT-321 design)
┌─────────────────────────────────────────────────────────────────┐
│  🔒 [1] [2] [3]    INT-343: faelight-compositor v3   19:42  🌲  │
└─────────────────────────────────────────────────────────────────┘
Left: lock status + workspace tags
Center: active intent (from focus.toml) + Friday signal
Right: time + Friday status

### Mode Indicator
When compositor enters a mode (resize/launcher/session):
┌─────────────────────────────────────────────────────────────────┐
│  🔒 [1] [2] [3]   ── RESIZE MODE ──  arrows=resize ESC=exit  🌲 │
└─────────────────────────────────────────────────────────────────┘
Bar communicates with compositor via Unix socket (state.db or IPC).

### Workspace Tags (i3 style)
Each tag shows: number, active indicator, window count.
Click to switch workspace (pointer events on layer surface).
Forest tags from state.db (not hardcoded).

## Gates
- [ ] smithay-client-toolkit wlr-layer-shell dependency added
- [ ] Layer surface creation -- bar anchored to top, exclusive zone set
- [ ] Bar visible above all windows in faelight-compositor v3
- [ ] Zone layout: left=lock+tags, center=intent, right=time
- [ ] Mode indicator: shows current compositor mode
- [ ] Workspace tags: click to switch
- [ ] Friday signal: brief flash on high-confidence prediction
- [ ] Works on NixOS + Pinnacle (primary target)

## Depends On
- INT-343 (faelight-compositor v3) -- needs WlrLayerShellState
- INT-337 (Snowcap study) -- patterns studied ✅

---
"The bar is not a window.
It is the compositor's voice.
It knows the mode.
It shows the intent.
It is always there." 🌲

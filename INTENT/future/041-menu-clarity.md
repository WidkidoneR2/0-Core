---
id: 041
date: 2026-01-11
type: future
title: "faelight-menu Clarity & Safety"
status: planned
tags: [rust, menu, safety, v0.3]
---

## Philosophy
> "Launcher is exploratory. Menu is decisive."
> "Let color carry danger, not weight."

## v0.3 - Clarity
- [ ] Divider line before dangerous actions (Reboot/Shutdown)
- [ ] Danger visible before selection (desaturated WARN)
- [ ] Unified quick-keys: all select only, Enter executes
- [ ] Typography: title bold (20), items regular (18), hint (14)
- [ ] ROW_HEIGHT: 44px (tighter than 48)

## v0.4 - Intent Friction
- [ ] "Enter again to confirm" for dangerous actions
- [ ] Baseline-aware text rendering
- [ ] Selection rect centered on text baseline
- [ ] Hint text changes when dangerous selected: "⚠ Enter to reboot"

## v0.5 - Shared Renderer
- [ ] Extract faelight-render module
  - canvas.rs
  - rect.rs
  - text.rs
  - theme.rs
- [ ] Share across launcher, menu, notify

## Layout Constants
```rust
const FONT_TITLE: f32 = 20.0;
const FONT_ITEM: f32 = 18.0;
const FONT_HINT: f32 = 14.0;
const ROW_HEIGHT: u32 = 44;
```

## Danger Model
```
Lock
Logout
Suspend
────────────
Reboot      ← desaturated warn
Shutdown    ← desaturated warn
```

---
_The forest acts with intention._ 🌲

---
id: 041
date: 2026-01-11
type: future
title: "faelight-menu Clarity & Safety"
status: complete
tags: [rust, menu, safety, v0.3]
---

## Philosophy
> "Launcher is exploratory. Menu is decisive."
> "Let color carry danger, not weight."

## v0.3 - Clarity
- [x] Divider line before dangerous actions (Reboot/Shutdown)
- [x] Danger visible before selection (desaturated WARN)
- [x] Unified quick-keys: all select only, Enter executes
- [x] Typography: title bold (20), items regular (18), hint (14)
- [x] ROW_HEIGHT: 44px (tighter than 48)

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

## Unique Faelight Features (v0.5+)
- [ ] Snapshot before reboot (auto-create)
- [ ] Health check on logout (warn uncommitted)
- [ ] Profile-aware (show current profile)
- [ ] Intent logging ("User initiated reboot")
- [ ] Lock core before shutdown

## Comparison Target
> "Better than wlogout - same polish, more safety, zero bloat."

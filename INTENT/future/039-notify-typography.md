---
id: 039
date: 2026-01-11
type: future
title: "faelight-notify Typography Polish"
status: complete
tags: [rust, notifications, typography, v0.4]
---

## Vision
Transform notifications from "hand-drawn" to "designed".

## Quick Wins (v0.4)
- [x] Typography hierarchy (app: 16, title: 20, body: 17)
- [x] Title color brighter than body
- [x] Vertical rhythm spacing

## Baseline & Kerning (v0.5)
- [ ] Use font metrics (ascent/descent/line_gap)
- [ ] Baseline-aware positioning
- [ ] Kerning pairs (AV, To, Wa)
- [ ] Glyph cache for stability

## Polish (v0.6)
- [ ] Gamma-aware alpha blending
- [ ] DPI/scale factor awareness
- [ ] Rounded corner illusion
- [ ] Softer border (1px + alpha stroke)

## Typography Constants
```rust
const FONT_APP: f32 = 16.0;
const FONT_TITLE: f32 = 20.0;
const FONT_BODY: f32 = 17.0;
const TITLE_COLOR: [u8; 4] = [0xa8, 0xe9, 0x78, 0xFF];
```

---
_The forest speaks clearly._ 🌲

---
id: 040
date: 2026-01-11
type: future
title: "faelight-launcher Polish"
status: complete
tags: [rust, launcher, typography, v0.4]
---

## Vision
From "mechanical" to "calm, intentional command surface".

## v0.4 - Feel Perfect
- [x] Typography hierarchy (FONT_TITLE: 22, FONT_ITEM: 18, FONT_HINT: 14)
- [x] ROW_HEIGHT constant (44px, not magic 42)
- [x] Selection padding (+4px vertical breathing)
- [x] Rank fuzzy results by score
- [x] Escape: first clears search, second closes

## v0.5 - Polish
- [ ] Highlight matched characters in results
- [ ] Baseline-aware text rendering
- [ ] Cache filtered_apps in state (recompute on change only)
- [ ] Preserve selection on backspace when valid

## v0.6 - Infrastructure
- [ ] Shared GlyphCache module (launcher, notify, bar)
- [ ] EntryKind enum (App, Command, Intent)
- [ ] Regular + Bold font hierarchy

## Typography Constants
```rust
const FONT_TITLE: f32 = 22.0;
const FONT_SEARCH: f32 = 16.0;
const FONT_ITEM: f32 = 18.0;
const FONT_HINT: f32 = 14.0;
const ROW_HEIGHT: u32 = 44;
const ROW_START: u32 = 100;
```

## Philosophy
> "Make it feel perfect first. Do not add commands yet."

---
_The forest responds with intention._ 🌲

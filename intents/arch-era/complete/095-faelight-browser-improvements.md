---
id: 095
date: 2026-02-25
type: complete
title: "faelight-browser — Stability & Feature Improvements"
status: complete
tags: [rust, browser, tui, w3m, v0.4]
version: 0.4.0
---

## Vision

faelight-browser v0.4.0 is functional on pure HTML sites. Core loop works:
load page → inline link highlighting → Tab to navigate → Enter to follow.

---

## Completed 2026-02-25

### Navigation ✅
- Forward navigation stack (Shift+F)
- Back navigation (Shift+B)
- Page title extraction from `<title>` tag

### Content ✅
- Reader mode (Shift+R) — strips nav/header/footer/aside/script/style
- Extracts `<article>` or `<main>` content when available
- Raw HTML stored in Tab for mode toggling without re-fetch

### Search ✅
- In-page search (Ctrl+F) — FindInPage mode
- Find next (Enter) / find prev (Shift+Tab)
- Match count shown in status bar
- Brave web search (/) still available

### Stability ✅
- Unicode panic fixes — all byte slices replaced with char-safe slicing
- No crashes on emoji or Unicode page titles
- Duplicate keybind conflicts resolved

---

## Sites That Work Well

Pure HTML sites where faelight-browser shines:
- news.ycombinator.com
- en.wikipedia.org
- lobste.rs
- most documentation sites

## Status: OUT OF WIP — Production Ready v0.4.0

## Future (v0.5.0)
- Replace regex HTML parser with `scraper` crate
- Better link matching on complex layouts
- Session restore
- Download to file (d key)

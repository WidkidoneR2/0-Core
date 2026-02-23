---
id: 095
date: 2026-02-23
type: future
title: "faelight-browser — Stability & Feature Improvements"
status: planned
tags: [rust, browser, tui, w3m, v11]
version: 11.0.0
---

## Vision

faelight-browser v0.4.0 is functional on pure HTML sites. Core loop works:
load page → inline link highlighting → Tab to navigate → Enter to follow.
Next phase focuses on stability, forward navigation, and better HTML parsing.

---

## Working Well (v0.4.0)

- ✅ Forest palette aesthetic — full terminal fill
- ✅ w3m-style inline link highlighting
- ✅ Tab/Shift+Tab to navigate links
- ✅ Enter to follow links
- ✅ Back navigation stack
- ✅ History panel
- ✅ Bookmarks with persistence
- ✅ Brave search integration (/ to search)
- ✅ Works on pure HTML sites (HN, Wikipedia, etc.)

## Known Limitations

- JS-heavy sites (GitHub, Reddit) render empty — fundamental TUI limitation
- Link anchor text matching is fragile on complex layouts
- No forward navigation (F key)
- Page titles sometimes show domain instead of real title

---

## Improvement Plan

### Phase 1 — Navigation
- Forward navigation stack (F key)
- Page title extraction from `<title>` tag — polish
- Anchor text matching improvements

### Phase 2 — Content
- Reader mode toggle — strip navigation/ads, show just article
- Better HTML-to-text rendering for tables and lists
- Image alt-text display

### Phase 3 — UX
- Download link to file (d key)
- In-page text search (Ctrl+F in content focus)
- Session restore — reopen last URLs on launch

### Phase 4 — Version Bump
- Cargo.toml version bump to 0.4.0
- README with keybindings and usage

---

## Sites That Work Well

Pure HTML sites where faelight-browser shines:
- news.ycombinator.com
- en.wikipedia.org
- lite.cnn.com
- lobste.rs
- most documentation sites


---
id: 027
date: 2026-01-10
type: future
title: "faelight-launcher Fuzzy Search"
status: planned
tags: [rust, launcher, ux]
---

## The Problem
Currently faelight-launcher requires arrow key navigation through a static list. With 12+ apps, this is slow.

## The Solution
Add fuzzy search - type to filter apps in real-time.

## Implementation
- Text input field at top
- Filter apps as user types
- Fuzzy matching (e.g., "ff" matches "Firefox")
- Highlight matched characters
- Enter launches top result

## Technical
- Add text input handling to keyboard handler
- Implement fuzzy matching algorithm
- Dynamic list rendering based on filter

## Success Criteria
- [ ] Type to filter apps
- [ ] Fuzzy matching works
- [ ] Enter launches top match
- [ ] Backspace clears filter
- [ ] Escape closes or clears

---
_Fast access to any app._ 🌲

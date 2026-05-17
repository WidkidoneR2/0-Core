---
id: 319
title: "faelight-menu v4 -- COSMIC visual upgrade, forest launcher"
status: planned
date: 2026-05-18
tags: faelight-menu, libcosmic, launcher, visual, cosmic, v15
depends_on: [317]
blocks: []
---

## Why This Intent Exists

faelight-menu exists but predates the COSMIC direction.
faelight-fm v2 set the visual standard for the forest.
faelight-menu must match it -- same stack, same quality, same feel.

The launcher is the first thing a visitor sees when they press
the key to open it. It must be worthy of the forest.

---

## Vision

faelight-menu v4 is a libcosmic launcher:
- Forest green background, glowing accents
- Fuzzy search across all forest commands, tools, intents
- Friday context on every result -- what this tool does,
  when it was last used, what intent it relates to
- Candy icon style matching INT-317 visual identity
- Smooth open/close animation
- Sub-50ms open time

---

## What It Launches

- All fsh vocabulary commands with descriptions
- All deployed tools (51+)
- Active intents -- cistart directly from menu
- Recent files from faelight-fm history
- Friday suggestions at the top -- "you usually run X next"
- Core domains -- core doctor, core friday, core intent...

---

## Gates

Phase 1 -- COSMIC port:
- [ ] faelight-menu rebuilt on libcosmic
- [ ] Forest colors throughout (matches faelight-fm palette)
- [ ] Opens in under 50ms

Phase 2 -- Forest intelligence:
- [ ] Fuzzy search across vocabulary + tools + intents
- [ ] Friday suggestions at top of results
- [ ] Shows last-used time per tool
- [ ] Intent-aware: cistart from menu result

Phase 3 -- Visual polish:
- [ ] Candy icon style (INT-317)
- [ ] Smooth open/close animation
- [ ] Glowing accent on selected item
- [ ] Matches faelight-fm visual quality

Final:
- [ ] faelight-menu looks and feels like it belongs in the forest
- [ ] Friday suggestions visible immediately on open
- [ ] Presentation-ready -- a visitor can navigate the forest from the menu
- [ ] Replaces any need for dmenu or rofi

"The door to the forest should be as beautiful
as everything inside it." 🌲

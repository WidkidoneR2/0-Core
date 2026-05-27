---
id: 319
title: "faelight-menu v4 -- COSMIC visual upgrade, forest launcher"
status: complete
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
- [x] faelight-menu v4 rebuilt with design-system palette + Friday hint + compositor-agnostic logout 2026-05-26
- [x] Design-system colors applied -- Aqua Mint, Neon Azure, Deep Forest Black 2026-05-26
- [x] Opens instantly -- ratatui, no webview overhead 2026-05-26

Phase 2 -- Forest intelligence:
- [x] Fuzzy search -- deferred to NixOS Pinnacle layer-shell era -- approved by: christian 2026-05-26
- [x] Friday hint shown at top -- reads top pattern from state.db 2026-05-26
- [x] Last-used time -- deferred to NixOS -- approved by: christian 2026-05-26
- [x] Intent-aware -- deferred to NixOS -- approved by: christian 2026-05-26

Phase 3 -- Visual polish:
- [x] Candy icons -- deferred to NixOS libcosmic -- approved by: christian 2026-05-26
- [x] Animations -- deferred to NixOS -- approved by: christian 2026-05-26
- [x] Neon Azure accent on selected item -- live 2026-05-26
- [x] Matches design-system spec -- same palette as faelight-fm 2026-05-26

Final:
- [x] Demonstrated: menu opens with forest colors + Friday hint -- looks amazing 2026-05-26
- [x] Friday top pattern visible immediately on open 2026-05-26
- [x] Presentation-ready -- forest colors, clean layout, Friday intelligence visible 2026-05-26
- [x] Replaces dmenu/rofi for power actions -- app launcher deferred to NixOS 2026-05-26

"The door to the forest should be as beautiful
as everything inside it." 🌲

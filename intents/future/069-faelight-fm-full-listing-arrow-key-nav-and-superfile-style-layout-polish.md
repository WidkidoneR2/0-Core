---
id: 069
date: 2026-06-20
type: future
title: "Faelight-FM: full listing, arrow-key nav, and Superfile-style layout polish"
status: planned
tags: [faelight-fm, file-manager, tui, ratatui, ux, navigation, layout, helix]
---

## Why
Faelight-FM is the forest's TUI file manager, but it currently truncates directory
listings (a correctness bug) and the layout/UX needs polish to match the clean,
organized feel of Superfile. This intent fixes the listing and reshapes navigation +
layout.

## What (observed in use)
Bugs:
- Listings truncate: intents/future shows only 5 entries when 19 exist. Must show all
  (scroll / viewport / pagination).
- Stray text by the helpers: an intent title ("faelight-login + ...", i.e. INT-005) is
  leaking into the FM near the helper bar; remove it.
Navigation:
- Arrow keys should navigate (up/down; left/right for dir in/out as fits), alongside
  the existing keys.
Layout (Superfile-inspired -- clean, organized):
- A "Faelight-FM" title header above the directory listing.
- Helper bar: more/better spacing, generally improved.
- Reduce the vertical gap between the listing and the helper bar.
- Directory rows slightly larger.
Behavior:
- Selecting a file opens it directly in Helix (hx).
North star: Superfile (spf) layout.

## Approach
Likely a ratatui TUI (forest tool family). The truncation is probably a fixed viewport
height / row cap / missing scroll-offset -- fix so the list scrolls through all entries.
The stray INT-005 text suggests a status/preview element pulling the focused intent;
remove or correct it. Layout is ratatui blocks/constraints: add a title block, retune
the vertical layout (less gap), bump row spacing, refine the help bar. Open-in-Helix:
suspend the TUI, exec hx on the selected file, restore on return. EDITOR is already hx
(framework16 sessionVariables) -- reuse it.

## Phases
Phase 0 -- locate + diagnose
  Find faelight-fm source; identify the listing-truncation cause and the stray-text
  source. Record here (mirrors the INT-068 dispatch-mapping step).
  Gate: source located; listing-limit + stray-text causes identified and noted

Phase 1 -- correctness
  Show all entries (scroll the full directory); remove the stray INT-005 title.
  Gate: a 19-entry dir shows all 19; no stray intent-title text near the helpers

Phase 2 -- navigation
  Arrow-key navigation alongside existing keys.
  Gate: arrow keys navigate the listing

Phase 3 -- layout polish (Superfile-inspired)
  "Faelight-FM" title above the listing; tighter listing<->helper gap; slightly larger
  rows; improved helper-bar spacing.
  Gate: title header + tighter gap + larger rows + helper spacing (visual review)

Phase 4 -- open in Helix
  Selecting a file launches hx on it (suspend TUI, exec, restore).
  Gate: opening a file opens it in Helix

## Gates
- [ ] Phase 0: faelight-fm source located; listing-limit + stray-text causes identified
- [ ] full listing: a 19-entry directory (intents/future) shows all 19 (scrollable)
- [ ] stray intent-title text near the helpers removed
- [ ] arrow-key navigation works
- [ ] "Faelight-FM" title header renders above the listing
- [ ] layout polish: tighter listing/helper gap, slightly larger rows, improved helper spacing
- [ ] selecting a file opens it in Helix (hx)

## Notes
- Aesthetic north star: Superfile (spf).
- Reuse EDITOR=hx (framework16 sessionVariables).
- Numbering: intent add assigned a duplicate 068; manually corrected to 069. The next-id
  logic skips complete/ -- worth fixing so it stops colliding.

## The Rule
"Show everything the forest holds, cleanly -- and hand you straight to the work."

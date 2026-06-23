---
id: 069
date: 2026-06-20
type: future
title: "Faelight-FM: full listing, arrow-key nav, and Superfile-style layout polish"
status: in-progress
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
- Basic file ops: copy / move / delete / rename.
- Path breadcrumb across the top showing the current location.
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

## Phase 0 Findings (2026-06-23)
Source: rust-tools/faelight-fm/ (ratatui TUI, v3.1, broot-style tree). Clean module
split: ui/mod.rs (render), fs/mod.rs (tree load/flatten), input/mod.rs (keys),
types.rs (state), plugins/ (intent/git/nix).

TRUNCATION CAUSE -- found: fs/mod.rs:4  const MAX_CHILDREN_SHOWN: usize = 6.
expand_node() loads all children then `.take(6)` and stuffs the remainder into
node.unlisted, which flatten() renders as a "N unlisted" marker row. So a 19-entry dir
shows only ~6 + an "unlisted" marker. This is a DELIBERATE broot-style cap, NOT a
viewport/scroll bug -- the renderer (render_tree) already uses render_stateful_widget
with full ListState scroll and builds items from ALL of `filtered` (no slice). Fix =
remove the cap so the full directory shows and scrolls.

STRAY TEXT CAUSE -- found: render_status()/render_dual_status() in ui/mod.rs take
active_intent:&str and render it into the status bar (MAGENTA "active intent"). The
INT-005 "faelight-login..." leak is the active-intent being drawn in the status line by
design. Fix = remove/relocate the active-intent from the status render.

DESIGN GOAL (Christian): make faelight-fm spectacular like faelight-logout (also a
ratatui TUI) -- craft pass on layout/spacing/header/color, staying ratatui (GTK4 rewrite
explicitly rejected: it would be a different program, large unaudited C dep tree, against
"every tool understood").
## Gates
- [x] Phase 0: faelight-fm source located; listing-limit + stray-text causes identified
- [x] full listing: a 19-entry directory (intents/future) shows all 19 (scrollable)
- [x] stray intent-title text near the helpers removed
- [x] arrow-key navigation works
- [x] "Faelight-FM" title header renders above the listing
- [ ] layout polish: tighter listing/helper gap, slightly larger rows, improved helper spacing
- [x] selecting a file opens it in Helix (hx)
- [ ] basic file ops: copy / move / delete / rename
- [ ] path breadcrumb shows the current location at the top

## Notes
- Aesthetic north star: Superfile (spf).
- Reuse EDITOR=hx (framework16 sessionVariables).
- Numbering: intent add assigned a duplicate 068; manually corrected to 069. The next-id
  logic skips complete/ -- worth fixing so it stops colliding.

## The Rule
"Show everything the forest holds, cleanly -- and hand you straight to the work."

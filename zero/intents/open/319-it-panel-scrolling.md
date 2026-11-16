---
id: 319
date: 2026-11-15
type: future
title: "it: page and scroll both panes"
status: planned
tags: [nsh, tui]
format: 2
---

## Vision
`it` pages the intent list and scrolls the detail, and the pane under the
pointer is the one that moves. Tab chooses which pane the keys act on. The
terminal is put back when the screen closes, including when it closes on an
error.

## The Problem
The left pane is a list that moves one row at a time. Page Up and Page Down
only change the detail scroll, by five lines, and there is no Home, End, or
mouse wheel. Nothing records which pane is focused, so a key cannot mean
"page this pane." Mouse capture is off, and a capture that is turned on has
to be turned off on every way out or the terminal stays grabbed.

## The Solution
A pure `list_nav` steps a selection by the visible height, or jumps to the
ends, and clamps. The screen keeps a focus, List or Detail. Tab switches it
and the focused border is highlighted. With the list focused, Page Up, Page
Down, Home, and End call `list_nav` and a selection change resets the detail
scroll. With the detail focused, Page Up and Page Down keep the saturating
scroll they have. The wheel over the left pane moves the selection by one,
and over the right pane moves the detail by three, using the stored split.
Mouse capture is enabled on entry and disabled on every exit.

## Success Criteria
- [ ] G1 list paging helper tested. Page down moves by the visible height
      (at least 1) and clamps at the last row. Page up clamps at 0. Home
      is 0. End is the last row. An empty list returns 0 and does not panic.
      A height of 0 still moves by 1.
      ask: list_nav pages by height, clamps, jumps, and an empty list returns 0
      ruling:
      evidence:
- [ ] G2 focus and keys (Tab, PgUp/PgDn, Home/End) work in it. Tab toggles
      the focused pane. Page keys page the list or scroll the detail
      according to that focus. Home and End jump the list.
      ask: Tab, PgUp/PgDn, and Home/End act on the focused pane in it
      ruling:
      evidence:
- [ ] G3 mouse wheel scrolls the pane under the pointer. The wheel over the
      list moves the selection by one. The wheel over the detail changes
      its scroll by three. The column is compared with the stored split.
      ask: the wheel scrolls the pane under the pointer
      ruling:
      evidence:
- [ ] G4 doors, nsh-test and d green on the laptop.
      ask: doors, nsh-test, and d are green on the laptop
      ruling:
      evidence:

## Out of scope
- How intents are loaded or filtered stays as it is.
- Shared TUI code with cheatsheet_tui.rs is not refactored.

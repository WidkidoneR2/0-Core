---
id: 284
title: "faelight-term rendering bugs -- scrollback corruption, emoji width, mouse flash"
status: planned
date: 2026-05-08
tags: [faelight-term, rendering, bugfix, scrollback, emoji, vte]
---
Three distinct rendering bugs identified during session 2026-05-08.
Each has a clear reproduction case. Fix one at a time.
---
BUG 1 -- SCROLLBACK CORRUPTION ON REPEATED LARGE OUTPUT (HIGH PRIORITY)
Reproduction:
  1. Run a command with large output (e.g. `d` -- 35+ lines)
  2. Run `d` again immediately (cursor is now at terminal bottom)
  3. Second `d` shows only the header box, then prompt
Pattern:
  - First `d` after short command: WORKS
  - Second `d` immediately after first: TRUNCATED
  - `d` after a short command (fsh, ls): WORKS again
Root cause hypothesis:
  When cursor starts at the bottom of the terminal (row 29 of 30),
  `d`'s entire 35-line output must scroll through the bottom.
  During extensive scrolling, something in the grid/scrollback
  transition corrupts the rendered view.
  The content may be in scrollback but not shown.
What was tried:
  - PTY burst read fix (32KB buffer + 3ms poll): improved first-run
  - Removed cursor-to-(0,0) scroll_offset reset
  - Added always-follow-output (scroll_offset = 0 after output)
  - None fully fixed the second-run truncation
Investigation path:
  - Add debug logging: print grid content after each scroll_up
  - Verify grid.len() == terminal.rows after 35 scroll_ups
  - Check if soft_wrapped Vec gets out of sync with grid Vec
  - Test if removing the resize fix (drain from top) restores behavior
---
BUG 2 -- EMOJI CELL WIDTH (MEDIUM PRIORITY)
Reproduction:
  Compare `d` output in foot vs faelight-term.
  In faelight-term: `🏥  Faelight` (extra space after emoji)
  In foot:          `🏥 Faelight` (correct single space)
Root cause:
  fontdue renders emoji as 1 cell wide.
  Unicode standard: emoji should be 2 cells wide.
  Text after emoji is offset by 1 extra cell in faelight-term.
Fix approach:
  In the VTE character processing, when a character is emoji
  (Unicode range U+1F000+), advance cursor by 2 cells not 1.
  Or: use a unicode-width crate to get correct cell width.
---
BUG 3 -- MOUSE DRAG FLASHING (LOW PRIORITY)
Reproduction:
  Click and drag to select text in faelight-term.
  Screen flashes/redraws on every mouse move during drag.
Root cause:
  pointer_frame() calls render() on every mouse event.
  During drag, mouse events fire many times per second.
  Each one triggers a full buffer redraw.
Fix approach:
  Add a dirty flag for selection changes.
  Only re-render if the selected region actually changed
  (different start/end cell than previous frame).
  Or: debounce selection renders to max 30fps.
---
GATES
BUG 1:
[ ] Root cause identified -- why does grid lose content after 35 scroll_ups?
[ ] Second `d` after first `d` shows complete output
[ ] Verified with 3 consecutive `d` commands all showing complete output
BUG 2:
[ ] Emoji characters advance cursor by correct width (2 cells)
[ ] `d` output in faelight-term matches foot spacing
[ ] No extra space after emoji in any output
BUG 3:
[ ] Mouse drag selection no longer causes flashing
[ ] Selection still updates correctly during drag
[ ] Render rate during drag <= 30fps
Final:
[ ] All three bugs fixed
[ ] faelight-term used as primary terminal for 1 week without reverting to foot
[ ] The forest no longer needs foot as a fallback

---
id: 292
title: "faelight-term v3 replaces v2 -- clean transition, clipboard, cursor, resize, path resilience"
status: complete
date: 2026-05-12
tags: [faelight-term, v3, transition, clipboard, cursor, resize, doctor, path-resilience]
---
faelight-term v3 is deployed and running as daily driver.
Super+Enter opens the GPU terminal (wgpu + cosmic-text + glyphon).
Stack: AMD RX 7700S RADV NAVI33, Vulkan, 60fps render loop.

---
COMPLETED
[x] wl-clipboard-rs 0.9.3 integrated
[x] Ctrl+Shift+V -- paste from Wayland clipboard (threaded, deadlock-free)
[x] Mouse selection -- click+drag with gold highlight
[x] Copy to clipboard -- selection text extracted from term.grid()
[x] Copy across scroll boundary -- global Line coords (fixed 2026-05-12)
[x] Paste from terminal to browser (Brave) -- foreground(true) fix (2026-05-12)
[x] Cursor rendering -- green block cursor at correct PTY position
[x] ANSI colors -- full 256-color palette
[x] Scrollback -- 10,000 lines, mouse wheel scroll
[x] Keyboard -- all keys, arrows, ctrl sequences
[x] PTY -- fsh spawns and runs correctly
[x] CELL_W = 10.5, tuned to match foot, no text wrapping
[x] Wayland deadlock fix -- initial render before blocking_dispatch
[x] Seat/pointer capability detection fixed

---
REMAINING
⏸ Window resize -- deferred to INT-324 faelight-term v4 -- approved by: christian 2026-05-25
⏸ Path resilience -- deferred to INT-324 faelight-term v4 -- approved by: christian 2026-05-25
⏸ 1 week daily driver -- deferred: term v4 will be the daily driver on NixOS -- approved by: christian 2026-05-25
⏸ v2 source retired -- deferred to INT-324 faelight-term v4 cleanup -- approved by: christian 2026-05-25
⏸ foot removed -- deferred: foot stays until term v4 is daily driver on NixOS -- approved by: christian 2026-05-25

---
GATES
[x] Clipboard copy/paste works (Ctrl+Shift+V)
[x] Cursor visible at correct position
[x] Mouse selection highlights text
[x] Copy to browser works
⏸ Window resize -- deferred to INT-324 -- approved by: christian 2026-05-25
⏸ path resilience doctor check -- deferred to INT-324 -- approved by: christian 2026-05-25
⏸ 1 week daily driver -- deferred to NixOS/INT-324 -- approved by: christian 2026-05-25
⏸ v2 retired -- deferred to INT-324 -- approved by: christian 2026-05-25
⏸ foot retirement -- deferred to NixOS/INT-324 -- approved by: christian 2026-05-25

---
KEY CONSTANTS
CELL_W: 10.5, CELL_H: 20.0, FONT_SIZE: 16.0, PADDING: 4.0
Binary: rust-tools/faelight-term-v3/

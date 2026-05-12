---
id: 292
title: "faelight-term v3 replaces v2 -- clean transition, clipboard, cursor, resize, path resilience"
status: in-progress
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
[ ] Window resize -- recalculate cols/rows, send SIGWINCH to PTY
[ ] Path resilience -- faelight-term v3 binary tracked in doctor/checks.rs
[ ] 1 week daily driver without foot fallback (timer starts 2026-05-12)
[ ] v2 source retired from rust-tools
[ ] foot removed as backup terminal

---
GATES
[x] Clipboard copy/paste works (Ctrl+Shift+V)
[x] Cursor visible at correct position
[x] Mouse selection highlights text
[x] Copy to browser works
[ ] Window resize recalculates grid and sends SIGWINCH
[ ] d shows 100% health with faelight-term v3 path resilience
[ ] 1 week daily driver without foot fallback
[ ] v2 source retired from rust-tools
[ ] foot is no longer the backup terminal

---
KEY CONSTANTS
CELL_W: 10.5, CELL_H: 20.0, FONT_SIZE: 16.0, PADDING: 4.0
Binary: rust-tools/faelight-term-v3/

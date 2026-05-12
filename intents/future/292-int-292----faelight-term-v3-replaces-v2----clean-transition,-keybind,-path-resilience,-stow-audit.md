---
id: 292
title: "faelight-term v3 replaces v2 -- clean transition, clipboard, cursor, resize, path resilience"
status: in-progress
date: 2026-05-12
tags: [faelight-term, v3, transition, clipboard, cursor, resize, doctor, path-resilience]
---

faelight-term v3 is deployed and running.
Super+Enter opens the GPU terminal.
ANSI colors work. fsh runs. Keyboard input works.

Before v2 can be fully removed, v3 needs:

---

PHASE A -- Clipboard (highest priority for daily use)

[ ] Add wl-clipboard-rs to faelight-term-v3/Cargo.toml
[ ] Ctrl+Shift+C -- copy selected text to Wayland clipboard
[ ] Ctrl+Shift+V -- paste from Wayland clipboard to PTY
[ ] Primary selection -- middle click paste
[ ] wl-clipboard-rs version: check 0.9.3 (had bug in earlier version)

---

PHASE B -- Visual completeness

[ ] Cursor rendering -- block cursor at current PTY position
[ ] Mouse selection -- click+drag highlights text region
[ ] Mouse selection color -- visible highlight over selected cells
[ ] Scrollback navigation -- Shift+PageUp/Down to scroll history
[ ] Window resize -- recalculate cols/rows, send SIGWINCH to PTY

---

PHASE C -- Integration

[ ] Path resilience -- faelight-term v3 binary tracked in doctor/checks.rs
[ ] doctor aliases.rs -- already has faelight-term entry, verify it finds v3
[ ] TOOLS.md -- update version from 2.0.0 to 3.0.0
[ ] Niri config -- Super+Enter already spawns faelight-term (works with v3)
[ ] faelight-shell downstream -- deploy script already knows faelight-term

---

PHASE D -- v2 retirement

[ ] 1 week daily driving v3 without falling back to foot
[ ] All INT-284 bugs confirmed fixed by architecture (not patches):
    - Bug 1: scrollback corruption -- alacritty_terminal ring buffer (fixed by design)
    - Bug 2: emoji width -- cosmic-text (fixed by design)
    - Bug 3: mouse drag flash -- wgpu damage rendering (fixed by design)
[ ] rust-tools/faelight-term (v2 source) retired -- moved to archive
[ ] Cargo.toml name conflict resolved -- v2 package removed from workspace
[ ] foot removed from autostart (faelight-term v3 is the daily driver)

---

CURRENT STATUS (2026-05-12)

Deployed:
  faelight-term 3.0.0 at ~/.cargo/bin/faelight-term
  Super+Enter opens v3 GPU terminal
  ANSI colors: full 256-color palette
  Keyboard: all keys including arrows, ctrl sequences
  PTY: fsh spawns and runs correctly
  Scrollback: 10000 lines (alacritty_terminal default)

Not yet:
  Clipboard (wl-clipboard-rs not added yet)
  Cursor rendering
  Mouse selection
  Resize handling
  Path resilience in doctor

---

GATES

[ ] Clipboard copy/paste works (Ctrl+Shift+C/V)
[ ] Cursor visible at correct position
[ ] Mouse selection highlights text
[ ] Window resize recalculates grid and sends SIGWINCH
[ ] d shows 100% health with faelight-term v3
[ ] 1 week daily driver without foot fallback
[ ] v2 source retired from rust-tools
[ ] foot is no longer the backup terminal

---
id: 094
date: 2026-02-25
type: complete
title: "faelight-term — VTE Refactor & Stability"
status: complete
tags: [rust, terminal, vte, wayland, v10.3]
version: 10.3.0
---

## Vision

faelight-term has solid foundations — real PTY, swash font rendering,
Wayland via Smithay, emoji support better than foot. The gaps were in the
VT parsing layer which was handrolled and incomplete.

Replaced the handrolled parser with the `vte` crate (same as Alacritty,
Zellij, WezTerm) to close the remaining issues.

---

## Completed 2026-02-25

### Phase 1 — VTE Integration ✅
- Added `vte` crate dependency
- Implemented `vte::Perform` trait routing to existing terminal grid
- Replaced `process_bytes` + `handle_csi_sequence` with `vte::Parser`
- Preserved emoji rendering pipeline unchanged

### Phase 2 — Cursor & Alternate Screen ✅
- Ghost cursor eliminated
- Alternate screen buffer (?1049h/?1049l) implemented
- nvim opens full size, prompt returns correctly on exit

### Phase 3 — Dynamic Resize ✅
- TIOCSWINSZ implemented — PTY notified of actual window dimensions
- Rows/cols calculated from pixel dimensions dynamically
- faelight-fm renders at full terminal size

### Phase 4 — Key Handling ✅
- Backspace, Delete, Escape, Return all explicit
- DSR cursor position response (atuin inline history working)
- Sway IPC (swaymsg) works inside faelight-term

### Phase 5 — Testing ✅
- btm — graphs, braille characters, temperatures all render
- faelight-fm — full size, forest palette correct
- faelight-git — colors, Unicode, layout correct
- nvim — full size, clean exit
- Copy/paste (Ctrl+Shift+C/V) working
- Zoom in/out working
- Sway keybind ($mod+Mod1+Return)

---

## Status: OUT OF WIP — Production Ready v10.3.0

Actively replacing foot as primary terminal.

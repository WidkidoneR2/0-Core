---
id: 094
date: 2026-02-23
type: future
title: "faelight-term — VTE Refactor & Stability"
status: planned
tags: [rust, terminal, vte, wayland, v11]
version: 11.0.0
---

## Vision

faelight-term has solid foundations — real PTY, swash font rendering,
Wayland via Smithay, emoji support better than foot. The gaps are in the
VT parsing layer which is handrolled and incomplete.

Replace the handrolled parser with the `vte` crate (same as Alacritty,
Zellij, WezTerm) to close the remaining issues.

---

## Known Issues

- Ghost cursor — old cursor position not cleared before drawing new one
- Commands not always executing — newline/enter handling gaps
- VT parsing incomplete — only 12 CSI patterns handled, need hundreds
- Needs testing across common programs (vim, htop, faelight-fm inside term)

## Working Well

- ✅ Emoji rendering — better than foot terminal
- ✅ Copy/paste — underline selection working
- ✅ Scrollback — 10,000 line buffer
- ✅ PTY — proper fork, setsid, TIOCSCTTY
- ✅ Font rendering — swash pipeline solid

---

## Refactor Plan

### Phase 1 — VTE Integration
- Add `vte` crate dependency
- Implement `vte::Perform` trait routing to existing terminal grid
- Replace `process_bytes` + `handle_csi_sequence` with `vte::Parser`
- Preserve emoji rendering pipeline unchanged

### Phase 2 — Cursor Fix
- Clear old cursor position before drawing new one
- Implement proper cursor state tracking

### Phase 3 — Testing
- vim inside faelight-term
- htop inside faelight-term
- faelight-fm inside faelight-term
- neovim inside faelight-term

### Phase 4 — Polish
- TERM environment variable tuning
- Color scheme matching Faelight Forest palette
- Title bar showing current command

---

## Key Insight

Architecture stays the same: PTY → vte parser → grid state → renderer
Only the middle layer (VT parsing) gets replaced.
The hard parts (PTY, font rendering, Wayland) are already solid.


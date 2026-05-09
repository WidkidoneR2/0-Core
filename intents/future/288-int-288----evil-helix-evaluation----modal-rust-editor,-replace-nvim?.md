---
id: 288
date: 2026-05-09
type: future
title: \"INT-288 -- evil-helix evaluation -- modal Rust editor, replace nvim?\"
status: planned
tags: [faelight]
version: TBD
---

## Vision

<!-- What is this intent trying to achieve? -->

## Why Now

<!-- Why is this the right time for this intent? -->

## Approach

<!-- How will this be implemented? -->

## Success Criteria

- [ ] <!-- First criterion -->
- [ ] <!-- Second criterion -->

## Gate Check
```
⬜ Not started
```

---

*\"The forest grows with intention.\"* 🌲

EVALUATION: evil-helix as nvim replacement
evil-helix is a fork of Helix editor adding vim/evil keybindings.
WHY CONSIDER IT:
  Rust-native -- aligns with forest philosophy
  Tree-sitter first -- better syntax understanding than nvim default
  LSP built-in -- no plugin required
  No plugin system (intentional) -- less configuration drift
  Cleaner architecture than nvim + plugin ecosystem
  modal editing via evil keybindings -- familiar muscle memory
WHY STAY WITH NVIM:
  Existing nvim config investment
  Plugin ecosystem (oil, telescope, etc.)
  Lua scripting for forest-specific integrations
  Known quantity -- no migration risk before presentation
EVALUATION CRITERIA:
  1. Does Rust LSP work as well as nvim + rust-analyzer?
  2. Does tree-sitter coverage match current nvim setup?
  3. Can fsh integrate with evil-helix as cleanly as nvim?
  4. Is the editing experience faster or slower day-to-day?
  5. What is lost from nvim that cannot be replaced?
EVALUATION METHOD:
  Install evil-helix alongside nvim (not replacing)
  Use for 1 week on non-critical editing tasks
  Document friction points daily
  Decision: keep nvim, switch to evil-helix, or use both
TIMELINE: post-summer -- not before presentation
GATES:
[ ] evil-helix installed alongside nvim
[ ] 1 week daily use evaluation complete
[ ] Rust LSP verified working correctly
[ ] Decision recorded with rationale
[ ] If switching: nvim config archived, not deleted

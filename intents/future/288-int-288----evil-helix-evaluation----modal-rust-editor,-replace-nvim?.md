---
id: 288
date: 2026-05-12
type: future
title: "INT-288 -- evil-helix evaluation -- modal Rust editor, replace nvim?"
status: planned
tags: [faelight, editor, helix, nvim, rust, cosmic]
version: TBD
---

## Vision

Replace nvim with evil-helix as the forest's primary editor.
evil-helix is a Helix fork with vim/evil keybindings -- Rust-native, tree-sitter
first, LSP built-in. Aligns fully with the forest philosophy and the COSMIC stack
direction. With Graydon Hoare and the POP_OS team watching the COSMIC integration
work, a full Rust-native stack (shell + terminal + editor) is a compelling demo.

## Why Now

- evil-helix aligns with the Rust-native forest philosophy
- COSMIC stack momentum -- cosmic-text proven in faelight-term v3
- Pre-presentation opportunity to show a complete Rust-native editing environment
- Graydon Hoare has noted the COSMIC integration progress
- POP_OS team has flagged the project -- full Rust stack is the story

## Approach

Install evil-helix alongside nvim (not replacing immediately).
Run 1-week parallel evaluation on non-critical editing tasks.
Focus on Rust LSP quality, tree-sitter coverage, fsh integration.
Document friction points daily in state.db as Friday observations.
Decision recorded with rationale before presentation.

## Evaluation Criteria

1. Does Rust LSP work as well as nvim + rust-analyzer?
2. Does tree-sitter coverage match current nvim setup?
3. Can fsh integrate with evil-helix as cleanly as nvim?
4. Is editing experience faster or slower day-to-day?
5. What is lost from nvim that cannot be replaced?

## Why evil-helix over pure Helix

- Vim/evil muscle memory preserved -- no relearning period
- Rust-native architecture -- no Lua runtime, no plugin ecosystem drift
- Tree-sitter first, LSP built-in -- better than nvim defaults
- Intentionally minimal -- aligns with "understanding over convenience"
- Forest philosophy: own the stack, understand every layer

## Success Criteria

- [ ] evil-helix installed alongside nvim
- [ ] 1 week daily use evaluation complete
- [ ] Rust LSP verified working at nvim + rust-analyzer quality
- [ ] fsh aliases and editor integration updated
- [ ] Decision recorded with rationale
- [ ] If switching: nvim config archived (not deleted), evil-helix config in faelight-link

## Gate Check
⬜  Not started -- scheduled pre-presentation

## Timeline

Pre-presentation (summer 2026). Not post-summer.

---

*"The forest grows with intention."* 🌲

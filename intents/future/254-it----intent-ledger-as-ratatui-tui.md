---
id: 254
date: 2026-04-28
type: feature
title: "it -- Intent Ledger as Ratatui TUI"
status: in-progress
tags: [feature, rust, faelight, tui, ratatui, intent, fsh, ux]
version: TBD
---

## Vision

`it` is a ratatui-based terminal UI for browsing, editing, and managing
intents. Invoked from fsh as `it` or hotkey, it presents the entire intent
ledger (in-progress / planned / complete / future / cancelled) in a unified
two-pane view with rich filtering, gate toggling, and inline editing.

This replaces the cycle of:
- `intent list` (overview)
- `intent show 234` (detail)
- nvim intent.md (edit)
- `intent gate close 234 5` (gate operation)

with a single live interface where state navigation is keyboard-driven and
gate toggles happen inline.

`gt` (INT-253) makes git workflow a TUI. `it` makes intent workflow a TUI.
Together they remove most of the daily-driver shell-parser friction surface.

## Why Now

1. **Intent-driven workflow is core to the project.** Every session opens
   with `intent list` and ends with `cicomplete` or gate updates. Yet each
   of these is a separate command with its own output format. A unified
   view gives Christian a constant ambient awareness of project state.

2. **Gate management is currently friction-heavy.** Toggling a single gate
   today means: `intent show 234`, find the gate line, edit the .md file,
   save, the registry picks up the change later. With `it`, select gate,
   press space, gate flips with timestamp, registry updates immediately.

3. **Foundation already shipped.** Same as INT-253: ratatui + crossterm
   pattern proven by INT-250.

4. **No risk of breaking command-line interface.** All existing `intent`
   subcommands keep working unchanged. `it` is additive.

## Approach

### Invocation
- `it` from fsh prompt -> opens TUI
- TUI exits cleanly back to fsh prompt
- Optional: `it 234` opens TUI focused on intent 234

### Layout (initial)
- Top bar: total / in-progress / planned / complete counts; current filter
- Left pane: intent list (filterable). Columns: ID, status icon, title, age.
  Selectable with arrow keys. Filterable by status (tab cycles: all -> in-progress
  -> planned -> complete -> future -> cancelled).
- Right pane: rendered intent detail. Markdown renderer with:
  - Front-matter metadata as header
  - Vision/Why Now/Approach as collapsible sections
  - Gate Check section interactive (gates toggleable inline)
  - Tags as colored chips
- Bottom bar: action keys (n=new, c=complete, e=edit-in-nvim, space=toggle-gate,
  /=search, q=quit, ?=help)

### Gate toggling
- Select an intent, navigate to Gate Check section in right pane
- Arrow keys move between gate lines
- Space toggles `⬜` -> `✅` (with timestamp added) or back
- Save happens immediately via the registry update path
- Multi-gate toggle preserved across navigation

### New intent flow
- `n` opens a small dialog for template (feature/fix/arch/study) and title
- Calls `core intent new` under the hood
- Returns to TUI with new intent selected, ready for editing

### Search
- `/` opens a fuzzy-search input
- Searches across: titles, tags, ID, content
- Live-filtered as you type

### Implementation modules (suggested)
- `rust-tools/faelight-shell/src/intent_tui/mod.rs` — entry point
- `rust-tools/faelight-shell/src/intent_tui/state.rs` — intent state model
- `rust-tools/faelight-shell/src/intent_tui/render.rs` — ratatui rendering
- `rust-tools/faelight-shell/src/intent_tui/registry.rs` — read/write registry

Or as standalone tool `rust-tools/it/` if scope grows or if it should be
callable independently of fsh.

## Hard Dependencies

- ratatui 0.28 + crossterm 0.28 (already in fsh)
- Existing intent registry / file structure (no schema changes)
- INT-253 (gt-tui) is NOT a dependency but the rendering patterns will be
  shared if both ship close together

## Success Criteria

- [ ] `it` from fsh prompt opens a working TUI
- [ ] Intent list shows all intents grouped by status with counts
- [ ] Tab cycles status filter cleanly
- [ ] Right pane renders selected intent's full markdown
- [ ] Space toggles a gate ⬜ <-> ✅ and writes change immediately
- [ ] `n` creates new intent via core intent new and reloads list
- [ ] `c` marks selected intent complete (calls cicomplete)
- [ ] `e` opens current intent in $EDITOR (nvim)
- [ ] `/` enables live fuzzy search
- [ ] `q` returns cleanly to fsh prompt
- [ ] No regression in existing intent commands

## Scope

### In scope
- Browse / search / filter all intents
- Toggle gates inline
- Create new intent (delegating to core intent new)
- Mark complete (delegating to cicomplete)
- Open in editor for free-form edits

### Out of scope (separate intents or future expansion)
- Burndown / velocity / forecast graphs in TUI (already CLI commands;
  could be added as panels later)
- Dependency graph visualization (separate intent — INT-247 Intent Ledger v2)
- Multi-intent batch operations
- Import / export
- Inline knowledge-engine queries (Friday integration)

## Gate Check
⬜ Not started

---

*"The forest knows what it has decided.
The terminal should help the human know it too."* 🌲

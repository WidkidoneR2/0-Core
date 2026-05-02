---
id: 253
date: 2026-04-28
type: feature
title: "gt -- Git Workflow as Ratatui TUI"
status: in-progress
tags: [feature, rust, faelight, tui, ratatui, git, fsh, ux]
version: TBD
---

## Vision

`gt` is a ratatui-based terminal UI for git operations, invoked from fsh with
`gt` or via Ctrl+G hotkey. It bypasses fsh's command-line parser entirely
for everything git-related, eliminating quote/escape friction (INT-245 #12,
#13, #15) for the most common workflow that exercises those gaps.

The pattern proven by INT-250 (Ctrl+R native history TUI) is the foundation:
ratatui + crossterm running inside fsh's REPL loop, feeding back via a clean
event-driven interface. `gt` extends that pattern to git workflow.

This is NOT a git replacement. It orchestrates the existing `git` binary and
faelight-git via ratatui rendering. Same outputs, same commit hashes, same
remote behavior — just a better interface for the human in front of it.

## Why Now

Three real frictions converge here:

1. **fsh parser limits.** Single quotes inside double quotes, em-dashes,
   parentheses, special chars in commit messages keep tripping fsh's
   command-line parser despite #12 and #13 fixes. Each new edge case is
   another patch. A TUI sidesteps the entire problem because the user types
   in a multiline ratatui input widget, not on a shell command line.

2. **Daily-driver context.** Christian commits multiple times per session.
   Every commit is friction surface. Reducing per-commit cost compounds.

3. **Foundation already shipped.** INT-250 proved ratatui+crossterm works
   inside fsh. The lift for `gt` is mostly UI work, not new architecture.

## Approach

### Invocation
- `gt` from fsh prompt -> opens TUI
- `Ctrl+G` from fsh REPL -> opens TUI (rustyline ConditionalEventHandler,
  same pattern as INT-250 Ctrl+R)
- TUI exits cleanly back to fsh prompt; if a git command was executed,
  fsh shows a one-line summary of what happened

### Layout (initial)
- Top bar: branch, ahead/behind, dirty file count, current commit hash short
- Left pane: file list. Sections: Staged / Unstaged / Untracked. Selectable
  with arrow keys. Numeric indicators per section.
- Right pane: diff preview of selected file. Syntax-highlighted via the
  same colors faelight-shell already uses. Scrollable independently.
- Bottom bar: action keys (s=stage, u=unstage, d=discard, c=commit, p=push,
  r=refresh, q=quit, ?=help)

### Commit flow
- `c` opens a multiline ratatui input widget for the commit message
- No fsh quote parsing involved -- user types em-dashes, $signs, parens
  freely
- `Ctrl+S` from input widget commits with the message; `Esc` cancels
- Optional: pre-fill commit message with active intent prefix (e.g.
  "INT-245: ") if Christian has an active focused intent

### Push flow
- `p` runs `git push`. If push-main check needs confirmation, `gt` shows
  the confirm dialog inline rather than passing through fsh prompt.
- Push output streamed to a transient overlay; success/failure summary
  shown in TUI before returning to file pane.

### Implementation modules (suggested)
- `rust-tools/faelight-shell/src/git_tui/mod.rs` — entry point
- `rust-tools/faelight-shell/src/git_tui/state.rs` — git state model
  (uses git2 crate, already a fsh dep)
- `rust-tools/faelight-shell/src/git_tui/render.rs` — ratatui rendering
- `rust-tools/faelight-shell/src/git_tui/actions.rs` — stage/unstage/commit/push

Or as standalone tool `rust-tools/gt/` if scope grows.

## Hard Dependencies

- ratatui 0.28 + crossterm 0.28 (already in fsh)
- git2 (already in fsh via faelight-git)
- ConditionalEventHandler pattern (proven in INT-250)

## Success Criteria

- [ ] `gt` from fsh prompt opens a working TUI
- [ ] Ctrl+G hotkey works from fsh REPL line edit mode
- [ ] File list correctly shows staged/unstaged/untracked counts
- [ ] Diff preview renders with colors for selected file
- [ ] Stage/unstage actions modify git index correctly
- [ ] Commit action opens multiline input, accepts em-dashes and `$` chars literally
- [ ] Push action runs git push, handles push-main confirmation
- [ ] Quit (q or Esc) returns cleanly to fsh prompt
- [ ] Active focused intent ID auto-prefills commit message subject line
- [ ] No regression in fsh's existing git aliases (`gc`, `gp`, etc. still work via command line)

## Scope

### In scope
- Common git workflow: status, stage, unstage, discard, commit, push
- Commit message editing without fsh parser involvement
- Active intent integration (auto-prefix)
- Visual feedback for git operations

### Out of scope (separate intents)
- Branch switching / creating / merging (INT-255 faelight-git productivity)
- Interactive rebase
- Conflict resolution
- Reflog browsing
- Stash management

These are real features but each adds complexity. Ship the daily-flow
workflow first, layer on advanced operations once the foundation is solid.

## Gate Check
⬜ Not started

---

*"The terminal is the interface between the human and the machine.
It should know what you are building and help you build it."* 🌲

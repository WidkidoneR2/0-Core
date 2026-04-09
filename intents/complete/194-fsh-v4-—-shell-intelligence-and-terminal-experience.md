---
id: 194
date: 2026-04-04
type: planned
title: "fsh v4 — Shell Intelligence and Terminal Experience"
status: complete
tags: [fsh, shell, terminal, faelight-term, intelligence, ux, v4]
# Completed: All 15 gates done. Gate 14 — edit builtin opens $EDITOR (nvim) with last command, executes result on save. fsh v4 is complete.
---
fsh v3 became a daily driver. fsh v4 becomes intelligent.
The shell should anticipate. It should remember context across sessions.
It should feel alive — not just execute commands but participate in work.
faelight-term should be more than a terminal — it should be the forest's window.
These are known limitations from v3 that v4 must address natively:
- `>` and `>>` delegate to sh — fsh should handle these natively
- `2>/dev/null` stderr redirection goes to sh — needs native support
- `||` operator falls to sh — fsh handles `&&` but not `||`
- Complex pipe chains with `|&` go to sh
- Process substitution `<(cmd)` — not supported, falls to sh
- `$(subshell)` command substitution — partial, some cases fall to sh
- Heredocs use sh stdin inheritance — should be native in v4
- `command &` background with complex pipes — partial support
- `export` works but does not persist across sessions without `persist`
- `.` shorthand for `source` not supported
- Signal handling (SIGPIPE) in pipelines — partially handled
- File path completion partial — `~/0-core/<TAB>` incomplete
- Core subcommand completion — `core intent <TAB>` not working
- Git branch completion — not implemented
When fsh detects a pattern (same sequence of commands repeated),
it should offer: "You usually run deploy after this — press Tab to confirm"
Not intrusive. One line. Dismissable.
Currently fsh handles one line at a time.
Add: Up/down arrow to navigate multi-line input
Add: Ctrl+X to open $EDITOR for complex commands
Every command already has execution time via `time`.
Surface slow commands automatically:
"⚠️  d took 8.2s — 3x slower than usual"
Currently completes aliases from DB.
Add: complete file paths (~/0-core/<TAB>)
Add: complete core subcommands (core intent <TAB>)
Add: complete git branches (git checkout <TAB>)
Shell variables set with VAR=value currently lost on restart.
Add: opt-in persistence — `persist VAR` saves to state.db
Restored on next fsh session.
`last` command — show output of last command again
`save <name>` — save last output to named slot
`recall <name>` — retrieve saved output
Currently: flat list ordered by time
Add: frequency scoring — most-used commands ranked higher in search
Add: context-aware history — `hs deploy` shows only deploy commands
     in current directory context
Currently: type DELETE in caps to confirm.
fsh v4 improvement — before asking for confirmation, show:
  ⚠️  rm -rf ~/0-core/target
  → 847 files, 2.3 GB
  → Most recent: faelight-shell (modified 4 minutes ago)
  → Type DELETE to confirm, or Ctrl+C to cancel
faelight-term is a VTE-based terminal. Works. But it is basic.
Horizontal split: Ctrl+Shift+H
Vertical split: Ctrl+Shift+V
This eliminates needing multiple terminal windows for most workflows.
Each terminal window gets a name.
Shown in title bar: "🌲 0-core | fsh"
Nameable: `term-name "deploy session"`
Title updates based on active intent: "🌲 INT-194 | fsh | ~/0-core"
Ctrl+F to search scrollback buffer
Highlights matches, jumps between them
Commands that exit non-zero get a red marker in scrollback.
Successful deploys get a green marker.
Jump between markers: Ctrl+Shift+E (next error), Ctrl+Shift+D (next deploy)
Detect URLs in output, make them clickable
Ctrl+Click to open in faelight-browser
✅ Native `>` and `>>` redirect without sh fallback
✅ Native `||` operator support
✅ Native stderr redirect `2>/dev/null`
✅ Native heredoc without sh stdin inheritance
✅ File path tab completion
✅ Core subcommand tab completion (core intent <TAB>)
✅ Git branch tab completion
✅ Prediction-aware suggestions — fires after 3+ pattern occurrences in history
✅ edit builtin — opens $EDITOR with last command, executes on save
✅ Command timing intelligence (slow command warnings)
✅ Session variable persistence — persist VAR saves to state.db, restores on login
✅ last / save / recall commands
✅ Frequency-scored history search
✅ Smarter DELETE confirmation — shows file count, size, recent files
⬜ faelight-term split panes — tracked in INT-201
✅ faelight-term session names + forest-aware title — live in INT-201
⬜ faelight-term scrollback search — tracked in INT-201
⬜ faelight-term output annotations — tracked in INT-201
⬜ faelight-term URL detection — tracked in INT-201
**"The shell is not a tool you use.
It is a space you think in.
Every improvement to the shell is an improvement to thought itself.
fsh v4 is not faster — it is smarter.
fsh v4 is not just fixed — it is complete."** 🌲

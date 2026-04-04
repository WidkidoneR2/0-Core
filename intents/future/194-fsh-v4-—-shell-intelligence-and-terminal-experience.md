---
id: 194
date: 2026-04-04
type: planned
title: "fsh v4 — Shell Intelligence and Terminal Experience"
status: planned
tags: [fsh, shell, terminal, faelight-term, intelligence, ux, v4]
---
fsh v3 became a daily driver. fsh v4 becomes intelligent.
The shell should anticipate. It should remember context across sessions.
It should feel alive — not just execute commands but participate in work.
faelight-term should be more than a terminal — it should be the forest's window.
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
faelight-term is a VTE-based terminal. Works. But it is basic.
Horizontal split: Ctrl+Shift+H
Vertical split: Ctrl+Shift+V
This eliminates needing multiple terminal windows for most workflows.
Each terminal window gets a name.
Shown in title bar: "🌲 0-core | fsh"
Nameable: `term-name "deploy session"`
Title updates based on active intent:
"🌲 INT-191 | fsh | ~/0-core"
Ctrl+F to search scrollback buffer
Highlights matches, jumps between them
Detect URLs in output, make them clickable
Ctrl+Click to open in faelight-browser
⬜ Prediction-aware suggestions in fsh (pattern detection → Tab offer)
⬜ Multi-line editing (arrow navigation, editor escape)
⬜ Command timing intelligence (slow command warnings)
⬜ File path tab completion
⬜ Core subcommand tab completion
⬜ Session variable persistence (persist VAR)
⬜ last / save / recall commands
⬜ Frequency-scored history search
⬜ faelight-term split panes
⬜ faelight-term session names + forest-aware title
⬜ faelight-term scrollback search
⬜ faelight-term URL detection + clickable links
⬜ Smarter DELETE confirmation — shows file count, size, recent files
⬜ faelight-term output annotations — red/green markers in scrollback
⬜ Jump between error markers in scrollback
"The shell is not a tool you use.
It is a space you think in.
Every improvement to the shell is an improvement to thought itself.
fsh v4 is not faster — it is smarter." 🌲

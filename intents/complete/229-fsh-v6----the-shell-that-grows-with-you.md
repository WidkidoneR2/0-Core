---
id: 229
title: "fsh v6 -- The Shell That Grows With You"
status: complete
date: 2026-04-13
tags: faelight-shell, shell, intelligence, ux, builtins
requires: []
unlocks: []
strategic_value: leaf
---
fsh v0.7.0 is the daily driver. It has:
- Native execution layer (query, fsearch, patch, edit, run)
- Color and syntax highlighting (show, goto, fdiff)
- History intelligence (ht today/session/intent/slow)
- Pattern learning and context awareness
- Native pipes without sh fallback
This is solid. But it is not yet delightful.
v6 is about joy. Speed. The feeling that the shell anticipates you.
Tab completion that knows the forest:
  deploy <TAB>  — shows deployable tools from registry
  intent <TAB>  — shows in-progress intents
  core <TAB>    — shows all core subcommands
  goto <TAB>    — shows recently edited files
Not generic completion. Forest-aware completion.
Short aliases that expand inline (like fish shell abbreviations):
  gc   — expands to: fg commit
  dc   — expands to: cicomplete
  ds   — expands to: cistart
  dep  — expands to: deploy
Typed as abbreviation, shown expanded in the line before execution.
The forest teaches you its own shortcuts.
When you type exit or ctrl-D:
  Session: 47 commands  —  12 deploys  —  3 commits
  Active: INT-201 (14 gates done)
  Time: 2h 34m  —  peak focus: 22:15-23:30
Not a wall of text. A clean, honest summary.
When a command fails:
  Current: error message
  v6: "That failed. Did you mean: query file.rs 1:50?"
Context-aware suggestions from the forest's knowledge of what you were doing.
The prompt shows time on active intent:
  → 100% · INT-201 · 2h14m · peak
You always know how long you have been working on something.
  fsh diag   — show shell health (sessions, speed, error rate, patterns)
  fsh config — show current shell configuration
  fsh gaps   — show commands you could be using but aren't (teach integration)
✅ forest-aware tab completion for deploy, intent, core — deferred to fsh v7
✅ fsh abbreviations (gc, dc, ds, dep and more) (2026-04-14)
✅ session summary on exit (commands, deploys, commits, time) (2026-04-14)
✅ smarter error recovery with forest-context suggestions — deferred to fsh v7
✅ live intent timer in prompt (hours:minutes on active intent) — deferred to fsh v7
✅ fsh diag shows shell health and patterns (2026-04-14)
✅ fsh gaps shows missed builtin opportunities (2026-04-14)
✅ d passes 100% after full implementation (2026-04-14)
"The shell is where you live.
It should know you.
Not just what you type.
What you need.
v6 is the shell that has been watching,
learning,
and is finally ready to help
before you ask." 🌲

---
id: 223
date: 2026-04-10
type: feature
title: \"fsh Native Execution Layer -- Script Mode, Query Builtin, Zero Friction Development\"
status: in-progress
tags: [feature, rust, faelight]
version: TBD
---

# fsh Native Execution Layer — Script Mode, Query Builtin, Zero Friction Development

See full content in git history commit 425723c.

## The Problem We Solve Every Single Session

Every time we read lines from a file: head/tail/tmp dance. Three commands, two temp files.
Every time we run Python: write to /tmp, execute, clean up.
Every time we search with grep: redirect to /tmp because pipes are fragile.
INT-223 eliminates all of it.

## The query Builtin

The single biggest productivity improvement for development sessions.

  query file.rs 100:150  — print lines 100-150
  query file.rs :50      — first 50 lines
  query file.rs 900:     — line 900 to end
  query file.rs fn_main  — lines containing fn_main

Pure Rust builtin. No temp files. No pipes needed.

## The fsh run Command

  fsh run file.py   — python3
  fsh run file.sh   — sh
  fsh run file.fsh  — fsh native scripting

## The patch Builtin

  patch file.rs --old "old_text" --new "new_text"

Eliminates 90% of the Python fix scripts written every session.

## The search Builtin

  search fn_expand           — all .rs files recursively
  search fn_expand --type rs — filtered by type

## The edit Builtin Enhancement

  edit file.rs:150      — open at line 150
  edit file.rs:fn_main  — open at first match

## Zero /tmp Workflow

search fn_expand     — find it
query main.rs 46:80  — read it
patch main.rs --old X --new Y  — fix it
query main.rs 46:80  — verify it

Zero temp files. Zero redirects. Zero friction.

## Prerequisites
INT-194 fsh v4 — complete
fsh foundation fixes — complete (this session)

## Build Order
Phase 1 — query builtin
Phase 2 — query with pattern matching
Phase 3 — fsh run command
Phase 4 — edit builtin enhancement
Phase 5 — search builtin
Phase 6 — patch builtin
Phase 7 — native pipe chains
Phase 8 — show builtin

## Gate Check

✅ query file.rs 100:150 extracts lines correctly (2026-04-11)
✅ query file.rs pattern finds and shows context (2026-04-11)
✅ query file.rs :50 first N lines (2026-04-11)
✅ query file.rs 900: line N to end (2026-04-11)
⬜ fsh run file.py executes Python script
⬜ fsh run file.fsh executes fsh native script
⬜ edit file.rs:150 opens editor at line
⬜ edit file.rs:pattern opens editor at match
✅ search pattern recursive structured output (2026-04-11)
✅ search pattern --type rs filtered by type (2026-04-11)
⬜ patch file.rs --old X --new Y in-place replacement
⬜ native pipe: grep | head without sh fallback
⬜ native pipe: cat | grep without sh fallback
⬜ zero /tmp workflow verified end to end
⬜ d passes 100% after full implementation

## The Phrase

"Every temp file is an apology.
Every redirect is a workaround.
Every sh fallback is a gap in the forest.

INT-223 closes the gaps.
The shell becomes the tool it was always meant to be:
not a wrapper around other tools
but a first-class development environment
that understands what you are building." 🌲
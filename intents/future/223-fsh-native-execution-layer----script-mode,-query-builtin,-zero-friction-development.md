# INT-223 — fsh Native Execution Layer — Script Mode, Query Builtin, Zero Friction Development
Status: [planned]
Date: 2026-04-10
Tags: fsh, shell, scripting, query, builtin, developer-experience, zero-friction, native

## The Problem We Solve Every Single Session

Every time we read lines from a file: head/tail/tmp dance. Three commands, two temp files.
Every time we run Python: write to /tmp, execute, clean up.
Every time we search with grep: redirect to /tmp because pipes are fragile.
This is not how a forest-native shell should work. INT-223 eliminates all of it.

## The query Builtin

The single biggest productivity improvement for development sessions.

  query file.rs 100:150        — print lines 100-150
  query file.rs 100:+30        — print 30 lines starting at 100
  query file.rs fn_main        — lines containing fn_main
  query file.rs :50            — first 50 lines
  query file.rs 900:           — line 900 to end

Replaces the entire head/tail/grep pipeline.
One command. No temp files. No pipes needed.
Implementation: pure Rust builtin in fsh commands/mod.rs.

## The fsh run Command

Executes a script file natively:
  fsh run file.py   — python3
  fsh run file.sh   — sh
  fsh run file.fsh  — fsh native scripting

Eliminates the write-to-tmp-then-execute pattern entirely.

## The edit Builtin Enhancement

  edit file.rs          — open in editor
  edit file.rs:150      — open at line 150
  edit file.rs:fn_main  — open at first fn_main match

Replaces: grep to find line number, then open editor at that line.

## The patch Builtin

The most repeated operation in development sessions.
Find exact text, replace with new text, write back.

  patch file.rs --old "old_text" --new "new_text"

Eliminates 90% of the Python fix scripts written every session.

## The search Builtin

Native recursive search across the codebase:

  search fn_expand           — all .rs files under current dir
  search fn_expand --type rs — only .rs files
  search fn_expand --file main.rs — only in main.rs

Returns structured output: file:line match
No grep. No redirects. No temp files.

## The show Builtin

  show file.rs          — preview with line numbers
  show file.rs:fn_main  — show function at match
  show file.rs:100:150  — show lines with context

## Zero /tmp Workflow

After INT-223 the development workflow becomes:
  search fn_expand_subshells  — find it
  query main.rs 46:80         — read it
  edit main.rs:46             — edit it
  patch main.rs --old X --new Y — fix it
  query main.rs 46:80         — verify it

Zero temp files. Zero redirects. Zero friction.

## Native Pipe Fixes

fsh detects common pipe chains and uses Rust process pipe chaining
instead of delegating to sh. No subprocess boundary. Faster. More predictable.

## Prerequisites
INT-194 fsh v4 — complete
fsh foundation fixes — complete (this session)
fsh execute() tokenizer — quote-aware (this session)

## Build Order
Phase 1 — query builtin (line range extraction)
Phase 2 — query with pattern matching
Phase 3 — fsh run command (script file execution)
Phase 4 — edit builtin enhancement (line/pattern focus)
Phase 5 — search builtin (recursive codebase search)
Phase 6 — patch builtin (find-and-replace in files)
Phase 7 — native pipe chains
Phase 8 — show builtin (preview with context)

## Gate Check
⬜ query file.rs 100:150 — extracts lines correctly
⬜ query file.rs pattern — finds pattern and shows context
⬜ query file.rs :50 — first N lines
⬜ query file.rs 900: — line N to end
⬜ fsh run file.py — executes Python script file
⬜ fsh run file.fsh — executes fsh native script
⬜ edit file.rs:150 — opens editor at line
⬜ edit file.rs:pattern — opens editor at pattern match
⬜ search pattern — recursive search, structured output
⬜ search pattern --type rs — filtered by file type
⬜ patch file.rs --old X --new Y — in-place replacement
⬜ native pipe: grep | head works without sh fallback
⬜ native pipe: cat | grep works without sh fallback
⬜ zero /tmp workflow verified end to end in a real session
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
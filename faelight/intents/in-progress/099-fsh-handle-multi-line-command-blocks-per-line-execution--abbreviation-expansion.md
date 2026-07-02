---
id: 099
date: 2026-06-29
type: future
title: "fsh: handle multi-line command blocks (per-line execution + abbreviation expansion)"
status: in-progress
tags: [fsh, blocks, command line]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

---


## Why
Pasting multiple commands as a block runs them concatenated, and abbreviations
(gp, gc, dep, ds, dc) do NOT expand inside the block -- they come back as
"command not found". Observed repeatedly: pasting `git add` / `git commit` / `gp`
as three lines fails on the `gp` line. Forces a slow, error-prone
one-command-per-line workflow for every multi-step sequence.

## Desired behaviour
- fsh parses pasted multi-line input LINE BY LINE, treating each line as if typed
  at the interactive prompt: expand abbreviations, then run, in sequence.
- The common case (several independent commands pasted together) just works.

## The hard part (design question to solve first)
Must distinguish "a sequence of independent commands" from "ONE multi-line command":
- Heredocs: `cat > /tmp/x.py << 'EOF' ... EOF` -- the body lines are NOT separate
  commands; must be collected until the closing delimiter.
- Python/shell blocks fed via heredoc, `for`/`while`/`if` multi-line constructs,
  backslash line-continuations -- all must stay together.
- Naive line-splitting would break every heredoc workflow we rely on (the Python
  edit pattern used all session). So the parser must track heredoc/continuation
  state and only split at TRUE command boundaries.

## Approach (rough)
- Find fsh's REPL input path (rust-tools/faelight-shell/src/ -- main loop / line
  reader). Identify where pasted input arrives as one chunk.
- Pre-process: split into logical commands, respecting heredoc delimiters
  (`<< 'TAG'` ... `TAG`), line-continuations (`\`), and open block constructs.
- For each logical command: run the existing prompt-path (so abbreviation
  expansion + alias handling apply uniformly).
- Preserve current single-command behaviour exactly; this is additive.

## Gates (demonstrated, not declared)
- Pasting `git add X` / `git commit -m "..."` / `gp` as a block runs all three,
  `gp` expands and pushes.
- A heredoc Python block pasted in still executes as ONE command (not split).
- A `for` loop pasted across lines runs as one construct.
- Single commands behave exactly as before (no regression).

## Notes
Discovered during the 2026-06-29 metal-tuigreet session (came up every time a
multi-line git/python block was pasted). Daily papercut; fix improves every
multi-step workflow. Touches fsh input parsing -- do with a clear head; the
multi-command-vs-multi-line-command distinction is the crux.

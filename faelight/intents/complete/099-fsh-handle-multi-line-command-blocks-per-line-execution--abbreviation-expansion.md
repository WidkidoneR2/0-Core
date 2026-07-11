---
id: 099
date: 2026-06-29
type: future
title: "fsh: handle multi-line command blocks (per-line execution + abbreviation expansion)"
status: complete
tags: [fsh, blocks, command line]
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
- [x] Pasting `git add X` / `git commit -m "..."` / `gp` as a block runs all three,
  `gp` expands and pushes. <!-- INT-130 2026-07-10: done. Impl commit 3c170e2a (split_into_commands + per-line abbreviation expansion), completed 3fcdde34. Demonstrated continuously this session -- every commit was a pasted block whose `gp` expanded and pushed. -->
- [x] A heredoc Python block pasted in still executes as ONE command (not split). <!-- INT-130 2026-07-10: done. is_complete_command keeps heredoc bodies glued. Demonstrated all session -- every python patch was a `cat > /tmp/x.py << 'PYEOF' ... PYEOF; python3` block that ran as one command. -->
- [x] A `for` loop pasted across lines runs as one construct. <!-- INT-130 2026-07-10: VERIFIED LIVE -- pasted a 4-line `for i in 1 2 3 / do / echo / done` block; ran as one loop, output line 1/2/3. -->
- [x] Single commands behave exactly as before (no regression). <!-- INT-130 2026-07-10: additive change (commit 3c170e2a: zero changes to existing execute sites); single-command behaviour unchanged, confirmed by daily use since 2026-06-29. -->

## Notes
Discovered during the 2026-06-29 metal-tuigreet session (came up every time a
multi-line git/python block was pasted). Daily papercut; fix improves every
multi-step workflow. Touches fsh input parsing -- do with a clear head; the
multi-command-vs-multi-line-command distinction is the crux.

<!-- Gates reconciled per INT-130, 2026-07-10: GENUINE reconcile + CHARTER REPAIR. Charter was malformed -- a dead template stub (Vision/Problem/Solution placeholders + junk '- [ ] ...') above the real content, and real gates written as prose bullets not checkboxes. Removed the stub; converted 4 gates to [x] with evidence. Work confirmed: NixOS-era impl commits 3c170e2a + 3fcdde34 (NOT the Arch-era Niri 099 in the log); for-loop gate VERIFIED LIVE; heredoc/block/gp demonstrated all session. (Cosmetic note: frontmatter type:future while status:complete -- left as-is, future hygiene.) 8/23. -->

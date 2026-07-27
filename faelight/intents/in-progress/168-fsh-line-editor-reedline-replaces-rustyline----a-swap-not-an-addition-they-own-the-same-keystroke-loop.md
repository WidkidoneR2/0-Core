---
id: 168
date: 2026-07-16
type: arch
title: "fsh line editor: reedline replaces rustyline -- a swap, not an addition (they own the same keystroke loop)"
status: in-progress
tags: [fsh, reedline, rustyline, line-editor, ux, phase2]
---

## Vision
Replace rustyline with reedline: better multiline editing, hinting, menus, and syntax-highlight hooks
for a shell that wants to be more than readline-with-a-prompt.

## THIS IS A SWAP, NOT AN ADDITION
Both cannot be used. They own the SAME THING: the loop that reads every keystroke. Running both means
two editors fighting for the terminal. MEASURED 2026-07-16: fshs Cargo.toml lists `rustyline = "17.0.2"`
among 14 dependencies. Adding reedline means deleting rustyline in the same commit.

## Why reedline
It is nushells line editor, built for exactly the kind of shell fsh is trying to be -- structured,
modern, interactive-first. rustyline is a good traditional readline binding and has served fine.
The gap is not correctness, it is ceiling: multiline editing, completion menus, hinting, and
highlighting are things reedline was designed for and rustyline was not.

## The honest risk
This is the code BETWEEN CHRISTIANS FINGERS AND THE SHELL, and fsh is the daily driver. Every
keystroke, every history recall, every Ctrl-C, every tab-complete goes through it. A regression here
is not a bug report, it is an unusable terminal.
MITIGATED BY WHAT ALREADY WORKS: the login shell is BASH, not fsh (proven by getent 2026-07-16), so
fsh is the TERMINAL, not the door. SafeShell is in the greeter. Generation rollback is one command.
And the debug-binary-as-child pattern (build, run ./target/debug/faelight-shell as a nested shell,
test, exit) means a broken editor never reaches metal -- that pattern caught a real regression during
INT-143 and is mandatory here.

## Sequencing
AFTER INT-171. 171 gives ONE parsing entry point; swapping the editor while four parsers still exist
means a regression could come from either change and you would not know which. One variable at a time.

## What must not regress -- the real gate list
- Ctrl-C, Ctrl-D, Ctrl-L, arrows, tab-completion, reverse-search
- 285 aliases still expand at the prompt
- history persists across sessions and across `reload` (fshs hot-swap)
- the prompt renders: git branch, intent focus, timing, the health/notification lines
- multiline paste of a heredoc block still works (this is how every python3 << PYEOF edit lands)
- fsh -c "cmd" still works (INT-190: niri-session boot depended on it)

## Success Criteria
- [ ] rustyline is REMOVED from Cargo.toml in the same commit reedline is added. Not both. Prove it
      with the diff
- [ ] Every item in "What must not regress" verified on the DEPLOYED binary, not target/debug
      (INT-110: "a cargo build alone shows green while the live command still fails")
- [ ] Multiline heredoc paste works -- tested with a real python3 << PYEOF block, since that is the
      mechanism every source edit in this repo uses
- [ ] At least one thing reedline does that rustyline could not, DEMONSTRATED. If nothing, this
      intent was churn and should be cancelled rather than shipped
- [ ] Each gate carries evidence per INT-158

## The Rule
"They own the same keystroke loop. There is no both." 🌲

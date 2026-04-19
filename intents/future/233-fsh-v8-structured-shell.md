---
id: 233
title: "fsh v8 -- Structured Shell: Data, Syntax, Intelligence"
status: in-progress
date: 2026-04-16
tags: [fsh, shell, structured-data, syntax, ux, errors, performance, v8]
---
fsh has grown organically -- builtins added session by session.
v8 is a deliberate upgrade.
Not Nu-shell. But informed by what Nu-shell gets right.
A shell that handles structured data, speaks clearly,
recovers from errors gracefully, and grows with Friday.
- Structured output: commands return data, not just text
- Consistent syntax: no surprising edge cases
- Clear errors: tell you exactly what went wrong and why
- Readable pipelines: you can see what data flows where
Every error message tells you: what failed, why, and what to do
No silent failures -- every failed command reports clearly
✅ echo 'text' > /tmp/file works natively in fsh (2026-04-19)
rspatch newline handling: \n in --new interpreted as real newline
Heredoc improvements: better error messages for common mistakes
Command not found: suggest closest known builtin or alias
JSON output mode: command | json produces parseable output
Table display: core knowledge patterns shows as formatted table
Filter syntax: command | where field = value
Select syntax: command | select field1 field2
These work on fsh builtins first, extend to external commands
Pipeline visualization: show data flow before executing
Type-aware pipes: warn when text is piped where data expected
Friday-aware pipelines: Friday suggests next pipe stage
Error recovery in pipelines: partial failure reports clearly
config.fsh improvements: better syntax, validation on load
Shell startup time: must be under 100ms cold start
History intelligence: frequency-weighted completion
Abbreviation expansion: more abbreviations, context-aware
After every failed command: fsh queries knowledge engine
If known fix exists (confidence >= 0.85): shows fix inline
fsh diag improvements: more specific gap detection
fsh gaps: shows frequency of old-habit commands with alternatives
⬜ faelight-term -- cat large output renders correctly (no corruption)
⬜ faelight-term -- Ctrl+Click URL opens reliably (modifier state fix)
⬜ faelight-term -- heredoc inside faelight-term works via fsh not sh

✅ echo 'text' > /tmp/file works natively in fsh (2026-04-19)
✅ rspatch \\n in --new content interpreted as real newline (2026-04-19)
✅ Every error message includes: what failed, why, what to do -- rspatch and deploy done (2026-04-19)
✅ Command not found suggests nearest known alternative (2026-04-19)
✅ JSON output mode -- core knowledge patterns pipes cleanly to head/grep (2026-04-19)
✅ Table display -- git-commits | where author == christian | first 3 works (2026-04-19)
✅ Filter syntax -- command | where field == value demonstrated live (2026-04-19)
✅ Pipeline error recovery -- fsh builtins now pipe to external commands, error shows which stage failed (2026-04-19)
✅ config.fsh syntax validation on load -- errors with line number and fix shown (2026-04-19)
✅ Shell startup under 100ms -- measured 4ms cold start (2026-04-19)
✅ fsh queries knowledge engine on build/command failure -- fires on every error (2026-04-19)
✅ Known fix shown inline when confidence >= 0.85 -- demonstrated with rspatch (2026-04-19)
✅ fsh gaps updated -- rspatch, fsh-patch, sed alternatives shown (2026-04-19)
✅ fsearch supports basic regex alternation -- pipe-separated patterns work (2026-04-19)
✅ fsh code injection helper -- fsh-patch script for safe Rust patching via temp files (2026-04-19)
✅ gp abbreviation wired -- now expands to git push correctly (2026-04-19)
✅ fg sync clippy scope -- clippy now scoped to staged packages only (2026-04-19)
✅ rspatch and patch handle em dashes -- fsh-patch helper bypasses Python unicode issues (2026-04-19)
⬜ shell friction audit -- review all session pain points before v8 build

"The shell is not just a command runner.
It is the interface between you and the forest.
v8 makes that interface intelligent." 🌲

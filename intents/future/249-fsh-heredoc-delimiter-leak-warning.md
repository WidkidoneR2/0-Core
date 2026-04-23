---
id: 249
title: "fsh heredoc delimiter leak warning"
status: planned
date: 2026-04-23
type: fix
tags: [fix, fsh, heredoc, safety, friction]
version: 11.9.0
---
fsh's heredoc handler is correct when delimiters match, but silent when they do not.
A malformed heredoc lets the delimiter token leak into stdout and, worse,
into files being written. This has caused real corruption:
- Yesterday (INT-232 session): RSEOF appeared literally inside a Rust source
  file because a heredoc terminator was misplaced. The build failed before
  anything shipped, but the source was briefly corrupt.
- Today (INT-248 migration): multiple heredocs ran across sessions with
  PYEOF, MIGRATION_EOF, CONSOL_EOF, NEWASK_EOF. Any one of them misaligned
  would have been silent until something downstream failed.
This intent adds a detection: when a line in command output matches the
shape of a heredoc delimiter, fsh prints an inline warning. Humans catch
the problem in the moment, not three steps later.
SCOPE
Narrow and focused. One behavior, one file, one demonstrable test.
This intent is NOT:
- A heredoc parser rewrite
- A full static analysis of shell scripts
- A warning for every uppercase token in output
This intent IS:
- A post-execution scan of stdout for lines matching a specific pattern
- A colored inline warning when the pattern fires
- Narrow enough that false positives are rare
PATTERN
Detect lines matching ^[A-Z_]{3,}EOF$ in command output.
This catches every delimiter we have actually used:
  PYEOF, RSEOF, MIGRATION_EOF, CONSOL_EOF, NEWASK_EOF
Plus the common convention EOF itself is 3 characters so is excluded by
the {3,}EOF anchor -- we require a prefix of at least 3 uppercase/underscore
characters before EOF to avoid false positives on plain "EOF".
Actual false-positive surface:
- Legitimate output like "added PYEOF to documentation" does NOT match --
  the pattern requires the whole line to be the token, not a substring.
- A script echoing the literal word "MYEOF" on its own line would fire the
  warning. This is acceptable: the user sees it, confirms it was intended,
  and moves on. Conservative pattern beats silent corruption.
BEHAVIOR
When a matching line is detected in command output:
  ⚠  possible unclosed heredoc -- "RSEOF" appeared as standalone output line
Printed in amber, after the offending line, before the next line of real
output. Does not block execution. Does not modify the output stream seen
by downstream commands (pipes still work correctly).
On by default. The pattern is narrow enough that false positives are rare,
and the cost of missing the warning is high.
IMPLEMENTATION GATES
⬜ Identify the output-processing point in fsh where post-command stdout can be inspected line-by-line
⬜ Add detection for lines matching ^[A-Z_]{3,}EOF$ regex
⬜ Print amber inline warning after the offending line
⬜ Warning does not modify the output stream for pipes or redirections
DEMONSTRATION GATES
⬜ Run a deliberately broken heredoc, warning fires with the correct delimiter name
⬜ Run echo "we have PYEOF in our docs" -- warning does NOT fire (substring match)
⬜ Run legitimate heredoc (proper close) -- warning does NOT fire
⬜ Run command with output piped to another command -- pipe still works, warning still appears in interactive output
BLOCKS
None. Independent friction fix.
"A shell that catches its own knives before they fall." 🌲

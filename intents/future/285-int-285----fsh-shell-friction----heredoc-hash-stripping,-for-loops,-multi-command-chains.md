---
id: 285
title: "fsh shell friction -- heredoc hash stripping, for loops, knowledge add hang, command chains"
status: planned
date: 2026-05-08
tags: [fsh, shell, bugfix, heredoc, friction, loops, commands]
---

# INT-285 -- fsh Shell Friction

Status: planned | Date: 2026-05-08
Tags: fsh, shell, bugfix, heredoc, friction, loops

Four friction points from session 2026-05-08.
Each has reproduction case, root cause, fix approach.

---

BUG 1 -- HEREDOC STRIPS LINES STARTING WITH HASH (HIGH)

Reproduction: cat > /tmp/test.py with PYEOF delimiter.
Lines starting with hash-hash get stripped from output.
Root cause: fsh is_comment() fires on heredoc body lines.
Fix: disable comment stripping inside heredoc body.
Workaround: use python3 -c or avoid hash headers in heredoc.

BUG 2 -- FOR LOOPS SPLIT INTO SEPARATE COMMANDS (HIGH)

Reproduction: for f in a b c; do echo $f; done
fsh splits at semicolons -- for/do/done not recognized.
Root cause: is_complete_command() missing for/do/done context.
Fix option 1: buffer for loop until done is seen.
Fix option 2: detect for/while/until and pass to sh.
Workaround: python3 -c loop or write to /tmp/script.sh

BUG 3 -- core knowledge add HANGS ON ARGUMENT INPUT (HIGH)

Reproduction: core knowledge add "some fact"
Hangs waiting for stdin. Requires Ctrl+C.
Root cause: reads stdin even when argument provided.
Fix: use argument directly, only prompt when no arg given.
Workaround: direct sqlite3 INSERT via Python script.

BUG 4 -- MULTI-COMMAND CHAINS INCONSISTENT (MEDIUM)

Symptoms: && chains sometimes skip second command.
; separators split instead of chain.
Specific: deploy core && deploy faelight-shell
sometimes only runs first deploy.
Fix: audit && and ; handling in multi-command executor.

---

GATES

[ ] heredoc body not subject to comment stripping
[ ] for f in ...; do ...; done works or routes to sh
[ ] core knowledge add returns immediately with arg
[ ] deploy core && deploy faelight-shell always runs both
[ ] Session runs without /tmp Python workarounds

The shell should not fight you.
Every workaround is a failure of the tool.
fsh fixes its own friction. The forest speaks human first.
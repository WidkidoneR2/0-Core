---
id: 176
date: 2026-03-30
type: future
title: "Failure Recovery — The Shell Remembers What Went Wrong"
status: in-progress
tags: [shell, fsh, errors, recovery, retry, intelligence, observability]
version: 12.0.0
priority: high
depends_on: [146, 174]
---

## The Problem
When a command fails in faelight-shell, the failure is forgotten.
No retry. No fix. No explanation. No history of failures.
```fsh
fsh ❯ deploy core
❌ permission denied

fsh ❯ # now what? the error is gone
```

This is the opposite of a self-aware shell.
A self-aware shell remembers what went wrong
and helps you recover from it.

"Why did this fail?" is one of the four questions
that define shell self-awareness. Without failure memory,
this question can never be answered.

## The Solution
The shell remembers every failure this session
and gives you tools to recover:
```fsh
last_command retry        # re-run the exact last command
last_command fix          # suggest a corrected version
last_command explain      # why did this fail? (uses INT-174)
history failures          # all failed commands this session
history failures | last 5 # last 5 failures with errors
```

## Retry Semantics
```fsh
last_command retry
# Re-runs the last failed command exactly as typed.
# Useful after: fixing a permission, changing directory,
# unlocking core, installing a dependency.

last_command retry --with "unlock-core &&"
# Prepend a fix before retrying.
```

## Fix Suggestions
```fsh
last_command fix
# Uses structured error (INT-174) to suggest correction:
#
# Last command: gc "update"
# Error: E_NOT_GIT_REPO
# Suggested fix: cd ~/0-core && gc "update"
```

## Failure History
```fsh
history failures
# Shows all failures this session:
# [09:14] gc "update"          E_NOT_GIT_REPO
# [09:22] deploy core          E_PERMISSION (core locked)
# [09:31] cargo build          E_EXIT_NONZERO (exit 1)

history failures | last 5
history failures | where code == "E_PERMISSION"
```

## Integration With Core Intelligence
Failure patterns feed core v11 predictions:
```
You hit E_PERMISSION 3 times this week before deploy.
Prediction: run unlock-core before deploy.
```

Reaction rules can fire on repeated failures:
```
3 failures in 10 minutes → health advisory
```

## Phase 1 — Failure Memory
Store failed commands + structured errors in session state.
`history failures` reads from session state.

## Phase 2 — last_command retry
Re-run the last failed command from session state.

## Phase 3 — last_command explain
Use structured error (INT-174) to explain the failure in plain language.

## Phase 4 — last_command fix
Analyze the error code and suggest a corrected command.
Start with the most common error codes:
E_NOT_GIT_REPO, E_PERMISSION, E_CMD_NOT_FOUND, E_CORE_LOCKED.

## Phase 5 — Failure Pattern Feed
Feed failure history into core v11 and v12.
Failures become training data for predictions and strategy.

## Gate Check
```
✅ Failed commands stored in session state — failure_log_NNN keys in shell_state (2026-03-31)
✅ failures command — session failure log as structured table (2026-03-31)
✅ failures pipeable — works with all pipeline operators (2026-03-31)
✅ last_command retry — shows command and prompts re-run (2026-03-31)
✅ last_command explain — shows error code, message, suggestion (2026-03-31)
✅ last_command fix — suggests fixes for E_CORE_LOCKED/E_NOT_GIT_REPO/E_PERMISSION (2026-03-31)
✅ Failure patterns stored — failure_log available for core v11/v12 analysis (2026-03-31)
✅ Failures in shell_state — reaction rules can query failure_log keys (2026-03-31)
```

## The Phrase
**"The shell that forgets its failures
is doomed to repeat them.
Remember every failure.
Name it. Explain it. Recover from it.
The forest does not abandon the fallen branch —
it learns from where it broke."**

---
*"retry is not stubbornness.
fix is not automation.
explain is not documentation.
Together they are: a shell that learns."* 🌲

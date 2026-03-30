---
id: 174
date: 2026-03-30
type: future
title: "Structured Errors — The Shell Explains Its Failures"
status: planned
tags: [shell, fsh, errors, debugging, observability, intelligence]
version: 12.0.0
priority: high
depends_on: [146, 173]
---

## The Problem
When a command fails in faelight-shell, the error is plain text.
No code. No context. No suggestion. The failure is forgotten the moment
it scrolls past.
```
fsh ❯ gc "update"
error: not a git repository
```

That is all you get. No structure. No memory. No path forward.

This violates the self-aware shell principle:
"Why did this fail?" must have a real answer.

## The Solution
Every error in faelight-shell becomes a structured value:
```
Error {
    code:       "E_NOT_GIT_REPO",
    message:    "Not a git repository",
    suggestion: "Run this command from inside ~/0-core or another git repo",
    context:    { command: "gc", directory: "~/Downloads" },
    timestamp:  2026-03-30T09:14:00Z,
}
```

## Error Codes
```
E_CMD_NOT_FOUND     — command not in PATH or registry
E_PERMISSION        — permission denied
E_NOT_GIT_REPO      — git operation outside a repo
E_PIPE_EMPTY        — no input to pipeline stage
E_PARSE_FAILED      — .fsh script parse error
E_CORE_LOCKED       — attempted write while core is locked
E_EXIT_NONZERO      — external command exited with error code
E_TIMEOUT           — command exceeded time limit
```

## Query Interface
```fsh
last_error              # show last structured error
last_error explain      # full explanation with context
last_error suggest      # just the suggested fix
history errors          # all errors this session
history errors | last 5 # last 5 errors
```

## Integration With Before_Run (INT-171)
before_run can fire on error patterns:
```fsh
on_error E_NOT_GIT_REPO {
    suggest "Navigate to a git repo first: 0core"
}
```

## Phase 1 — Error Type
Define the Error struct in faelight-shell.
Replace plain string errors with structured errors throughout commands/mod.rs.

## Phase 2 — Error Display
Structured errors display clearly:
```
❌ E_NOT_GIT_REPO: Not a git repository
   💡 Run this from inside ~/0-core or another git repo
```

## Phase 3 — Error Memory
Store last N errors in session state.
`last_error` reads from session state.

## Phase 4 — Error Query
```fsh
history errors
last_error explain
last_error suggest
```

## Phase 5 — Error Patterns
Feed error history into core v11 predictions:
"You frequently hit E_NOT_GIT_REPO in ~/Downloads —
consider adding a guard."

## Gate Check
```
⬜ Error struct defined — code, message, suggestion, context, timestamp
⬜ All builtins return structured errors
⬜ External command failures wrapped in structured errors
⬜ Error display shows code + message + suggestion
⬜ last_error shows most recent structured error
⬜ last_error explain shows full context
⬜ history errors shows session error log
⬜ Error codes documented in registry (INT-173)
⬜ before_run (INT-171) can pattern match on error codes
```

## The Phrase
**"An error that disappears
is an error that repeats.
Structure the failure.
Remember the context.
The shell that explains itself
is the shell that improves itself."**

---
*"E_CMD_NOT_FOUND is not a wall.
It is a question: what were you trying to do?
Answer that and you have intelligence."* 🌲

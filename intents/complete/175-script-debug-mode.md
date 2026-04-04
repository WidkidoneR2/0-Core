---
id: 175
date: 2026-03-30
type: future
title: "Script Debug Mode — Trace Every Step"
status: complete
tags: [shell, fsh, scripting, debug, trace, observability]
version: 12.0.0
priority: medium
depends_on: [146, 174]
---

## The Problem
.fsh scripts run silently. When something goes wrong there is no way
to see what happened without adding print statements manually.
```fsh
run deploy.fsh
# output: nothing
# result: broken
# diagnosis: impossible
```

This makes .fsh scripts hard to trust for real workflows.
A script you cannot debug is a script you cannot rely on.

## The Solution
A --trace flag that shows every step as it executes:
```fsh
run deploy.fsh --trace
```

Output:
```
[1] let tool = "core"          ✅  (0ms)
[2] let version = "2.0.0"      ✅  (0ms)
[3] run cargo build --release  ✅  (8.2s)
[4] emit "deploy.started"      ✅  (0ms)
[5] run cp target/core scripts ❌  exit 1 (permission denied)
    → E_PERMISSION: Core is locked
    → Suggestion: run unlock-core first
```

## Modes
```fsh
run script.fsh --trace     # show each step with timing and result
run script.fsh --dry-run   # show what would run without executing
run script.fsh --verbose   # show variable values at each step
run script.fsh --step      # pause after each step (interactive)
```

## Trace Output Format
```
[N] <statement>    ✅ / ❌   (<timing>)
    → <error code if failed>
    → <suggestion if failed>
    → <variable values if --verbose>
```

## Integration With Structured Errors (INT-174)
--trace uses the structured error system for failure output.
Every failed step shows the full Error struct in readable form.

## Integration With Failure Recovery (INT-176)
After a traced run, `last_error` holds the first failure point.
`last_command retry` can re-run from the failure point.

## Phase 1 — Trace Flag
Add --trace parsing to the `run` builtin.
Wrap each script statement in a timer + result capture.

## Phase 2 — Dry Run
Add --dry-run: resolve variables and show steps without executing.
Useful for verifying scripts before running in production.

## Phase 3 — Verbose Mode
Show variable values at each step.
Show pipeline contents between stages.

## Phase 4 — Step Mode
Pause after each step, show state, wait for enter to continue.
Allows interactive debugging of complex scripts.

## Gate Check
```
⬜ run script.fsh --trace shows each step with result
⬜ Timing shown for each step
⬜ Failed steps show structured error (INT-174)
⬜ run script.fsh --dry-run shows steps without executing
⬜ run script.fsh --verbose shows variable values
⬜ run script.fsh --step pauses after each step
⬜ Trace output is clean and readable
⬜ --trace adds < 10ms overhead per step
```

## The Phrase
**"A script you cannot trace
is a script you cannot trust.
--trace turns the black box transparent.
Every step visible. Every failure named.
The script becomes a story you can read."**

---
*"Debug mode is not for broken scripts.
It is for understanding scripts that work —
so you know why they work
and can fix them when they don't."* 🌲

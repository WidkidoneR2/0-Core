---
id: 023
date: 2026-01-10
type: future
title: "Health Engine - Dependency Graph Model"
status: planned
tags: [rust, health, dot-doctor, v6.4.0]
---
## The Vision
Evolve dot-doctor from "run checks" to "model system integrity" with dependency awareness.

## Why
- Detect root cause, not just symptoms
- Explain failures intelligently
- Machine-consumable output

## Internal Model
```rust
struct Check {
    id: &'static str,
    depends_on: &'static [&'static str],
    severity: Severity,
    run: fn() -> HealthResult,
}

enum HealthResult {
    Ok,
    Warning { reason: String },
    Failed { reason: String, fix: Option<String> },
}
```

## New Commands
```bash
faelight health              # Standard check
faelight health --explain    # Root cause analysis
faelight health --json       # Machine output
faelight health --fail-on warning  # CI mode
faelight health --graph      # Show dependency tree
```

## Dependency Graph Example
```
stow-symlinks
├── shell-config (depends on stow)
├── sway-config (depends on stow)
└── theme-packages (depends on stow)

services
├── faelight-bar
└── faelight-notify
```

If `stow-symlinks` fails, children are skipped with "blocked by parent failure".

## Success Criteria
- [ ] Checks have dependencies defined
- [ ] --explain shows root cause
- [ ] --json output format
- [ ] --fail-on for CI
- [ ] Failure propagation

---
_Understand the forest's health._ 🌲

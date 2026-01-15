---
id: 004
date: 2025-12-31
type: future
title: "Git Guardrails - v3.5.1"
status: complete
tags: [git, safety, v3.5.1, guardrails]
---

## Why

0-Core is locked for a reason - immutability protects against accidents.
But filesystem locks don't prevent git operations.

Problem: You can commit while core is locked (violates immutability).
Problem: You can push directly to main (risky for stable system).

Git Guardrails enforces philosophy at the git level.

## What

Git wrapper that prevents dangerous operations:

1. **Block commits when core is locked**

```
   git commit -m "test"
   🔒 0-core is LOCKED
   ❌ Commit blocked to protect immutable core
   💡 Run: unlock-core
```

2. **Warn on push to main**

```
   git push
   ⚠️  Pushing directly to MAIN in 0-core
   Proceed? (type 'push-main'):
```

3. **Escape hatch for experts**

```bash
   alias git!='/usr/bin/git'  # Bypass guardrails
```

## Philosophy Alignment

- **Manual control** - Explicit confirmation required
- **Intentionality** - No accidental operations
- **Safety** - Prevents mistakes without blocking experts
- **Understanding** - Forces you to think before acting

The `!` escape hatch is psychological - it PROVES intention.

## Implementation

Shell function wrapper around git:

- Only active in ~/0-core
- Checks lsattr for lock status
- Requires typed confirmation for main pushes
- Uses `command git` to preserve full functionality

## Timeline

v3.5.1 (early January 2026)
After Intent Ledger (v3.5.0)
Before Shell Polish (v3.5.2)

## Impact

**Positive:**

- Prevents accidental commits to locked core
- Prevents thoughtless main pushes
- Enforces discipline
- Protects against muscle memory mistakes

**Negative:**

- Extra friction (intentional!)
- Experts must use git! for bypass

## Why Between 3.5.0 and 3.5.2

Logical progression:

- v3.5.0: Build memory (Intent Ledger)
- v3.5.1: Enforce safety (Git Guardrails)
- v3.5.2: Polish experience (Shell improvements)

Foundation → Safety → Polish

### 2026-01-01

Status changed to complete.
v3.5.1 shipped with full Git Guardrails implementation.
All tests passing. The forest protects itself. 🔒

[Initial creation]

---

_Part of the 0-Core Intent Ledger_ 🌲

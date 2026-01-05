---
id: 005
date: 2026-01-04
type: future
title: ""Intent Ledger Integration - v3.6.0""
status: planned
tags: [[intent-ledger, dot-doctor, v3.6.0]]
---

## Why

Intent Ledger exists but is disconnected from other tools.
No visibility into intent health during daily workflow.
Easy to forget updating status when features ship.

## What

Five improvements:

1. **Add `packages:` field**
   - Optional frontmatter field
   - Links intents to related packages
   - Enables validation

2. **Add `version:` field**
   - Target version in frontmatter
   - Enables tag/status mismatch detection

3. **dot-doctor Check 11: Intent Health**
   - Count by status (complete, planned, in-progress)
   - Flag stale intents (planned > 30 days)
   - Show totals

4. **Package reference validation**
   - If `packages:` exists, verify each package exists in 0-core
   - Warn on invalid references

5. **Version/tag mismatch detection**
   - If `version:` matches a git tag but status != complete
   - Warn: "v3.5.2 is tagged but intent still planned"

## Example Output

```
📜 Checking Intent Ledger...
   Total: 7 intents
   ✅ Complete: 4
   📅 Planned: 2
   🔨 In-progress: 1
   ⚠️ Stale: intent 001 (planned > 30 days)
   ✅ All package references valid
```

## Constraints

- No enforcement, only warnings
- Optional fields (don't break existing intents)
- ~1.5 hours estimated

## Timeline

v3.6.0 (January 2026)

## Impact

- Intent health visible in daily workflow
- Catches forgotten status updates
- Validates package references
- Facts over assumptions

---

_Part of the 0-Core Intent Ledger_ 🌲

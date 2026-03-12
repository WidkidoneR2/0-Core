---
id: 119
date: 2026-03-12
type: future
title: "core security advise — judgment layer for security decisions"
status: planned
tags: [security, core-v6, advise, judgment, decisions, v10.8]
version: 10.8.0
priority: high
---

## Vision

Apply the Core v6 judgment layer specifically to security decisions.

Right now `core security scan` reports findings.
`core security advise` would contextualize them using decision history.

## What It Does
```bash
core security advise
```

Example output:
```
🛡️ Security Advisory
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Current findings: 19 (all upstream pending)
  Last scan: 2 days ago

  Historical pattern:
    Decisions made with scan age > 7 days → partial 2/3 times
    Current scan age: 2 days — within safe window

  Advice:
    → Scan age acceptable — no action needed
    → Next recommended scan: in 5 days
```

## Integration Points

- Reads from `core security` domain
- Queries `decisions` table for security-related patterns
- Uses Core v6 context hash matching
- Surfaces as `core security advise` subcommand

## Success Criteria

- [ ] `core security advise` command working
- [ ] Reads scan age from last security event in state.db
- [ ] Correlates with historical security decisions
- [ ] Integrates with Core v6 advisory pattern

---
*"Security without context is noise. Security with history is judgment."* 🌲

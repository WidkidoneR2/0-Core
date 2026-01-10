---
id: 024
date: 2026-01-10
type: future
title: "faelight-git - Git as Governance Layer"
status: planned
tags: [rust, git, policy, v6.4.0]
---
## The Vision
Git stops being a storage tool and becomes a governance layer. Shell hooks die here.

## Why
- Git hooks in shell are fragile
- Intent enforcement should be automatic
- Policy violations caught before push

## Commands
```bash
faelight git install-hooks   # Setup hooks
faelight git verify          # Manual verification
faelight git status          # Policy compliance
```

## Hooks Behavior
### Pre-commit
- Block commits when core is locked
- Require intent reference for 0-core changes
- Run `faelight health --fail-on warning`

### Commit-msg
- Enforce format: `Intent: 0xx` for major changes
- Warn on missing intent for new features

### Pre-push
- Final verification
- Block risky diffs

## Intent Enforcement
```
# Valid commit messages:
feat: Add faelight-menu power options

Intent: 020

# Invalid (blocked):
fixed stuff
```

## Success Criteria
- [ ] `faelight git install-hooks`
- [ ] Block commits when locked
- [ ] Intent reference enforcement
- [ ] `faelight git verify` command
- [ ] Pre-push validation

---
_The forest guards its history._ 🌲

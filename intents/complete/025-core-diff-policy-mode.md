---
id: 025
date: 2026-01-10
type: future
title: "core-diff - Shell Policy View"
status: complete
tags: [rust, diff, policy, v6.5.0]
---
## The Vision
Show where shell scripts violate Tooling Authority Policy. Review intelligence, not enforcement.

## Why
- Catch shell authority violations before they ship
- Highlight risk, not style
- Respect manual review

## Command
```bash
core-diff --policy shell
core-diff wm-sway --policy shell
core-diff --policy shell --verbose
core-diff --policy shell --json
```

## Output Example
```
🌲 core-diff — Shell Authority Violations

Package: wm-sway (🔴 CRITICAL)
  + scripts/reload-wm.sh
    Status: ❌ New Violation
    Domain: State Mutation
    Reason: Uses swaymsg + systemctl
    Suggested Fix: Graduate to Rust (faelight sway reload)

Package: shell-zsh (🟠 HIGH)
  ~ scripts/theme-preview.sh
    Status: ⚠️ Scope Expanded
    Change: Added filesystem writes
    Expires: 2026-02-01

Package: scripts-global (✅ OK)
  - scripts/old-update.sh
    Status: ✅ Violation Removed
```

## Violation Classifications
| Status | Meaning |
|--------|---------|
| ✅ Allowed | Interface-only |
| ⚠️ Exception | Temporary/expiring |
| ❌ Violation | Authority breach |
| ➕ New | Introduced in diff |
| ~ Changed | Scope expanded |
| ➖ Removed | Violation eliminated |

## Forbidden Patterns
- `sudo` → Security
- `systemctl` → Automation
- `pacman`, `yay` → Package Management
- `git commit/push` → Git Governance
- writes to `~/0-core` → State Mutation

## Relationship to dot-doctor
| Tool | Role |
|------|------|
| core-diff | Change review ("What did I just do?") |
| dot-doctor | State enforcement ("Is system valid?") |

## Success Criteria
- [ ] `--policy shell` flag
- [ ] Group by package + blast radius
- [ ] Forbidden pattern detection
- [ ] Metadata awareness (shell-policy headers)
- [ ] JSON output

---
_Review the forest's changes._ 🌲

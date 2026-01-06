---
id: 015
date: 2026-01-07
type: future
title: "entropy-check - Configuration Drift Detection"
status: planned
tags: [rust, maintenance, entropy, v5.4]
---

## The Vision

Detect configuration entropy. Maintenance wisdom, not cleanup scripts.

## Why

Systems accumulate cruft:

- Aliases never used
- Configs never touched
- Rules never triggered
- Profiles forgotten

entropy-check surfaces this drift without forcing action.

## Features

### Usage Analysis

```bash
entropy-check
```

Output:

```
📊 Entropy Report - 0-Core v5.1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔧 Aliases
   12 aliases unused in 90+ days
   Candidates: ll, la, ..., grep-config

📁 Configs
   3 files untouched in 180+ days
   Consider review: old-theme.conf, ...

🎮 Profiles
   "low-power" unused in 60 days

📜 Scripts
   2 scripts never executed
   Candidates: old-backup.sh, test-wifi.sh

💡 Suggestions
   "Consider pruning unused aliases"
   "Review stale configs for relevance"
```

### Tracking

- Alias execution frequency (shell integration)
- Config file access times
- Profile switch history
- Script execution counts

## Philosophy

**Surfaces entropy. Doesn't clean it.**

You decide what stays. entropy-check just shows truth.

## Success Criteria

- [ ] Tracks alias usage
- [ ] Reports stale configs
- [ ] Shows unused profiles
- [ ] Identifies dead scripts
- [ ] Suggests, never forces

---

_The forest sheds what it doesn't need._ 🌲

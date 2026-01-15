---
id: 056
date: 2026-01-15
type: future
title: "0-Core Archaeology Tool - System History Explorer"
status: complete
tags: [rust, tools, v7.2, understanding]
---

## Vision
Never lose sight of how your system evolved. See the complete history of every package, intent, and decision.

**Philosophy:** "Understanding over convenience" - Know your system's story.

## The Problem
- "When did I change wm-sway?"
- "What commits were part of Intent 036?"
- "What have I worked on this week?"
- Git log is powerful but not package-aware

## The Solution
### archaeology-0-core v0.2

**Package-specific history:**
```bash
arch wm-sway    # Package evolution
```

**System-wide views:**
```bash
archtime        # Full chronological timeline
archwk          # Last 7 days
archint 036     # Intent-specific commits
archsince v7.0.0  # Changes since version
```

## Features
- [x] Package-specific history with file stats
- [x] System-wide timeline (all packages)
- [x] Time-based filtering (--this-week)
- [x] Intent correlation (--by-intent)
- [x] Version-based filtering (--since)
- [x] Color-coded output (dates, commits, stats)
- [x] Automatic package detection from changed files

## Output
```
🌲 0-Core Archaeology - wm-sway
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📅 2026-01-11
   26df4d2  🔧 Sync sway config - use faelight tools
   Files: 1 (+2) (-2)

📅 2026-01-10
   e593fd4  🌲 Release v6.3.0 - Power Menu & Screen Locker
   Intent: 036
   Files: 1 (+1) (-1)
```

## Success Criteria
- [x] CLI tool: `archaeology-0-core`
- [x] Aliases: arch, arch0, archtime, archwk, archint, archsince
- [x] Package history with stats
- [x] System timeline
- [x] Intent correlation
- [x] Version filtering
- [x] Proper error handling

## Impact
- Understand system evolution at a glance
- Track intent → implementation → release
- Answer "what changed?" questions instantly
- Perfect complement to Intent Ledger

---
_"The forest remembers its growth."_ 🌲

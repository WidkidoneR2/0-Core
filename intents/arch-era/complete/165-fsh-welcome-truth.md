---
id: 165
date: 2026-03-28
type: future
title: "fsh Welcome Screen — Truth Only, No Stale Data"
status: complete
tags: [fsh, welcome, accuracy, truth, live-data, shell]
version: 11.5.0
priority: high
depends_on: [157, 163]
---

## The Problem
The fsh welcome screen shows stale and inaccurate data:
```
Current (broken):
101 intents complete  ← stale (actually 113)
1702 commits         ← stale (actually 1718)
95% ✅               ← was stale until health fix
67 tools             ← wrong (scripts/ has 67 files but only 44 are tools)
17 planned           ← reads all future/*.md including non-intents
"v9.3.0"             ← hardcoded old version in some aliases
```

This violates the core principle:
"Document what exists. Not what you wish existed."
A welcome screen that lies about the system
is worse than no welcome screen.

## Root Causes

### 1. Commit count reads /etc/faelight/COMMITS
Only updated on release. Between releases it drifts.
Fix: read from git log directly.

### 2. Tool count reads scripts/ file count
scripts/ has 67 files including shell scripts, not just tools.
Fix: read from path resilience registry or count deployed binaries only.

### 3. Intent count reads directory file count
future/*.md includes non-intent files (decisions, incidents, philosophy).
Fix: count only files where status != complete and type == future intent.

### 4. Health reads cache file
Fixed in current version — reads ~/.cache/faelight/health-status.
But cache only updates when doctor runs.
Fix: already correct, document it.

### 5. Quote and "Today" fields
Quote is random from a hardcoded list — acceptable.
"Today" now reads actual in-progress intents — fixed.
Verify this remains accurate.

## The Solution
Every data point in the welcome screen reads live data.
No caches older than the last doctor run.
No hardcoded values.
No approximations.

## Data Sources (live)
```
commits      → git -C ~/0-core rev-list --count HEAD
tools        → count files in scripts/ where file is executable binary (not .sh)
intents      → count intents/complete/*.md where frontmatter type=future
planned      → count intents/future/*.md where status=planned or in-progress
health       → ~/.cache/faelight/health-status (updated by doctor)
version      → /etc/faelight/VERSION
active       → intents/future/*.md where status=in-progress
```

## The Welcome Screen Contract
Every line must be:
1. **True** — matches actual system state
2. **Current** — data from this session or last doctor run
3. **Meaningful** — tells you something actionable
4. **Concise** — one line, one fact

## Phase 1 — Fix Commit Count
Replace /etc/faelight/COMMITS read with live git count:
```rust
let commits = std::process::Command::new("git")
    .args(["-C", core_root, "rev-list", "--count", "HEAD"])
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .and_then(|s| s.trim().parse::<u64>().ok())
    .unwrap_or(0);
```

## Phase 2 — Fix Tool Count
Count only executable binaries, not shell scripts:
```rust
let tools = std::fs::read_dir(root.join("scripts"))
    .map(|entries| entries.flatten()
        .filter(|e| {
            // Only count files without extension (compiled binaries)
            e.path().extension().is_none() &&
            e.file_type().map(|t| t.is_file()).unwrap_or(false)
        })
        .count())
    .unwrap_or(0);
```

## Phase 3 — Fix Intent Count
Count only genuine intents:
```rust
// Only count .md files where frontmatter has type: future
// Not decisions, incidents, or philosophy entries
```

## Phase 4 — Verify "Today" Accuracy
Confirm in-progress intent detection reads status correctly.
Already fixed — verify stays correct after INT-163 alias cleanup.

## Phase 5 — Add Session Summary Line
One new line that tells you what changed since last session:
```
↑ 5 commits since yesterday  ·  INT-162 in progress
```

## Gate Check
```
✅ Commit count reads git rev-list live — main.rs + session.rs (2026-03-30)
✅ Tool count reads from tools.toml registry — mirrors doctor exactly, shows 50 (2026-03-30)
✅ Intent count scans all categories by status: complete — mirrors doctor exactly, shows 114 (2026-03-30)
✅ Health reads ~/.cache/faelight/health-status correctly — verified (2026-03-30)
✅ Version reads 00-meta/VERSION — correct, version controlled in git (2026-03-30)
✅ Active intents reads in-progress status correctly — verified (2026-03-30)
✅ No hardcoded values anywhere in welcome screen (2026-03-30)
✅ All values verified accurate — matches doctor output exactly (2026-03-30)
✅ Session summary line shows changes since last session — verified in session.rs (2026-03-30)
```

## The Phrase
**"The welcome screen is the forest greeting you.
It should tell the truth.
Every stale number is a small lie.
Small lies compound."**

---
*"Truth in the welcome screen means
truth in everything the shell shows you.
Start honest. Stay honest."* 🌲

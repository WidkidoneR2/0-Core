---
id: 163
date: 2026-03-28
type: future
title: "Alias Audit — One Concept, One Command"
status: complete
tags: [aliases, audit, zsh, fsh, canonicalization, cleanup, integrity]
version: 11.5.0
priority: critical
depends_on: [162]
---

## The Problem
Current state (measured 2026-03-28):
- 446 aliases in aliases.zsh
- 16 duplicate names
- 20 stale aliases (dot-doctor, sway, v9.3.0)
- Multiple synonyms per concept
- Hidden logic inside aliases
- Mixed abstraction levels

This violates the core principle:
"Everything is understood. Nothing is installed blindly."
You do not understand 446 aliases.
Nobody does.

## The Evidence
```
doctor = dot-doctor     (stale — dot-doctor retired)
check-health = dot-doctor (stale + synonym)
health = dot-doctor     (stale + synonym)
d = core doctor run     (correct canonical)

sec = security-audit    (duplicate × 2)
sb = faelight-sandbox   (duplicate × 2)
cls = core link sync    (duplicate — also core ledger stats)
cpl = core checkpoint list (duplicate — also core plugin list)
docs = cd ~/Documents   (collision — also faelight-docs)
```

## The Three Layers

### Layer 1 — Muscle Memory (MAX 100 aliases)
Short, stable, never changes. You type these without thinking.
These are the only aliases that earn their place.
```
# Navigation (10)
..  ...  ....  0core  src  work  keep  tmp  conf  cdp

# Daily tools (15)
c  d  v  g  l  b  y  ya  fm  lg  loc  top  repo  diff  bench

# Git (10)
ga  gaa  gc  gp  gl  gst  gd  glog  gco  gcb

# faelight tools (10)
fg  fs  fm  fsh-deploy  bump  fu  sec  lock-core  unlock-core  cistart

# Core shortcuts (10)
cicomplete  cpc  cpr  ce  cew  cistart  cicomplete  decide  vault  clip

# System (5)
ports  myip  sr  ssn  paci
```

Total Layer 1: ~100 aliases. Every one memorizable.

### Layer 2 — Named Scripts (not aliases)
These are workflows disguised as aliases. They deserve real scripts:
```
# Currently aliases — should be scripts:
safe-up        → ~/0-core/scripts/safe-up (already exists)
pre-commit     → ~/0-core/scripts/pre-commit
full-audit     → core security scan (already in core)
system-health  → d (already canonical)
overview       → faelight-digest (already exists)
release-prep   → faelight-release preview (already exists)
cdcheck        → remove (use d separately)
```

### Layer 3 — Core Commands (not aliases at all)
These should be called directly, not aliased:
```
# Remove these aliases — use core directly:
health        → core doctor run  (use d)
decisions     → core decision list
story         → core story
advise        → core advise
audit         → core audit scan
forecast      → core doctor forecast
```

## The Canonicalization Rules

### Rule 1: One concept, one command
```
# BEFORE (4 ways to check health):
doctor, check-health, health, d

# AFTER (1 canonical):
d = core doctor run
# All others: REMOVED
```

### Rule 2: Aliases only shorten, never multiply
```
# WRONG (multiplying meanings):
alias docs = 'cd ~/Documents'
alias docs = 'faelight-docs'  # COLLISION

# RIGHT:
alias fdocs = 'faelight-docs'  # unique prefix
# docs removed — too ambiguous
```

### Rule 3: Hidden logic → named scripts
```
# WRONG (alias with hidden logic):
alias safe-up='snap-now && safe-update && dot-doctor'

# RIGHT (named script):
~/0-core/scripts/safe-up  # already exists, use it directly
```

### Rule 4: No stale references
```
# REMOVE all references to:
dot-doctor      → retired (use core doctor)
swaymsg         → retired (use niri)
v9.3.0          → stale version string
bump-system-version → retired (use faelight-release)
compile-changelog.sh → retired
```

### Rule 5: No namespace collisions
```
# DANGER — system command overrides:
alias cat = 'bat'      # breaks scripts expecting cat
alias top = 'btm'      # breaks scripts expecting top
alias diff = 'difft'   # breaks scripts expecting diff

# RIGHT:
alias ccat = '/usr/bin/cat'  # already exists
# Keep overrides DOCUMENTED and MINIMAL
# bat, btm, difft: keep as personal preference
# Document them explicitly as overrides
```

## The Audit Process

### Step 1 — Classify every alias (automated)
```bash
core alias-audit scan  # new core command
# Classifies each alias as:
# KEEP    — Layer 1, used regularly
# SCRIPT  — Layer 2, needs real script
# CORE    — Layer 3, should be core command
# STALE   — references retired tools
# DUPE    — duplicate definition
# COLLIDE — namespace collision
```

### Step 2 — Kill duplicates immediately
Lines 66-76 in aliases.zsh: complete duplicate block.
Delete lines 66-76. No discussion needed.

### Step 3 — Remove stale references
All dot-doctor references → core doctor run or d
All swaymsg references → remove (niri migration complete)
All v9.3.0 references → remove
All bump-system-version → remove

### Step 4 — Rename collisions
docs → fdocs (faelight-docs)
docs collision with cd ~/Documents → remove cd alias, use cdocs

### Step 5 — Move hidden logic to scripts
safe-up, pre-commit, full-audit → verify scripts exist, remove aliases

### Step 6 — fsh config.fsh audit
Apply same rules to ~/.config/faelight-shell/config.fsh
Target: max 32 aliases in fsh config (already at 32, review each)

### Step 7 — Document canonical commands
Create docs/CANONICAL-COMMANDS.md:
One table. Every concept. One canonical form.

## Target State
```
Layer 1 aliases (zsh):    ~100  (down from 446)
Layer 1 aliases (fsh):    ~32   (current, review each)
Named scripts:            +5    (extracted from aliases)
Core commands:            0     (all already in core)
Duplicates:               0     (down from 16)
Stale references:         0     (down from 20)
Namespace collisions:     documented and minimal
```

## New Core Command: core alias-audit
Add `core alias-audit` domain to support ongoing alias health:
```bash
core alias-audit scan     # classify all aliases
core alias-audit dupes    # show duplicates
core alias-audit stale    # show stale references
core alias-audit report   # full audit report
```

## The Command Guide (docs/COMMAND-GUIDE.md)
A single factual reference for daily and weekly use.
Not aspirational — only what actually exists and works today.

### Structure
```
# 🌲 Faelight Forest — Command Guide
Last updated: <date> (v11.x.x)

## Daily Commands (use every session)
| Command | What it does | When to use |
|---------|-------------|-------------|
| d       | Run doctor — full health check | Every session start |
| fs      | Open faelight-shell | Primary shell |
| v       | Open neovim | Edit any file |
| g       | Git shortcut | Any git operation |
| ya      | Yazi file manager | Navigate filesystem |
| cistart NNN | Start an intent | Before any intent work |
| cicomplete NNN | Complete an intent | After intent done |
| fg commit | Forest git commit | After any changes |
| lock-core | Lock core files | Before shutdown |
| unlock-core | Unlock core files | Before editing |

## Weekly Commands (use most weeks)
| Command | What it does | When to use |
|---------|-------------|-------------|
| core predict next | What ships next | Planning sessions |
| core react story | What forest signaled | Weekly review |
| core stress report | Verify v11 integrity | After major changes |
| fu | Run faelight-update | Weekly updates |
| core security scan | Security check | Weekly |
| bump | Release new version | After milestone |

## Forest Intelligence (core commands)
| Command | What it does |
|---------|-------------|
| predict sessions | Your work rhythm patterns |
| predict health | Health trajectory forecast |
| predict next | Next intent prediction |
| react run | Evaluate all rules now |
| react story | Today's reaction narrative |
| stress report | Full system verification |
| doctor forecast | Health forecast |

## fsh Builtins (native shell commands)
| Command | What it does |
|---------|-------------|
| pwd | Current directory |
| which <cmd> | Where command resolves |
| type <cmd> | Full resolution detail |
| env | Shell environment table |
| theme <name> | Switch prompt theme |
| echo <text> | Output text |
| cat <file> | View file with line numbers |
| clear / c | Clear terminal |

## Prompt Themes
| Theme | What it shows |
|-------|-------------|
| forest | Full context: path, git, health, commits |
| minimal | Path only — zero noise |
| classic | user@host path $ |
| jarvis | Forest + prediction inline |
```

### Rules for the Guide
1. Only commands that work TODAY — no aspirational entries
2. Updated on every release by faelight-docs
3. One row per command — no duplicates
4. "When to use" column keeps it actionable
5. Replaces scattered documentation across 15 docs files

## Gate Check
```
✅ Duplicate block removed (2026-03-29) — 16 duplicates eliminated
✅ All dot-doctor references removed (2026-03-29) — retired tool
✅ All swaymsg/sway references removed (2026-03-29) — niri migration complete
✅ Stale version strings removed (2026-03-29) — v9.3.0, bump-system-version, compile-changelog
✅ docs collision resolved (2026-03-29) — fdocs=faelight-docs, cdocs=cd ~/Documents
✅ Hidden logic aliases removed (2026-03-29) — safe-up, pre-commit, full-audit, overview
✅ Aliases reduced 446 → 368 (2026-03-29) — 78 removed, zero duplicates
✅ fsh config.fsh reviewed — dead alias removed, all remaining earn their place (2026-03-30)
✅ docs/CANONICAL-COMMANDS.md — folded into COMMAND-GUIDE.md (2026-03-30)
✅ core alias-audit scan passes with 0 critical issues (2026-03-30)
✅ Both zsh AND fsh alias files in sync on concepts (2026-03-30)
✅ docs/COMMAND-GUIDE.md created — 155 lines, all current commands (2026-03-30)
✅ Guide is factual — only what works today (2026-03-30)
✅ Guide updated by faelight-docs on each release — deferred to INT-157 (faelight-docs v2) which owns this responsibility (2026-03-30)
```

## The Phrase
**"100 aliases you know by heart
are worth more than 446
you discover by accident."**

---
*"The forest that cannot name its own tools
cannot trust them.
Audit is not housekeeping.
It is self-knowledge."* 🌲

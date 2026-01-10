# Theory of Operation - 0-Core System

> How the system is SUPPOSED to work, what must ALWAYS be true, and what NEVER happens automatically.

**Version:** 3.3.4  
**Last Updated:** 2025-12-22  
**Status:** Living Document

---

## Purpose

This document explains:

- **How** 0-core works at a fundamental level
- **Why** design decisions were made
- **What** invariants must always hold
- **When** things should fail safely
- **Where** to look when debugging

**Audience:**

- Future-you (6 months from now)
- Contributors
- People learning from this system
- Anyone wanting to understand the design

---

## Core Principles

### 1. Manual Control Over Automation

**Principle:** YOU decide when things happen, the system doesn't decide for you.

**Why:** 12-hour password incident proved automation at boot is unpredictable and dangerous.

**Implementation:**

- No systemd timers at boot
- No cron jobs with sudo
- All maintenance requires explicit trigger
- Confirmation prompts for safety

**Enforcement:** See `docs/POLICIES.md` (Automation Policy)

---

### 2. Immutability of Core

**Principle:** Configuration is protected by default, requires explicit unlock to edit.

**Why:** Prevents accidental deletion, corruption, or unintended modification.

**Implementation:**

- `chattr +i ~/0-core` (filesystem immutable)
- `lock-core` / `unlock-core` commands
- `edit-core` with auto-lock
- Protected by default, editable by choice

**Enforcement:** Physical filesystem protection, not just permissions

---

### 3. Git as Truth

**Principle:** All configuration changes must be git-tracked.

**Why:** Provides reversibility, auditability, and history.

**Implementation:**

- Every change committed
- Clean git state required
- dot-doctor checks for dirty tree
- Snapshots capture known-good states

**Enforcement:** Health checks, manual verification

---

### 4. Health 100% Always

**Principle:** System must maintain 100% health score.

**Why:** Early detection prevents late debugging nightmares.

**Implementation:**

- dot-doctor validates 9 checks
- Stow packages deployed correctly
- No broken symlinks
- Git clean
- Services running

**Enforcement:** Manual checks, pre-commit validation

---

### 5. Documentation = Code

**Principle:** Documentation is as important as the code itself.

**Why:** Future-you has no memory of current-you's decisions.

**Implementation:**

- Every decision documented
- Every incident recorded
- Every policy explained
- Every failure mode catalogued

**Enforcement:** See `docs/POLICIES.md` (Documentation Policy)

---

## System Invariants

> These conditions MUST ALWAYS hold. If violated, the system is in an invalid state.

### Invariant 1: Manual Control

**Rule:** No automation runs without explicit user trigger.

**Verification:**

```bash
# Check for systemd user timers
systemctl --user list-timers
# Should show ZERO timers

# Check for cron jobs
crontab -l
# Should be empty or only manual-trigger jobs
```

**Violation Detection:** Boot-time authentication failures, mysterious system changes

**Recovery:** Disable automation, document incident, update policies

**Origin:** 2025-12-14 password incident

---

### Invariant 2: Immutability Protection

**Rule:** 0-core is filesystem-immutable when locked.

**Verification:**

```bash
# Check lock status
core-status

# Try to delete (should fail)
rm ~/0-core/test-file  # Permission denied

# Check attribute
lsattr ~/0-core | head -1  # Should show 'i' flag
```

**Violation Detection:** Accidental file deletion succeeds

**Recovery:** Re-lock core, restore from git

**Origin:** Protection against catastrophic mistakes

### Lock Reminders

Scripts provide gentle reminders to lock 0-core after modifications:

- `bump-system-version` shows lock as next step
- Shell startup warns if 0-core is unlocked

Philosophy: Gentle reminders, not forced automation. User maintains control.

### Config Aging Report

`dot-doctor` now tracks file modification age:

- Recent (< 30 days)
- Aging (30-90 days)
- Stale (90-365 days)
- Ancient (1+ year)

Provides awareness without enforcement. Helps identify cleanup candidates.

### Intentional Defaults Checker

`dot-doctor` enforces discipline passively:

- Flags non-semantic filenames
- Checks for proper naming conventions
- Verifies .dotmeta presence

Awareness-based enforcement prevents drift over time.

---

### Invariant 3: Git Truth

**Rule:** All changes must be git-tracked, git state must be clean.

**Verification:**

```bash
cd ~/0-core
git status
# Should show: "working tree clean"

dot-doctor
# Should show: "✅ Working tree clean"
```

**Violation Detection:** Uncommitted changes reported

**Recovery:** Review changes, commit or restore

**Origin:** Reversibility and auditability requirements

---

### Invariant 4: Health 100%

**Rule:** System health score must be 100%.

**Verification:**

```bash
dot-doctor
# Must show: "Health: 100%"
```

**Violation Detection:** Health score < 100%

**Recovery:** Fix reported warnings, restore functionality

**Origin:** Early detection philosophy

---

## System Guarantees

> What ALWAYS happens, what NEVER happens.

### What ALWAYS Happens

1. **Stow creates symlinks** from ~/.config → ~/0-core
2. **lock-core makes filesystem immutable** (chattr +i)
3. **Git tracks every config change** (history preserved)
4. **dot-doctor reports health accurately** (9 checks)
5. **Snapshots preserve system state** (Btrfs + Snapper)
6. **Manual triggers require user action** (no surprises)

### What NEVER Happens

1. **Boot-time automation** (no systemd timers at boot)
2. **Automatic package updates** (manual-only)
3. **Silent configuration changes** (git tracks all)
4. **Background sudo operations** (authentication policy)
5. **Untracked file modifications** (git + immutability)

---

## Architecture Overview

### Directory Structure

```
~/
├── 0-core/              # 🔒 Immutable configs (this repo)
│   ├── wm-sway/         # Desktop environment
│   ├── shell-zsh/      # Shell configuration
│   ├── editor-nvim/     # Editor setup
│   ├── scripts/         # Management scripts
│   └── docs/            # Documentation
│
├── 1-src/               # Source code projects
├── 2-work/              # Active work
├── 3-keep/              # Archives
├── 9-temp/              # Disposable
│
├── vault/               # 🔐 KeePassXC (secrets)
└── snapshots/           # Btrfs snapshots
```

**Why numbered?**

- Instant priority recognition
- Muscle memory (g+0, g+1, g+2 in Yazi)
- Clear hierarchy
- Scalable structure

---

## Failure Modes

> What can break, how it breaks, how to recover.

### Critical Failure: Black Screen on Login

**Symptom:** No desktop after login, blank screen

**Cause:** wm-sway corruption (blast radius: critical)

**Detection:** Can't interact with system

**Recovery:**

```bash
# 1. Drop to TTY2
Ctrl+Alt+F2

# 2. Unlock core
unlock-core

# 3. Restore from git
cd ~/0-core
git status
git restore wm-sway/

# 4. Lock and reboot
lock-core
reboot
```

**Prevention:** Always test wm-sway changes in nested session first

---

### High Failure: Broken Shell

**Symptom:** Zsh crashes on startup

**Cause:** shell-zsh corruption (blast radius: high)

**Detection:** Error messages, can't use terminal

**Recovery:**

```bash
# 1. Drop to bash
bash

# 2. Restore config
cd ~/0-core
git restore shell-zsh/

# 3. Reload
zsh
```

**Prevention:** Test shell changes in separate session

---

### Medium Failure: Editor Won't Start

**Symptom:** Neovim errors on launch

**Cause:** editor-nvim corruption (blast radius: medium)

**Detection:** Editor fails to open

**Recovery:**

```bash
# Restore config
cd ~/0-core
git restore editor-nvim/

# Reload
nvim
```

**Prevention:** Backup before major plugin changes

---

## Dependency Graph

```
CRITICAL (🔴):
  wm-sway
    ├── faelight-bar (HIGH)
    ├── notif-mako (MEDIUM)
    └── theme-gtk (LOW)

HIGH (🟠):
  shell-fish
    ├── prompt-starship (MEDIUM)
    └── editor-nvim (MEDIUM)

  faelight-bar
    └── wm-sway (CRITICAL)

MEDIUM (🔵):
  editor-nvim
  fm-yazi

LOW (🟢):
  browser-qutebrowser
  theme packages
```

**Blast Radius Impact:**

- CRITICAL: System unusable
- HIGH: Major functionality lost
- MEDIUM: Important but not essential
- LOW: Optional features

---

## core-diff - Package-Aware Diff Tool

### The Problem

Traditional `git diff` shows line-by-line changes across all files. This creates noise:

- Mixed package changes in one view
- No context about impact/risk
- Hard to answer: "What packages did I touch?"
- File-level granularity obscures package-level decisions

### The Solution

`core-diff` provides package-level awareness with risk-based grouping.

**Key Features:**

- Groups changes by package (not files)
- Color-codes by `blast_radius` from `.dotmeta`
- Integrates with delta (terminal) and Meld (GUI)
- Answers: "What changed?" at the right abstraction level

### Philosophy

> "Meld shows trees. core-diff shows the forest 🌲"

**Design Principles:**

- Package-aware, not file-aware
- Risk-based prioritization
- Manual inspection, not automated action
- Progressive disclosure (summary → verbose → visual)

### Usage Patterns

**Morning Check:**

```bash
core-diff  # Quick overview
```

**Before Commit:**

```bash
core-diff --verbose  # See all files
```

**Release Review:**

```bash
core-diff since v3.3.5  # Historical comparison
```

**Deep Inspection:**

```bash
core-diff wm-sway --open delta  # Visual review
```

### Risk-Based Workflow

1. Run `core-diff`
2. **CRITICAL** packages → careful review with Meld
3. **HIGH** packages → delta inspection
4. **MEDIUM/LOW** → quick review acceptable

This focuses attention where it matters.

### Integration

**With dot-doctor:**

```bash
core-diff && dot-doctor  # Health check after review
```

**With git workflow:**

```bash
core-diff --verbose      # Review changes
git commit               # Commit when satisfied
core-diff                # Should show clean
```

See [TOOLS.md](TOOLS.md) for complete reference.  
See [WORKFLOWS.md](WORKFLOWS.md) for practical usage patterns.

---

## Design Decisions

### Why Numbered Structure (0-9)?

**Decision:** Use numbers for priority hierarchy

**Rationale:**

- Instant visual recognition
- Muscle memory navigation
- Clear priority
- Scales well

**Alternatives Considered:**

- Named categories (configs/, work/, archive/)
- Flat structure (all in home)
- XDG-only structure

**Why Rejected:**

- Named: Priority unclear, harder to remember
- Flat: Chaos, no organization
- XDG-only: Too rigid, doesn't handle workspace

**Result:** Perfect for Yazi teleports (g+0, g+1, etc.)

---

### Why Manual-Only Updates?

**Decision:** No boot-time or automatic updates

**Rationale:** 12-hour password debugging proved automation dangerous

**Alternatives Considered:**

- Smart automation with safeguards
- Scheduled updates with notifications
- Automatic with rollback

**Why Rejected:**

- ANY automation can break mysteriously
- Boot timing is unpredictable
- Debugging automated failures is nightmare
- User loses control

**Result:** Slight inconvenience, massive reliability gain

---

### Why Immutable Core?

**Decision:** Filesystem-level protection (chattr +i)

**Rationale:** Prevents accidental disasters

**Alternatives Considered:**

- Git-only protection
- Permission-based protection
- No protection (trust yourself)

**Why Rejected:**

- Git: Doesn't prevent deletion before commit
- Permissions: Can be overridden with sudo
- No protection: Humans make mistakes

**Result:** Must explicitly choose to edit, prevents accidents

---

### Why Semantic Package Names?

**Decision:** Use `<category>-<app>` naming

**Rationale:** Self-documenting, immediately clear

**Examples:**

- ✅ wm-sway (window manager - sway)
- ✅ shell-fish (shell - zsh)
- ❌ hypr (old, removed)
- ❌ config (too generic)

**Alternatives Considered:**

- Application names only (sway, zsh, nvim)
- Generic categories (desktop, terminal)
- No structure

**Why Rejected:**

- App-only: Category unclear
- Generic: Too vague
- No structure: Chaos

**Result:** Professional, organized, self-documenting

---

## Version History

**Evolution:**

- v1.0-2.8: Generic dotfiles era
- v3.0: Tokyo Night, security hardening
- v3.1: Hybrid architecture, numbered structure
- v3.2: Smart updates, manual control philosophy
- v3.3: Auto-versioning, policies, incidents

**Key Transformations:**

- Dec 14: Password incident → Manual control philosophy
- Dec 16: dotfiles → 0-core rename
- Dec 17: Auto-versioning + documentation overhaul

---

## Transferability

This system can be:

### Cloned

```bash
git clone git@github.com:WidkidoneR2/0-Core.git ~/0-core
cd ~/0-core
./install.sh
```

### Understood

- Read this document
- Review POLICIES.md
- Study INCIDENTS/
- Examine .dotmeta files

### Modified

- Follow policies
- Document changes
- Test thoroughly
- Commit with meaning

### Taught

- Philosophy is clear
- Decisions explained
- Lessons documented
- Structure visible

### Archived

- Complete documentation
- Git history preserved
- Incident knowledge captured
- Future-proof design

---

## Maintenance Philosophy

### When to Update

- When YOU decide (not the system)
- Weekly maintenance (manual trigger)
- After major system changes
- When problems arise

### How to Update

```bash
# Manual trigger, smart recovery
safe-update

# Or with confirmation prompt
weekly-check
```

### What to Check

```bash
# System health
dot-doctor        # Must be 100%

# Package versions
dotctl status     # Show all versions

# Git state
git status        # Must be clean
```

---

## Related Documentation

**Core Documents:**

- `POLICIES.md` - Rules and principles
- `INCIDENTS/` - What broke and why
- `CHANGELOG-v3.x.md` - Version history
- `README.md` - Quick start guide

**Detailed Guides:**

- `COMPLETE_GUIDE.md` - Comprehensive reference
- `KEYBINDINGS.md` - All shortcuts
- `MELD_GUIDE.md` - Comparison workflow

---

## Future Sections (To Be Completed)

**v3.4.0 (State Verification):**

- [ ] MANIFEST system
- [ ] dotctl verify implementation
- [ ] State snapshots
- [ ] Deletion confidence

**v4.0.0 (Advanced Features):**

- [ ] Secret management integration
- [ ] Cloud backup procedures
- [ ] Multi-machine sync
- [ ] Environment profiles

---

## Questions This Document Answers

1. **How does 0-core work?** → See Architecture Overview
2. **Why these design decisions?** → See Design Decisions
3. **What must always be true?** → See System Invariants
4. **What can break?** → See Failure Modes
5. **How to recover?** → See each failure mode's Recovery section
6. **Why manual control?** → See Manual Control principle
7. **What's the philosophy?** → See Core Principles
8. **How to maintain it?** → See Maintenance Philosophy

---

**This document grows with the system. Every major change, every lesson learned, every policy added gets reflected here.**

**Status:** Foundation complete, detailed sections expand as system matures.

---

**Last Updated:** 2025-12-22 (v3.3.4)  
**Next Review:** 2026-01-01 (after v3.4.0)

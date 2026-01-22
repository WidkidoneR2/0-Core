# Theory of Operation - 0-Core System

> How the system is SUPPOSED to work, what must ALWAYS be true, and what NEVER happens automatically.

**Version:** 8.0.0  
**Last Updated:** 2026-01-22  
**Status:** Production - Philosophy Proven

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
- Contributors (especially kernel developers)
- People learning from this system
- Anyone wanting to understand philosophy-driven architecture

---

## The Philosophy

### Core Thesis

**"Manual control over automation. Intent over convention. Understanding over convenience."**

0-Core is not just dotfiles. It's a position on how personal computing should work:
- **You decide** when things run
- **You understand** what's happening
- **You document** why decisions were made
- **You recover** when things fail

This isn't ideology. It's **proven through 30 production-ready tools** that embody these principles.

---

## Core Principles

### 1. Manual Control Over Automation

**Principle:** YOU decide when things happen, the system doesn't decide for you.

**Why:** The 12-hour password incident (Intent 001) proved automation at boot is unpredictable and dangerous. Systems that "just work" fail silently and catastrophically.

**Implementation:**
- No systemd timers at boot
- No cron jobs with sudo
- All maintenance requires explicit trigger
- Confirmation prompts for destructive operations
- `bump-system-version` asks before every major action

**Evidence:** `bump-system-version v4.0.0` - 8 phases, interactive prompts at each decision point. Automates tedious work while human maintains control.

**Enforcement:** See `docs/POLICIES.md` (Automation Policy)

---

### 2. Immutability of Core

**Principle:** The most critical directory (`~/0-core`) can be physically protected from accidental changes.

**Why:** One `rm -rf` in the wrong place destroys months of work. Permissions aren't enough - we need kernel-level protection.

**Implementation:**
- `chattr +i` on critical files when locked
- `core-protect v1.0.1` - Immutable protection system
- Starship prompt shows lock status (🔒/🔓)
- faelight-bar shows `LCK`/`UNL`
- Git commits blocked when locked

**Evidence:** Survived accidental `rm` attempts, package manager conflicts, and script errors.

**Tools:** `lock-core`, `unlock-core`, `core-protect`

---

### 3. Intent Over Convention

**Principle:** Document WHY decisions were made, not just WHAT was done.

**Why:** Configuration files show *what* but never *why*. Six months later, you have no idea why you chose option X over Y.

**Implementation:**
- Intent Ledger (78+ documented decisions)
- YAML frontmatter with metadata
- Status tracking (planned/active/complete/abandoned)
- Category organization (decisions/experiments/philosophy/future)
- `intent v2.0.0` - Intent management system

**Evidence:** Intent 001 (Rust migration), Intent 059 (Monorepo), Intent 066 (faelight-bar v2.0) - each documents the *journey* not just the destination.

**Tools:** `intent`, `intent-guard`

---

### 4. Understanding Over Convenience

**Principle:** Know your system deeply, even if it's more work.

**Why:** Convenience tools hide complexity. When they break, you're helpless. Understanding gives you power.

**Implementation:**
- Build everything from source (30 Rust tools)
- Comprehensive READMEs for every tool
- Health monitoring with explanations
- `teach v1.0.0` - Interactive learning system
- No "magic" - every automation is auditable

**Evidence:** Complete tool audit achieved 100% documentation. Every tool has `--help`, `--health`, comprehensive README.

**Tools:** `teach`, `dot-doctor`, `archaeology-0-core`

---

### 5. Recovery Over Perfection

**Principle:** Plan for failure. Systems break - have a recovery path.

**Why:** Perfection is impossible. Resilience is achievable.

**Implementation:**
- Btrfs snapshots before every major operation
- `safe-update v1.0.0` - Updates with automatic snapshots
- `bump-system-version` creates snapshot before release
- `faelight-snapshot v1.0.0` - Snapshot management
- Clear rollback instructions

**Evidence:** Survived Omarchy→Arch migration, kernel updates, breaking changes.

**Tools:** `safe-update`, `faelight-snapshot`, `snapper`

---

## System Architecture

### The Stack

**Foundation:**
- Arch Linux (vanilla, not Omarchy)
- Sway 1.11 (Wayland compositor)
- Btrfs filesystem (snapshots)

**Desktop Environment:**
- 7 custom Rust tools (bar, launcher, menu, notify, lock, dashboard, CLI)
- Faelight Forest theme (cohesive visual identity)
- 98 Sway keybindings (zero conflicts)

**Infrastructure:**
- 10 core tools (health, protection, updates, diff, stow, snapshots)
- 8 workflow tools (intent, archaeology, workspace, git, profiles, teach)
- 5 version tools (bump-system-version, bootstrap, get-version)

**Total:** 30 production-ready Rust tools, 15,000+ lines of code

---

### The Monorepo

**Structure:**
```
~/0-core/
├── rust-tools/     # 30 tools in workspace
├── stow/           # 12 dotfile packages
├── scripts/        # Compiled binaries
├── INTENT/         # 78+ decisions
└── docs/           # Theory, policies, guides
```

**Why Monorepo:**
- Unified build (70% faster)
- Shared dependencies
- Atomic version bumps
- Easier auditing

**Evidence:** Intent 059 - Monorepo unification reduced disk from 2GB to 300MB, build times from 8min to 2.5min.

---

### The Stow System

**Principle:** Dotfiles as packages, managed by GNU Stow.

**Why:**
- Atomic deployments
- Easy rollback
- Clear ownership
- No symlink hell

**Structure:**
```
stow/
├── wm-sway/        # Window manager
├── shell-zsh/      # Shell config
├── editor-nvim/    # Editor
└── [9 more packages]
```

**Deployment:** `cd ~/0-core/stow && stow -t ~ package-name`

**Health Check:** `doctor` verifies all 12/12 packages properly stowed

---

## Key Invariants

### ALWAYS True

1. **`~/0-core` is the source of truth** - Never edit `~/.config` directly
2. **Git is always clean before releases** - `bump-system-version` enforces this
3. **System health is measurable** - `doctor` provides 13-check verification
4. **Every tool has `--help`, `--version`, `--health`** - CLI standards enforced
5. **Intents document major decisions** - No undocumented architecture changes
6. **Snapshots exist before destructive operations** - Safety net always present

### NEVER True

1. **Automation runs at boot** - Everything is manual trigger
2. **Passwords stored in configs** - Secrets in `~/secrets/` (never committed)
3. **Tools make decisions for you** - Interactive prompts for major actions
4. **Breaking changes without documentation** - CHANGELOG tracks everything
5. **Production tools without READMEs** - 100% documentation coverage

---

## The Proof

### v8.0.0 Achievement

After 8 weeks of intensive work:
- ✅ 30/30 tools at v1.0.0+ (100% production-ready)
- ✅ 100% system health (verified by 13-check monitoring)
- ✅ Complete audit (every tool documented, tested, standardized)
- ✅ Philosophy validated (tools embody principles)

**Key Milestones:**
- **v1.0.0:** First Omarchy system (chaos)
- **v5.0.0:** Rust migration begins (Intent 001)
- **v7.0.0:** Sway migration (Omarchy→Arch)
- **v8.0.0:** Complete audit, philosophy proven

### Flagship Tools

These tools demonstrate the philosophy in action:

**bump-system-version v4.0.0:**
- 8-phase interactive release workflow
- Automates CHANGELOG generation from git commits
- Detects completed intents automatically
- Creates snapshots before changes
- **But asks human before every major decision**

**faelight-bootstrap v1.0.0:**
- One-command system installation
- 7 interactive phases
- Automatic dependency installation
- **But human confirms each phase**

**teach v1.0.0:**
- Interactive learning system
- Quiz mode with achievements
- Progress tracking
- **Philosophy: Understanding over convenience**

---

## Design Patterns

### 1. Interactive Automation

**Pattern:** Automate tedious work, not decisions.

**Example:** `bump-system-version`
- Automates: Version updates, CHANGELOG generation, git operations
- Asks: Milestone description, release quote, confirm commit/tag/push

**Why:** Humans are good at decisions, bad at repetition. Computers are the opposite.

### 2. Fail-Safe Defaults

**Pattern:** When uncertain, choose the safer option.

**Example:** `safe-update`
- Creates snapshot BEFORE updating
- Shows package diff with risk analysis
- Asks for confirmation
- Provides rollback instructions

**Why:** Recovery is easier than perfection.

### 3. Observable State

**Pattern:** System state should be visible everywhere.

**Example:** Lock status shown in:
- Starship prompt (🔒/🔓)
- faelight-bar (`LCK`/`UNL`)
- Git hooks (commits blocked when locked)

**Why:** You can't control what you can't see.

### 4. Layered Verification

**Pattern:** Multiple checks at different levels.

**Example:** Release process
- Pre-commit: Gitleaks scans for secrets
- Pre-release: `doctor` verifies 100% health
- Post-release: Verification phase confirms success

**Why:** Single points of failure are unacceptable.

---

## When Things Break

### Debug Process

1. **Check system health:** `doctor`
2. **Review recent changes:** `git log --oneline -10`
3. **Check intent history:** `intent list`
4. **Explore system history:** `archaeology-0-core`
5. **Verify snapshots exist:** `faelight-snapshot list`

### Recovery Paths

**If health fails:**
```bash
doctor              # See what's broken
git status          # Check for uncommitted changes
git log -5          # What changed recently?
```

**If tools break:**
```bash
cd ~/0-core
cargo build --release    # Rebuild workspace
doctor                   # Verify health
```

**If system breaks:**
```bash
sudo snapper list                    # Find snapshot
sudo snapper rollback <snapshot-id>  # Restore
reboot
```

---

## Evolution

### From Chaos to Order

**v1.0.0 (Omarchy):**
- Inherited configs, didn't understand them
- Shell scripts everywhere
- No documentation
- Frequent breakage

**v8.0.0 (0-Core):**
- Every tool documented
- 78 intents explain decisions
- 100% system health
- Philosophy proven

### The Journey

1. **Incident 001:** 12-hour password disaster → Manual control principle
2. **Intent 001:** Rust migration → Better tooling
3. **v7.0.0:** Omarchy→Arch migration → Recovery validation
4. **v8.0.0:** Complete audit → Philosophy proof

### What We Learned

- **Automation serves the human** - Not the other way around
- **Documentation is love** - For future-you
- **Philosophy must ship** - Ideas without implementation are worthless
- **Recovery > Perfection** - Systems fail, resilience matters

---

## Future Evolution

See Intent 067 (Post-Linus Evolution Plan) for:
- Multi-machine deployment
- Secrets management improvements
- Advanced intent analytics
- Community sharing

---

## For Linus

**Why this matters:**

This isn't just dotfiles. It's a demonstration that:
- **Rust works for systems tools** - 30 production tools prove it
- **Philosophy can be concrete** - Not just ideas, but shipping code
- **Manual control scales** - Even with 30 tools, human stays in control
- **Documentation matters** - Every tool has comprehensive README
- **Intent-driven development works** - 78 decisions tracked, 73% success rate

**The proof:** From Omarchy chaos to 0-Core order in 8 weeks. 100% system health. Ready for production.

---

## Conclusion

**The Thesis:**

Personal computing should be:
- **Controllable** - You decide
- **Understandable** - You know why
- **Recoverable** - You can fix it
- **Intentional** - You document decisions

**The Evidence:**

30 production-ready tools that embody these principles.

**The Result:**

A system that evolves with intention, not accident.

---

> *"The audit is complete. Every tool documented, tested, and production-ready."* 🌲🦀

**Version:** 8.0.0  
**Status:** Production - Philosophy Proven  
**Last Updated:** 2026-01-22  
**Author:** Christian (Steward of the Forest)

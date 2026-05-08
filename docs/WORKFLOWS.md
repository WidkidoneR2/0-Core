# 0-Core Workflows

Practical usage patterns for real-world scenarios with 13.x tools.

**Version:** 13.1.0
**Philosophy:** Manual control, intentional decisions, observable changes

---

## Daily Workflows

### Morning System Check
**Goal:** Know your system's state before starting work
```bash
# 1. Check system health (23 checks)
d

# 2. What should I work on?
core intent next
# 3. Check git status
fg status

# 3. Check for updates
faelight-update --dry-run

# 4. Review current profile
profile list
```

**Why:** Sets intention, avoids surprises, confirms system integrity

**Expected output:**
- ✅ 100% health score
- ✅ Clean git state
- ℹ️ Available updates (if any)
- ℹ️ Current profile (usually "default")

---

### Configuration Change Workflow
**Goal:** Make every config change intentional and traceable
```bash
# 1. Unlock core for editing
unlock-core

# 2. Edit config files
nvim ~/.config/niri/config.kdl

# 3. Test changes
niri validate --config ~/.config/niri/config.kdl  # Validate
niri msg action load-config-file              # Apply

# 4. Verify health
d

# 5. Review changes
cd ~/0-core
git status
git diff

# 6. Commit with intent
git add stow/niri/
git commit -m "fix(sway): Update keybinding for launcher"

# 7. Push (hooks validate automatically)
git push

# 8. Lock core again
lock-core
```

**Why:** Deliberate, documented, reversible changes

**Git hooks automatically check:**
- 🔍 Secret scanning
- 🔍 Merge conflicts
- 💬 Commit message format
- 🎣 Uncommitted changes (pre-push)

---

### Quick Edit with Auto-Lock
**Goal:** Fast edits with automatic protection
```bash
# Use fg (faelight-git) for streamlined workflow
unlock-core
nvim ~/0-core/03-interfaces/stow/shell-zsh/.zshrc
fg sync                    # Pulls, commits, pushes automatically
lock-core
```

---

## Weekly Workflows

### System Update Routine
**Goal:** Keep system current with controlled updates
```bash
# 1. Check what needs updating
faelight-update --dry-run -v

# 2. Review impact analysis
# Look for:
# - Kernel updates (require reboot)
# - Critical packages (systemd, glibc, etc.)
# - Major version bumps

# 3. Create snapshot (if using BTRFS)
faelight-snapshot create --tag pre-update

# 4. Apply updates
faelight-update

# 5. Verify system health
d

# 6. Check for .pacnew files
find /etc -name "*.pacnew"

# 7. Reboot if kernel updated
# Check if reboot needed in update output
```

**Best day:** Sunday morning (low-pressure, time to fix issues)

---

### Intent Ledger Review
**Goal:** Document decisions and review past choices
```bash
# 1. Review recent intents
intent list

# 2. Check planned work
intent list future

# 3. Document new decision
intent add decision "Switching to X because Y"

# 4. Review specific intent
intent show 067

# 5. Search for topic
intent search "update"
```

**Why:** Builds institutional knowledge, prevents repeated mistakes

---

### Health & Maintenance Check
**Goal:** Prevent technical debt accumulation
```bash
# 1. Run comprehensive health check
doctor --explain

# 2. Run automated test suite
~/0-core/scripts/test-all-tools

# 3. Check git repository
fg status

# 4. Review profile settings
profile list

# 5. Check for package drift
entropy-check

# 6. Update documentation if needed
cd ~/0-core/docs
# Edit outdated docs
```

---

## Profile Workflows

### Switching Contexts
**Goal:** Optimize system for different use cases
```bash
# Morning: Start work mode
profile switch work
# Enables VPN, focused notifications

# Lunch break: Gaming
profile switch gaming
# Max GPU, minimal notifications, VPN off

# Evening: Battery saving
profile switch low-power
# CPU powersave, reduced bar refresh

# Default: Balanced
profile switch default
# VPN on, all features enabled
```

**Current profile shown in:** faelight-bar (DEF/WRK/GAM/LOW)

---

## Incident Response

### When Something Breaks
**Goal:** Quick diagnosis and recovery
```bash
# 1. Check system health
doctor --explain
# Identifies which check is failing

# 2. Check recent git changes
cd ~/0-core
git log --oneline -10

# 3. Review last commit
git show HEAD

# 4. If config issue, rollback
git revert HEAD
git push

# 5. If package issue, restore snapshot
faelight-snapshot list
faelight-snapshot restore <name>

# 6. Document incident
cd ~/0-core/INTENT/incidents
intent add incident "Description of what broke and fix"

# 7. Update POLICIES.md if needed
nvim ~/0-core/docs/POLICIES.md
```

---

### Lock Status Confusion
**Goal:** Verify and fix lock state
```bash
# Check lock status
core-protect status

# Shows in:
# - Starship prompt (🔒/🔓)
# - faelight-bar (LCK/UNL)

# If locked but need to edit:
unlock-core

# If unlocked but should be locked:
lock-core
```

---


**Goal:** Know exactly what to work on and where the forest stands
```bash
d
core intent next
core intent blocked
core intent graph
cistart NNN
```
**Why:** The ledger knows your dependencies, velocity, and what unblocks the most. Trust it.
---
**Goal:** Every piece of work is tracked, gated, and closed properly
```bash
cistart 247
fg done "description of what shipped"
cicomplete 247
```
**Expected on cicomplete:**
- Auto-checkpoint created
- Retrospective shown (commits, what was built)
- Newly unblocked intents surfaced
- core intent next suggestion
---
**Goal:** One-glance understanding of where everything stands
```bash
core intent brief
core intent blocked
core intent graph
core intent next
`---
## Intent Workflows

### Starting a Session
**Goal:** Know exactly what to work on and where the forest stands

    d                    # health check
    core intent next     # what to work on
    core intent blocked  # what is blocked
    cistart NNN          # start recommended intent


### The Intent Lifecycle
**Goal:** Every piece of work tracked, gated, and closed properly



### Checking Forest State

    core intent brief    # active intent, health, what is ready
    core intent blocked  # what cannot start and why
    core intent graph    # full dependency picture
    core intent next     # recommendation with explanation



## Development Workflows

### Adding New Rust Tool
**Goal:** Integrate new tool into workspace
```bash
# 1. Create new package
cd ~/0-core/rust-tools
cargo new --bin my-new-tool

# 2. Add to workspace
# Edit ~/0-core/Cargo.toml:
# members = [..., "rust-tools/my-new-tool"]

# 3. Develop tool
cd rust-tools/my-new-tool
nvim src/main.rs

# 4. Build and test
cd ~/0-core
cargo build --release -p my-new-tool

# 5. Copy to scripts/
cp target/release/my-new-tool scripts/

# 6. Test
scripts/my-new-tool --help

# 7. Update test suite
nvim scripts/test-all-tools
# Add test for new tool

# 8. Update documentation
nvim docs/TOOL_REFERENCE.md

# 9. Commit
git add rust-tools/my-new-tool Cargo.toml scripts/test-all-tools docs/
git commit -m "feat: Add my-new-tool v0.1.0"
git push
```

---

### Workspace Build & Deploy
**Goal:** Rebuild all tools efficiently
```bash
# Full rebuild (clean build)
cd ~/0-core
cargo clean
cargo build --release

# Copy all binaries to scripts/
cp target/release/faelight-* scripts/
cp target/release/dot-doctor scripts/doctor
cp target/release/intent scripts/
# ... etc

# Or use your deployment script
./deploy-binaries.sh   # If you have one

# Verify
~/0-core/scripts/test-all-tools
```

---

### Version Bump Workflow
**Goal:** Release new system version
```bash
# 1. Review changes since last version
git log $(git describe --tags --abbrev=0)..HEAD --oneline

# 2. Verify 100% health
d

# 3. Run full test suite
~/0-core/scripts/test-all-tools

# 4. Bump version (with pre-flight checks)
bump-system-version 8.5.0

# 5. Update CHANGELOG.md
nvim CHANGELOG.md

# 6. Commit and push
git add CHANGELOG.md
git commit -m "docs: Update changelog for v8.5.0"
git push
git push --tags
```

---

## Integration Patterns

### Morning Routine (All-in-One)
```bash
# Single command health + git check
doctor && fg status && faelight-update --dry-run
```

### Pre-Commit Validation
```bash
# Before committing
doctor                     # Health check
git status                 # Review changes
git diff                   # Review diff
git add .
git commit                 # Hooks run automatically
```

### Post-Update Verification
```bash
# After system updates
faelight-update
doctor                     # Verify health
~/0-core/scripts/test-all-tools  # Test tools
reboot                     # If kernel updated
```

---

## Quick Reference Aliases

Add to `~/.zshrc`:
```bash
# Health & Status
alias d="doctor"
alias dx="doctor --explain"

# Updates
alias up="faelight-update --dry-run"
alias upv="faelight-update -v"

# Git
alias gs="fg status"
alias gsync="fg sync"

# Core
alias lock="lock-core"
alias unlock="unlock-core"

# Profiles
alias prof="profile list"
alias profwork="profile switch work"
alias profdef="profile switch default"

# Intents
alias intents="intent list"
alias intentshow="intent show"
```

---

## Advanced Patterns

### Create Pre-Update Baseline
```bash
# Before major changes
doctor > ~/backups/health-$(date +%Y%m%d).txt
git log --oneline -20 > ~/backups/commits-$(date +%Y%m%d).txt
```

### Monitor Health Continuously
```bash
# Watch health status (updates every 5 seconds)
watch -n 5 'doctor'

# Or use doctor's history feature
doctor --history
```

### Automated Daily Check (Manual Trigger)
```bash
# Add to ~/.zshrc
daily-check() {
    echo "=== Daily System Check ==="
    echo ""
    echo "📊 Health Status:"
    doctor
    echo ""
    echo "📦 Git Status:"
    fg status
    echo ""
    echo "🔄 Available Updates:"
    faelight-update --dry-run
}

# Run manually each morning
daily-check
```

---

## Common Scenarios

### "What changed recently?"
```bash
git log --oneline -10
git diff HEAD~5..HEAD
```

### "Did updates break anything?"
```bash
doctor                     # Check health
~/0-core/scripts/test-all-tools  # Test tools
git log -1                 # Last change
```

### "Switch to gaming mode"
```bash
profile switch gaming
# Verify in faelight-bar: GAM
```

### "Quick config edit"
```bash
unlock-core
nvim ~/.config/niri/config.kdl
swaymsg reload
lock-core
```

### "Document decision"
```bash
intent add decision "Reason for change X"
```

---

## Philosophy in Practice

These workflows embody 0-Core principles:

- **Manual Control** - You choose when to run checks, updates, changes
- **Intent Over Convention** - Every decision is deliberate and documented
- **Understanding Over Convenience** - Tools explain, don't hide
- **Recovery Over Perfection** - Focus on catching and fixing mistakes

See `docs/THEORY_OF_OPERATION.md` for deeper context.

---

*Manual control over automation. Understanding over convenience.* 🌲

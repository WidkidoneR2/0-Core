# 🛡️ 0-Core Policies

Self-enforcing rules learned from pain. These policies prevent future-you from repeating past-you's mistakes.

**Last Updated:** 2026-05-08  
**Version:** 13.1.0  
**System:** Arch Linux + Niri + Faelight Forest

---

## Core Philosophy

**Manual Control > Automation**

Every policy stems from one principle: **YOU control the system, it doesn't control you.**

---

## 🔐 Authentication Policy

### Rules
- ❌ **NEVER** use sudo in automated scripts
- ❌ **NEVER** prompt for passwords at boot
- ❌ **NEVER** run privileged commands without explicit user action
- ✅ **ALWAYS** require manual confirmation for sudo operations
- ✅ **ALWAYS** use manual-trigger scripts only

### Rationale

**Incident:** 12-hour password debugging session (2025-12-14)

**What happened:**
1. systemd user timer ran at boot
2. Attempted sudo without credentials
3. Triggered faillock after 3 attempts
4. Locked user account
5. Broke sudo authentication system-wide

**Lesson learned:**  
*"Automation at boot + sudo = debugging nightmare"*

**Prevention:**
- All automation requires explicit user trigger
- No systemd timers at boot
- No cron jobs with sudo
- Manual confirmation prompts for everything

**Documentation:** See `docs/INCIDENTS/2025-12-14-password-sudo-failure.md`

---

## ⏰ Automation Policy

### Rules
- ❌ **NEVER** schedule scripts at boot
- ❌ **NEVER** use systemd timers for maintenance
- ❌ **NEVER** auto-run updates or backups
- ✅ **ALWAYS** require explicit user trigger
- ✅ **ALWAYS** use confirmation prompts
- ✅ **ALWAYS** make automation opt-in, never automatic

### Examples

**Bad (Automated):**
```ini
# systemd timer at boot
[Timer]
OnBootSec=5min
```

**Good (Manual Trigger):**
```bash
# Zsh function with confirmation
weekly-check() {
    read -q "?Continue? (y/N): "
    if [[ $? -eq 0 ]]; then
        faelight-update
    fi
}
```

### Rationale

**Lesson learned:**  
*"Anything that runs automatically WILL break mysteriously at the worst time."*

**Prevention:**
- User decides when things run
- Predictable behavior
- Easy to debug
- No surprises

---

## 🔒 Immutability Policy

### Rules
- ✅ **ALWAYS** keep 0-core locked by default
- ✅ **ALWAYS** require explicit unlock for edits
- ✅ **ALWAYS** re-lock after changes
- ❌ **NEVER** leave 0-core unlocked overnight
- ❌ **NEVER** edit configs outside 0-core structure

### Implementation
```bash
# Lock core (filesystem immutable)
lock-core    # chattr +i ~/0-core

# Unlock to edit
unlock-core  # chattr -i ~/0-core

# Check status
core-protect status
```

### Rationale

**Problem:** Accidental `rm -rf` or file corruption

**Solution:** Filesystem-level protection prevents:
- Accidental deletion
- Unintended modifications
- File corruption
- Directory removal

**Result:** Must explicitly choose to edit

---

## 📝 Documentation Policy

### Rules
- ✅ **ALWAYS** document every decision in Intent Ledger
- ✅ **ALWAYS** explain failure modes
- ✅ **ALWAYS** log breaking changes in CHANGELOG
- ✅ **ALWAYS** write for future-you
- ❌ **NEVER** assume you'll remember why

### Requirements

Every major change needs:
1. **Why** - Rationale for change
2. **What** - What changed
3. **How** - Implementation details
4. **Risk** - Potential failure modes
5. **Rollback** - How to undo if needed

### Examples

- `INTENT/` - Decision history and rationale
- `CHANGELOG.md` - Version changes
- `docs/ARCHITECTURE.md` - System structure
- `docs/INCIDENTS/` - What broke and why
- `docs/POLICIES.md` - This document

### Rationale

*"Future-you has no memory of current-you's decisions."*

**Prevention:**
- Document WHY, not just WHAT
- Record lessons learned
- Make knowledge transferable
- Help others learn

---

## 🔄 Update Policy

### Rules
- ❌ **NEVER** auto-update at boot
- ❌ **NEVER** update without health check
- ❌ **NEVER** skip pre-update verification
- ✅ **ALWAYS** use `faelight-update`
- ✅ **ALWAYS** create snapshots (if available)
- ✅ **ALWAYS** verify system health after updates
- ✅ **ALWAYS** check for .pacnew files

### Process
```bash
# Manual update (YOU decide when)
faelight-update

# Or dry-run first
faelight-update --dry-run

# With verbose output
faelight-update -v
```

### Safety Features
- Pre-update health check (`doctor`)
- Smart package detection (pacman, AUR, cargo)
- Impact analysis (kernel, critical packages)
- Post-update workspace rebuild
- .pacnew detection (coming soon)

### Rationale

**Lesson learned:**  
*"Updates break things. Know what changed."*

---

## 📊 State Verification Policy

### Rules
- ✅ **ALWAYS** maintain 100% health score
- ✅ **ALWAYS** fix warnings immediately
- ✅ **ALWAYS** commit changes to git
- ❌ **NEVER** leave dirty git state
- ❌ **NEVER** ignore health warnings

### Verification
```bash
# Check system health (14 checks)
doctor

# Check git state
fg status

# Run full test suite
~/0-core/scripts/test-all-tools
```

### Rationale

**Early detection > Late debugging**

**Prevention:**
- Catch problems early
- Maintain known-good state
- Easy rollback if needed

---

## 🎯 Blast Radius Policy

### Rules
- ⚠️ **ALWAYS** consider blast radius before editing
- ⚠️ **ALWAYS** backup critical components first
- ✅ **ALWAYS** use recovery procedures for high-risk edits

### Classification

- 🔴 **Critical:** System unusable if broken (`wm-sway`)
- 🟠 **High:** Major functionality lost (`shell-zsh`, `faelight-bar`)
- 🔵 **Medium:** Important but not essential (`editor-nvim`)
- 🟢 **Low:** Optional features (`browser-qutebrowser`)

### Before Editing
```bash
# Check current status
dotctl status

# Unlock carefully
unlock-core

# Edit with awareness
nvim ~/0-core/03-interfaces/stow/wm-sway/.config/sway/config

# Test changes
sway -C

# Lock again
lock-core
```

### Rationale

*"Know what you're risking before you break it."*

---

## 🔐 Security Policy

### Rules
- ✅ **ALWAYS** store secrets in KeePassXC
- ✅ **ALWAYS** use encrypted backups
- ✅ **ALWAYS** scan for secrets before commits (gitleaks)
- ❌ **NEVER** commit secrets to git
- ❌ **NEVER** store plaintext passwords
- ❌ **NEVER** expose API keys in configs

### Secrets Management

- **Storage:** `~/vault/passwords.kdbx` (KeePassXC)
- **Backup:** Encrypted cloud storage
- **Git:** Pre-commit hooks block secrets (`faelight-hooks`)
- **Configs:** Reference secrets, don't embed

### Current Security (v8.4.0)

- ✅ UFW firewall active
- ✅ fail2ban protecting SSH
- ✅ Mullvad VPN connected
- ✅ LUKS2 full disk encryption (recommended)
- ✅ DNSOverTLS (Quad9)
- ✅ Git hooks scan for secrets
```bash
# Verify security status
doctor --explain  # Shows security check
```

---

## 📦 Package Management Policy

### Rules
- ✅ **ALWAYS** use semantic package names
- ✅ **ALWAYS** document package purpose
- ✅ **ALWAYS** track dependencies
- ✅ **ALWAYS** use GNU Stow for deployment
- ❌ **NEVER** create generic package names

### Naming Convention

`<category>-<application>`

**Examples:**
- ✅ `wm-sway` (window manager - sway)
- ✅ `shell-zsh` (shell - zsh)
- ✅ `editor-nvim` (editor - neovim)
- ✅ `term-foot` (terminal - foot)
- ❌ `config` (too generic)
- ❌ `hypr` (old, removed)

### Rationale

*"Self-documenting structure > cryptic names"*

---

## 🧪 Testing Policy

### Rules
- ✅ **ALWAYS** test changes before committing
- ✅ **ALWAYS** verify health after changes
- ✅ **ALWAYS** check for broken symlinks
- ✅ **ALWAYS** run test suite for Rust tools
- ❌ **NEVER** commit untested changes

### Testing Checklist
```bash
# 1. Test functionality
# Make your changes, test they work

# 2. Check health
doctor

# 3. Run automated tests (if Rust tools changed)
cargo build --release
~/0-core/scripts/test-all-tools

# 4. Verify git
git status

# 5. Commit with hooks
git add -A
git commit -m "type(scope): description"
git push
```

### Git Hooks (faelight-hooks v1.0.0)

Automatic checks on every commit:
- 🔍 Secret scanning (gitleaks)
- 🔍 Merge conflict detection
- 💬 Conventional commit validation
- 🎣 Pre-push uncommitted change detection

---

## 📋 Violation Response

### If Policy Violated

1. **Stop immediately**
2. **Assess damage**
3. **Rollback if needed** (`git restore` or BTRFS snapshot)
4. **Document incident** (add to `INTENT/incidents/`)
5. **Update policy** (prevent recurrence)
6. **Learn lesson** (update documentation)

### Example Process
```bash
# Something broke
cd ~/0-core
git status  # What changed?
git restore <file>  # Rollback
lock-core  # Protect again

# Document
intent add incident "Description of what broke"

# Update policies if needed
nvim docs/POLICIES.md
```

---

## 🎓 Policy Evolution

These policies are **living documents**.

**When you:**
- Make a mistake → Add a policy
- Find a better way → Update a policy
- Learn a lesson → Document it in Intent Ledger

**Goal:** Make it impossible to repeat past mistakes.

---

## 📚 Related Documentation

- `docs/ARCHITECTURE.md` - System structure
- `docs/BUILD.md` - Build workflow
- `docs/THEORY_OF_OPERATION.md` - How system works
- `INTENT/` - Decision history
- `CHANGELOG.md` - Version changes
- `docs/INCIDENTS/` - What broke and why

---

**Remember:** These policies exist because we learned the hard way.  
Don't repeat history. Follow the policies. 🛡️

*Manual control over automation. Understanding over convenience.* 🌲

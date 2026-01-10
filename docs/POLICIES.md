# 🛡️ 0-Core Policies

> Self-enforcing rules learned from pain. These policies prevent future-you from repeating past-you's mistakes.

**Last Updated:** 2025-12-17  
**Version:** 3.3.0

---

## Core Philosophy

**Manual Control > Automation**

Every policy stems from one principle: YOU control the system, it doesn't control you.

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
- systemd user timer ran at boot
- Attempted sudo without credentials
- Triggered faillock after 3 attempts
- Locked user account
- Broke sudo authentication system-wide

**Lesson learned:**
> "Automation at boot + sudo = debugging nightmare"

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
```bash
# systemd timer at boot
[Timer]
OnBootSec=5min
```

**Good (Manual Trigger):**
```bash
# Fish function with confirmation
function weekly-check
    read -P "Continue? (y/N): " response
    if test "$response" = "y"
        ~/0-core/scripts/safe-update
    end
end
```

### Rationale
**Lesson learned:**
> "Anything that runs automatically WILL break mysteriously at the worst time."

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

# Quick edit with auto-lock
edit-core shell-fish
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
- ✅ **ALWAYS** document every decision
- ✅ **ALWAYS** explain failure modes
- ✅ **ALWAYS** log breaking changes
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
- `CHANGELOG-v3.x.md` - What changed per version
- `THEORY_OF_OPERATION.md` - How system works
- `INCIDENTS/` - What broke and why
- `POLICIES.md` - This document

### Rationale
> "Future-you has no memory of current-you's decisions."

**Prevention:**
- Document WHY, not just WHAT
- Record lessons learned
- Make knowledge transferable
- Help others learn

---

## 🔄 Update Policy

### Rules
- ❌ **NEVER** auto-update at boot
- ❌ **NEVER** update without snapshots
- ❌ **NEVER** skip health checks
- ✅ **ALWAYS** use `safe-update` script
- ✅ **ALWAYS** create pre/post snapshots
- ✅ **ALWAYS** verify system health after updates
- ✅ **ALWAYS** check for .pacnew files

### Process
```bash
# Manual update (YOU decide when)
safe-update

# Or prompted weekly check
weekly-check  # Asks for confirmation first
```

### Safety Features
- Pre-update snapshot
- Auto-detect yay issues
- Auto-rebuild on failure
- Post-update snapshot
- .pacnew detection
- Health verification

### Rationale
**Lesson learned:**
> "Updates break things. Be prepared to rollback."

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
# Check system health
dot-doctor  # Must show 100%

# Check git state
git status  # Must be clean

# Check package versions
dotctl status  # Shows all versions
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
- ⚠️  **ALWAYS** consider blast radius before editing
- ⚠️  **ALWAYS** backup critical components first
- ✅ **ALWAYS** use recovery procedures for high-risk edits

### Classification
- 🔴 **Critical:** System unusable if broken (wm-sway)
- 🟠 **High:** Major functionality lost (shell-fish, faelight-bar)
- 🔵 **Medium:** Important but not essential (editor-nvim)
- 🟢 **Low:** Optional features (browser-qutebrowser)

### Before Editing
```bash
# Check blast radius
dotctl status  # Shows color-coded risk

# Edit with awareness
edit-core wm-sway  # High-risk, will warn
```

### Rationale
> "Know what you're risking before you break it."

---

## 🔐 Security Policy

### Rules
- ✅ **ALWAYS** store secrets in KeePassXC
- ✅ **ALWAYS** use encrypted backups
- ❌ **NEVER** commit secrets to git
- ❌ **NEVER** store plaintext passwords
- ❌ **NEVER** expose API keys in configs

### Secrets Management
- **Storage:** ~/vault/passwords.kdbx (KeePassXC)
- **Backup:** filen.io (E2E encrypted)
- **Git:** NEVER commit secrets
- **Configs:** Reference secrets, don't embed

### Current Security
- 73% Lynis hardening score
- LUKS2 full disk encryption
- UFW firewall configured
- fail2ban active
- DNSOverTLS (Quad9)
- Mullvad VPN

---

## 📦 Package Management Policy

### Rules
- ✅ **ALWAYS** use semantic package names
- ✅ **ALWAYS** document package purpose
- ✅ **ALWAYS** track dependencies
- ✅ **ALWAYS** maintain .dotmeta files
- ❌ **NEVER** create generic package names

### Naming Convention
```
<category>-<application>

Examples:
✅ wm-sway (window manager - sway)
✅ shell-fish (shell - fish)
✅ editor-nvim (editor - neovim)
❌ hypr (old, removed)
❌ config (too generic)
```

### Rationale
> "Self-documenting structure > cryptic names"

---

## 🧪 Testing Policy

### Rules
- ✅ **ALWAYS** test changes before committing
- ✅ **ALWAYS** verify health after changes
- ✅ **ALWAYS** check for broken symlinks
- ❌ **NEVER** commit untested changes

### Testing Checklist
```bash
# 1. Test functionality
# Make your changes, test they work

# 2. Check health
dot-doctor  # Must pass 100%

# 3. Verify git
git status  # Review changes

# 4. Commit
git add -A
git commit -m "description"
git push
```

---

## 📋 Violation Response

### If Policy Violated

1. **Stop immediately**
2. **Assess damage**
3. **Rollback if needed** (`git restore` or Btrfs snapshot)
4. **Document incident** (add to `INCIDENTS/`)
5. **Update policy** (prevent recurrence)
6. **Learn lesson** (update `THEORY_OF_OPERATION.md`)

### Example Process
```bash
# Something broke
cd ~/0-core
git status  # What changed?
git restore <file>  # Rollback
lock-core  # Protect again

# Document
nvim docs/INCIDENTS/$(date +%Y-%m-%d)-description.md

# Update policies
nvim docs/POLICIES.md
```

---

## 🎓 Policy Evolution

**These policies are living documents.**

When you:
- Make a mistake → Add a policy
- Find a better way → Update a policy
- Learn a lesson → Document it

**Goal:** Make it impossible to repeat past mistakes.

---

## 📚 Related Documentation

- `THEORY_OF_OPERATION.md` - How system works
- `INCIDENTS/` - What broke and why
- `CHANGELOG-v3.x.md` - Version changes
- `PASSWORD-SOLUTION.md` - The incident that started it all

---

**Remember:** These policies exist because we learned the hard way.

**Don't repeat history. Follow the policies.** 🛡️

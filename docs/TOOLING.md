# 🔧 Faelight Forest - Dotfile Intelligence & Tooling

**Version:** 2.8.0+  
**Purpose:** Diagnostic and maintenance tools for dotfile health

---

## 📋 Table of Contents

- [Overview](#overview)
- [dot-doctor](#dot-doctor---system-health-check)
- [keyscan](#keyscan---keybinding-analysis)
- [Weekly Maintenance](#weekly-maintenance-routine)
- [Troubleshooting](#troubleshooting)

---

## Overview

Faelight Forest includes professional-grade diagnostic tools to monitor dotfile health, detect conflicts, and ensure system stability.

**Available Tools:**
- `dot-doctor` - Comprehensive health checker
- `keyscan` - Keybinding conflict detector

**Philosophy:** Catch problems before they break your workflow!

---

## dot-doctor - System Health Check

**Purpose:** Verify all dotfile components are working correctly

### Usage
```bash
# Run health check
dot-doctor

# Or use aliases
doctor
health-check
```

### What It Checks

**1. Stow Symlinks (8 packages)**
- Verifies all symlinks point to 0-core correctly
- Detects broken or missing symlinks
- Checks: fish, hypr, kitty, waybar, nvim, yazi, mako, gtk

**2. System Services (3 services)**
- Monitors critical background services
- Checks: mullvad-daemon, fail2ban, syncthing
- Reports running/stopped status

**3. Binary Dependencies (17 binaries)**
- Verifies all required commands exist
- Checks: kitty, hyprland, waybar, fish, nvim, etc.
- Warns about missing dependencies

### Example Output
```
🏥 Dotfile Health Check - Faelight Forest
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔗 Checking Stow symlinks...
   ✅ All 8 packages properly stowed

🔄 Checking system services...
   ✅ All 3 services running

📦 Checking binary dependencies...
   ✅ All 17 binaries found

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ All checks passed! System healthy! 🌲
```

### When to Use

- **After system updates** - Verify nothing broke
- **Before major changes** - Establish baseline
- **Weekly maintenance** - Regular health check
- **Troubleshooting** - Diagnose issues quickly

---

## keyscan - Keybinding Analysis

**Purpose:** Analyze keybindings and detect conflicts

### Usage
```bash
# Analyze keybindings
keyscan

# Or use alias
keys-check
```

### What It Analyzes

**1. Total Bindings**
- Counts all keyboard shortcuts
- Currently: 101 keybindings

**2. Grouping by Modifier**
- SUPER (39 bindings)
- SUPER+SHIFT (39 bindings)
- SUPER+CTRL (6 bindings)
- SUPER+ALT (17 bindings)

**3. Conflict Detection**
- Finds duplicate keybindings
- Detects case-sensitivity conflicts (L vs l)
- Highlights problems for fixing

### Example Output
```
🔍 Keybinding Analysis Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📊 Statistics:
   Total bindings: 101
   Conflicts: 0 ✅

📋 Bindings by Modifier:
   SUPER: 39
   SUPER+SHIFT: 39
   SUPER+CTRL: 6
   SUPER+ALT: 17

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### When to Use

- **After adding keybindings** - Check for conflicts
- **Troubleshooting shortcuts** - See what's bound
- **Before major changes** - Verify current state
- **Documentation** - Reference for key availability

### Understanding Conflicts

**Example Conflict:**
```
⚠️  Conflicts Detected:
   🔴 SUPER+K
```

**This means:**
- SUPER+K is bound to multiple actions
- Hyprland will use the LAST definition only
- Earlier bindings are silently ignored

**How to Fix:**
1. Find conflicting bindings: `grep "SUPER, K" ~/0-core/hypr/.config/hypr/bindings.conf`
2. Move less important binding to unused key
3. Reload Hyprland: `hyprctl reload`
4. Verify fix: `keyscan`

---

## Weekly Maintenance Routine

**Recommended schedule:** Every Sunday

### Quick Check (2 minutes)
```bash
# Run health check
dot-doctor

# Verify keybindings
keyscan

# Check security score
security-score

# Update system
sudo pacman -Syu
```

### Full Maintenance (10 minutes)
```bash
# 1. Health diagnostics
dot-doctor
keyscan

# 2. Security audit
security-check  # Runs pacman update + arch-audit + lynis

# 3. Check for configuration drift
cd ~/0-core
git status

# 4. Review services
jail-status      # Fail2ban status
systemctl --user status syncthing

# 5. Backup verification
# (Once backup automation is set up in v2.9)

# 6. Clean up
paru -Sc         # Clean package cache
journalctl --vacuum-time=7d
```

### Health Checklist

**Green Light (Continue normally):**
- ✅ dot-doctor: All checks passed
- ✅ keyscan: 0 conflicts
- ✅ security-score: 71/100+
- ✅ No uncommitted changes

**Yellow Light (Minor attention needed):**
- ⚠️ 1-2 warnings in dot-doctor
- ⚠️ Uncommitted changes in 0-core
- ⚠️ Services not running (but not critical)

**Red Light (Action required):**
- ❌ Any errors in dot-doctor
- ❌ Keybinding conflicts detected
- ❌ Security score dropped significantly
- ❌ Critical services down

---

## Troubleshooting

### dot-doctor Reports Broken Symlinks

**Symptom:**
```
🔗 Checking Stow symlinks...
   ❌ 1 broken symlink detected
```

**Fix:**
```bash
# Re-stow packages
cd ~/0-core
stow --restow */

# Verify
dot-doctor
```

### keyscan Shows Conflicts

**Symptom:**
```
⚠️  Conflicts: 2
   🔴 SUPER+K
   🔴 SUPER SHIFT+L
```

**Fix:**
```bash
# Find conflicting bindings
grep -n "SUPER, K" ~/0-core/hypr/.config/hypr/bindings.conf
grep -n "SUPER SHIFT, L" ~/0-core/hypr/.config/hypr/bindings.conf

# Edit bindings
nvim ~/0-core/hypr/.config/hypr/bindings.conf

# Move one binding to unused key
# Save, reload, verify
hyprctl reload
keyscan
```

### Services Not Running

**Symptom:**
```
🔄 Checking system services...
   ❌ syncthing not running
```

**Fix:**
```bash
# Start service
systemctl --user start syncthing

# Enable on boot
systemctl --user enable syncthing

# Verify
systemctl --user status syncthing
dot-doctor
```

### Missing Binaries

**Symptom:**
```
📦 Checking binary dependencies...
   ❌ swappy not found
```

**Fix:**
```bash
# Install missing package
paru -S swappy

# Verify
which swappy
dot-doctor
```

---

## Future Enhancements (v2.8.6+)

**Planned Features:**
- `dot-doctor --fix` - Auto-repair broken symlinks
- `dot-doctor --report` - HTML health report
- `keyscan --export` - Generate KEYBINDINGS.md
- `dot-lint` - Comprehensive static analysis
- `dot-benchmark` - Performance profiling

See [ROADMAP.md](planning/ROADMAP.md) for details!

---

**Made with 💚 by Christian**  
*Last Updated: December 02, 2025 - v2.8.0*

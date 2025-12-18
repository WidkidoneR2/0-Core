# v3.3.1 - Blast Radius Warnings

**Release Date:** 2025-12-17  
**Type:** Feature Enhancement

---

## 🛡️ BLAST RADIUS AWARENESS

### Enhanced edit-core with Risk Warnings

**New Behavior:**
Before editing any package, `edit-core` now:
1. Reads blast radius from `.dotmeta`
2. Shows color-coded warning based on risk level
3. Displays potential failure modes
4. Requires explicit confirmation
5. Auto-creates backup for critical/high packages

### Warning Levels

**🔴 Critical** (wm-hypr):
- Shows RED warning with failure modes
- Requires typing "CRITICAL" to proceed
- Auto-creates git stash backup
- Example: "Type 'CRITICAL' to proceed:"

**🟠 High** (shell-fish, bar-waybar):
- Shows YELLOW warning with failure modes
- Requires typing "yes" to proceed
- Auto-creates git stash backup
- Example: "Type 'yes' to proceed:"

**🔵 Medium** (editor-nvim, fm-yazi, notif-mako):
- Shows BLUE informational warning
- Simple y/N confirmation
- No auto-backup
- Example: "Continue? (y/N):"

**🟢 Low** (all themes, browsers, tools):
- No warning shown
- Proceeds directly to edit
- No confirmation required

---

## 🎯 FEATURES

### Auto-Backup System
- Critical/high packages automatically backed up before editing
- Uses `git stash` with descriptive message
- Format: "Pre-edit backup: <package> YYYY-MM-DD-HHMM"
- Allows safe recovery if edits break system

### Enhanced sync-0-core
- Auto-detects if core is locked
- Temporarily unlocks for sync
- Pulls and pushes changes
- Re-locks automatically
- Handles conflicts gracefully

---

## 📊 STATISTICS

**Files Changed:** 2  
**Lines Added:** 145+ (enhanced core-protect)  
**Packages Documented:** 15 (all with .dotmeta)  
**Warning Levels:** 4 (critical, high, medium, low)  
**Auto-Backup:** Critical + High packages

---

## 🧪 TESTING

All blast radius levels tested:
- ✅ Critical: RED warning, "CRITICAL" required, backup created
- ✅ High: YELLOW warning, "yes" required, backup created
- ✅ Medium: BLUE warning, y/N confirmation
- ✅ Low: No warning, direct edit

---

## 🔄 UPGRADING

**From v3.3.0:**
```bash
sync-0-core  # Pull latest changes
exec fish    # Reload shell
```

**Testing:**
```bash
# Test each level:
edit-core browser-qutebrowser  # Low - no warning
edit-core editor-nvim          # Medium - blue
edit-core shell-fish           # High - yellow + backup
edit-core wm-hypr              # Critical - red + backup
```

---

## 💡 NEXT: v3.4.0 - Policy Enforcement

Planned features:
- Policy gates (enforce requirements before operations)
- Temporal tracking (incident-free windows)
- Requirement validation (snapshots, health checks)
- `--ack-critical` override flags

---

**v3.3.1 - Smart, Safe, Risk-Aware** 🛡️

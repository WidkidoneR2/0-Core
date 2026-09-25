---
id: 009
date: 2026-02-12
type: incidents
title: "Sudo Authentication Failure - PAM System-Auth Broken After Tool Audit"
status: resolved
tags: [sudo, pam, authentication, system-auth, tool-audit]
---

## Summary
After recent tool audit, sudo completely rejected correct user password while login and `su` authentication worked perfectly. Required bypassing broken `/etc/pam.d/system-auth` configuration.

---

## Timeline

### 2026-02-11 23:18 - Initial Problem
- User attempted `lock-core` command
- Sudo prompted for password
- **Correct password rejected** with "Sorry, try again"
- Login with same password worked fine
- Previous sudo incident (001) was from December 2025 - different root cause

### 2026-02-12 00:00-02:00 - First Recovery Attempts
- Multiple recovery mode boots (`init=/bin/bash`, `rescue.target`)
- Attempted to fix sudoers file (wheel group)
- Reset user password multiple times (temp123, dog911, etc.)
- **Problem persisted** - sudo still rejected password
- Set root password (temp123) for recovery access

### 2026-02-12 09:00-10:00 - Diagnosis Phase
- Verified user in wheel group ✅
- Verified sudoers configuration correct ✅
- Verified `/etc/sudoers.d/00_christian` correct ✅
- **Key discovery:** `su christian` worked with same password that sudo rejected
- **Journalctl revealed:**
```
  pam_unix(sudo:auth): conversation failed
  pam_unix(sudo:auth): auth could not identify password for [christian]
```

### 2026-02-12 10:00-10:15 - Root Cause Identified
- Found zsh `sudo()` wrapper function (red herring - not the issue)
- Discovered `/etc/pam.d/system-auth` was broken
- Tried removing `try_first_pass` parameter (didn't fix it)
- Tried commenting out `pam_systemd_home.so` (didn't fix it)
- Tried fixing `/etc/nsswitch.conf` (didn't fix it)
- **Critical discovery:** Direct `pam_unix.so` works, `system-auth` doesn't

### 2026-02-12 10:15-10:30 - Resolution
- Applied fix: Made `/etc/pam.d/sudo` use `pam_unix.so` directly
- **Bypassed broken system-auth entirely**
- Sudo immediately worked with original password ✅
- Verified survives reboot ✅
- **RESOLVED**

---

## Root Cause

**Technical Analysis:**

`/etc/pam.d/system-auth` configuration was broken (likely modified during recent tool audit).

**Evidence:**
1. December 2025 fix modified system-auth (commented systemd_home, changed nsswitch)
2. Recent tool audit **undid those changes** or introduced new breakage
3. `su` uses `pam_unix.so` directly → worked ✅
4. `sudo` uses `system-auth` → failed ❌

**Why it broke:**
- `/etc/pam.d/sudo` included `system-auth`
- system-auth had complex control flow designed for systemd_home
- Something in system-auth broke sudo's password conversation
- PAM error: "conversation failed" = can't even establish password prompt properly

**Suspected culprits (investigation needed):**
- `faelight-daemon`
- `faelight-snapshot` 
- `core-protect`
- `faelight-link`
- Security hardening changes during audit

---

## Impact

### What Broke
- ❌ All sudo operations system-wide
- ❌ Package management (pacman/yay with sudo)
- ❌ System configuration requiring root
- ❌ `lock-core` command
- ❌ Service management
- ❌ Any privileged operation via sudo

### What Still Worked
- ✅ User login (graphical and TTY)
- ✅ `su` to become root (with temp123)
- ✅ `su christian` (with original password)
- ✅ Desktop environment (Sway)
- ✅ All user-level tools
- ✅ System stability

### Workarounds During Incident
- Used `su` with root password (temp123) for privileged operations
- Could become root, then run commands
- Not sustainable for normal workflow

---

## Resolution

### The Fix

**File:** `/etc/pam.d/sudo`

**Before (broken):**
```
#%PAM-1.0
auth		include		system-auth
account		include		system-auth
session		include		system-auth
```

**After (working):**
```
#%PAM-1.0
auth       required     pam_unix.so
account    required     pam_unix.so
session    required     pam_unix.so
```

**Why this works:**
- Uses same authentication method as `su` (which worked)
- Bypasses broken `system-auth` configuration entirely
- Direct `pam_unix.so` authentication
- Secure, stable, standard approach

### Verification
```bash
sudo -k                    # Clear cached credentials
sudo whoami               # Prompted for password
# Entered original password → SUCCESS ✅
reboot                     # Test persistence
sudo whoami               # Still works ✅
```

---

## What We Tried (That Didn't Work)

### Attempted Fixes
1. ❌ **Uncommented wheel in sudoers** - wasn't the issue
2. ❌ **Reset user password multiple times** - password was never the problem
3. ❌ **Removed `try_first_pass` from system-auth** - didn't fix conversation failure
4. ❌ **Commented out pam_systemd_home** - logic still broken
5. ❌ **Fixed nsswitch.conf** - not the root cause
6. ❌ **Reinstalled sudo package** - binary was fine
7. ❌ **Checked for file locks** - no locks present
8. ❌ **Disabled zsh wrapper** - wasn't interfering

### Red Herrings
- Zsh `sudo()` function wrapper (harmless)
- Keyboard layout issues (user password worked for login/su)
- Faillock/account lockout (no locks present)
- Password corruption (password worked for other auth methods)
- sudoers syntax (configuration was correct)

---

## Lessons Learned

### Critical Insights

1. **Symptom ≠ Root Cause**
   - "Password rejected" doesn't mean password is wrong
   - Could be PAM configuration, conversation failure, etc.

2. **Compare Working vs Broken Authentication**
   - `su` worked, `sudo` didn't → problem in sudo's PAM config
   - Same password, different results = configuration issue

3. **PAM Is Complex**
   - `system-auth` has intricate control flow
   - Designed to work with systemd_home as a unit
   - Commenting out parts breaks the logic

4. **Tools Can Silently Break System Files**
   - Security hardening tools modify PAM configs
   - Changes not always visible in git (system files)
   - Need auditing of what tools touch `/etc/`

### Technical Lessons

**PAM Architecture:**
- `su` uses direct `pam_unix.so` (simple, reliable)
- `sudo` uses `system-auth` (complex, can break)
- Both approaches are valid, direct is more resilient

**Authentication vs Authorization:**
- User was authorized (in wheel group, sudoers correct)
- Authentication mechanism was broken (PAM conversation)
- Separate concerns, separate fixes

**Debugging Methodology:**
- Start simple (check obvious things)
- Compare working vs broken (su vs sudo)
- Check logs for actual errors (journalctl)
- Isolate components (test direct pam_unix)

---

## Prevention

### Immediate Actions
- [x] Document working PAM configuration
- [x] Back up `/etc/pam.d/sudo`
- [x] Create this incident report
- [ ] Audit which tool modified system-auth
- [ ] Add PAM file protection to core-protect (or remove modification capability)

### Long-term Prevention

**Investigation Needed:**
```bash
# Find which tool touches PAM files
grep -r "pam.d\|system-auth\|nsswitch" ~/0-core/rust-tools/*/src/

# Check git history for system file changes
git log --all -p -S "system-auth"

# Audit tool capabilities
# - core-protect: Does it modify PAM?
# - faelight-daemon: Does it touch system auth?
# - Security tools: What do they change?
```

**Policy Changes Needed:**

1. **System File Protection Policy**
   - Tools should NOT modify `/etc/pam.d/` files
   - If security hardening needed, document changes clearly
   - Always back up before modifying system auth

2. **Authentication Testing**
   - After any system changes, test ALL auth methods:
     - Login (TTY and graphical)
     - `su`
     - `su username`
     - `sudo`
   - Create test script for this

3. **Audit Trail**
   - Log what tools modify in `/etc/`
   - Version control system files if possible
   - Document "permanent fixes" in BACKUPS/

---

## Related Incidents

### Incident 001 (December 2025)
- **Issue:** Sudo failed due to faillock at boot
- **Cause:** Systemd timers running sudo without credentials
- **Fix:** Disabled boot automation, modified system-auth
- **Connection:** That incident's "fix" (modifying system-auth) may have been undone by recent audit

### Key Difference
- **Incident 001:** faillock triggering, boot automation
- **Incident 009:** PAM conversation failure, broken system-auth
- **Common thread:** Both involved system-auth modifications

---

## Files Changed

### System Files (Outside Git)
- `/etc/pam.d/sudo` - Direct pam_unix.so authentication (PERMANENT FIX)
- `/etc/pam.d/system-auth` - Attempted fixes, reverted to stock
- `/etc/nsswitch.conf` - Attempted fixes, reverted
- `/etc/shadow` - Password reset attempts (password wasn't the issue)

### Backups Created
- `/etc/pam.d/sudo.backup` - Original system-auth-based config
- `/etc/pam.d/system-auth.backup` - Before modifications
- `/etc/pam.d/system-auth.before-fix` - Before systemd_home comments
- `/etc/nsswitch.conf.backup` - Before systemd removal

### Recommended Backup
```bash
sudo cp /etc/pam.d/sudo ~/0-core/BACKUPS/sudo-working-config-20260212
```

---

## Recovery Procedure

If this happens again:

### Quick Fix
```bash
# 1. Boot to recovery mode
# In GRUB: add "init=/bin/bash"

# 2. Remount filesystem
mount -o remount,rw /

# 3. Fix sudo PAM config
cat > /etc/pam.d/sudo << 'EOF'
#%PAM-1.0
auth       required     pam_unix.so
account    required     pam_unix.so
session    required     pam_unix.so
EOF

# 4. Reboot
reboot -f

# 5. Test
sudo whoami  # Should work
```

### Diagnosis Steps
```bash
# If sudo fails but login works:

# 1. Test su authentication
su username  # Does user password work?

# 2. Check PAM logs
journalctl -n 50 | grep -i "pam\|sudo\|auth"

# 3. Compare configs
cat /etc/pam.d/su        # Working
cat /etc/pam.d/sudo      # Broken?

# 4. If system-auth is broken, bypass it
# Use direct pam_unix.so (the fix above)
```

---

## Technical Deep Dive

### PAM Control Flow (system-auth)

**Original (broken) logic:**
```
-auth      [success=2 default=ignore]  pam_systemd_home.so
auth       [success=1 default=bad]     pam_unix.so
```

This means:
1. Try systemd_home first
2. If it succeeds, skip 2 modules (to pam_permit)
3. If it fails, try pam_unix
4. If pam_unix succeeds, skip 1 module
5. Complex success/failure path

**When systemd_home is commented:**
```
#-auth     [success=2 default=ignore]  pam_systemd_home.so
auth       [success=1 default=bad]     pam_unix.so
```

The `[success=2...]` logic expects systemd_home to exist!
When it doesn't, the control flow breaks.

**Simple approach (our fix):**
```
auth       required     pam_unix.so
```

Direct, simple, works.

### Why `su` Worked But `sudo` Didn't

**`/etc/pam.d/su`:**
```
auth            required        pam_unix.so
```
Direct authentication, no complex control flow.

**`/etc/pam.d/sudo`:**
```
auth		include		system-auth
```
Uses broken system-auth with complex logic.

**Solution:** Make sudo authenticate like su.

---

## Action Items

### Immediate (Today)
- [x] Fix sudo authentication ✅
- [x] Verify fix survives reboot ✅
- [x] Create incident documentation ✅
- [ ] Back up working config to 0-core/BACKUPS/

### Short-term (This Week)
- [ ] Audit which tool modified system-auth
- [ ] Review all Rust tools for `/etc/` modifications
- [ ] Add test script for authentication methods
- [ ] Document "never touch PAM" policy

### Long-term (This Month)
- [ ] Create system file protection strategy
- [ ] Review December 2025 incident (001) connection
- [ ] Add authentication testing to bump-system-version
- [ ] Consider whether system-auth should be in version control

---

## Notes

**Root Password Set:** temp123 (for emergency recovery)
- Created during troubleshooting
- Allows `su` to become root
- Keep for recovery purposes

**Original User Password:** Preserved and working
- Never was the actual problem
- Works for login, su, and sudo
- No need to change

**PAM Philosophy:**
> "Simple is better than complex. Direct pam_unix.so authentication is more resilient than complex system-auth control flow."

---

## Impact on Architecture

**Before:**
- Assumed system-auth was stable
- Tools could modify PAM configs during audits
- No protection for critical authentication files

**After:**
- PAM configs are critical infrastructure
- Tools should NOT modify authentication stack
- Simple, direct authentication preferred over complex flows
- Need auditing and protection for `/etc/` changes

**New Understanding:**
- System files outside git are vulnerable
- Tool audits can break authentication
- Always test auth after system changes
- Keep working backups of PAM configs

---

**Duration:** 3+ hours active debugging (plus overnight troubleshooting)  
**Severity:** CRITICAL - System privileged operations completely broken  
**Status:** RESOLVED ✅  
**Resolution:** Bypass broken system-auth, use direct pam_unix.so  

**Reviewed:** 2026-02-12  
**Next Review:** 2026-03-12 (1 month)  

---

**This incident reinforces that authentication is CRITIC

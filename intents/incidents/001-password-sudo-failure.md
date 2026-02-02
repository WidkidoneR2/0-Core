---
id: 001
date: 2025-12-14
type: incidents
title: "Password/Sudo Authentication Failure"
status: resolved
severity: critical
duration: "12 hours"
tags: [systemd, sudo, faillock, authentication, boot]
---

## Summary

Systemd user timer running at boot attempted sudo operations without credentials, triggered faillock after 3 failed attempts, completely broke sudo authentication system-wide. Required 12 hours of debugging to identify root cause and resolve.

---

## Timeline

### 09:00 - Initial Problem

- Attempted to use sudo
- Received "Authentication failure" immediately
- Password definitely correct
- No obvious error messages

### 09:30 - Investigation Begins

- Checked /var/log/auth.log
- Found repeated faillock triggers
- Discovered account locked
- Unlocked with faillock --user christian --reset
- Problem returned on next reboot

### 11:00 - Deep Investigation

- Checked systemd timers
- Found dotfiles-backup.timer
- Timer ran at boot
- Script contained sudo commands
- No credential handling

### 15:00 - Root Cause Identified

**The smoking gun:**

```bash
# dotfiles-backup.timer ran at boot
# backup script tried: sudo rsync ...
# No credentials available at boot
# Failed 3 times → faillock triggered
# Account locked before user even logged in
```

### 21:00 - Resolution

- Disabled all systemd user timers
- Removed boot-time automation
- Created manual-trigger scripts
- Added confirmation prompts
- Tested thoroughly
- Problem resolved

---

## Root Cause

**Technical details:**

- Component: systemd user timer (dotfiles-backup.timer)
- Configuration: `OnBootSec=5min`
- Script: backup-dotfiles.sh contained `sudo rsync`

**Why it broke:**

1. Timer started 5 minutes after boot
2. User not yet logged in / no credentials available
3. Script attempted sudo command
4. Failed (no password available)
5. Retry logic attempted 3 times
6. faillock triggered after 3 failures
7. Account locked before user login
8. All subsequent sudo attempts failed

**Underlying issue:**

> "Automation + privileged operations + boot timing = authentication disaster"

---

## Impact

### What Broke

- All sudo operations system-wide
- Package management (pacman/yay)
- System configuration changes
- File system modifications
- Service management
- Any privileged operation

### What Still Worked

- User-level applications
- Desktop environment (Hyprland)
- Non-privileged commands
- Web browsing, editing, etc.
- Basic system functionality

### Workarounds Attempted

- Reboot (problem persisted)
- Different password entry methods
- TTY login (same issue)
- Recovery mode consideration

---

## Resolution

### Immediate Fix

```bash
# 1. Unlock account
sudo faillock --user christian --reset

# 2. Disable problematic timer
systemctl --user disable dotfiles-backup.timer
systemctl --user stop dotfiles-backup.timer

# 3. Remove timer file
rm ~/.config/systemd/user/dotfiles-backup.timer

# 4. Verify no other boot timers
systemctl --user list-timers

# 5. Reboot and test
reboot
# sudo now works!
```

### Long-term Prevention

1. **Created Authentication Policy** - No sudo in automated scripts
2. **Created Automation Policy** - No boot-time automation
3. **Rewrote backup script** - Manual trigger only with confirmation
4. **Added safe-update** - Smart update system with manual control
5. **Updated documentation** - Warning against boot automation

---

## Lessons Learned

1. **What went wrong:** Automation at boot + sudo = unpredictable failures
2. **Why it happened:** Assumed boot environment would have credentials available
3. **How to prevent:** NEVER automate privileged operations, especially at boot

### Key Insights

**Technical lesson:**

> "Boot environment ≠ User environment. Credentials, environment variables, and context are different."

**Philosophical lesson:**

> "Convenience (automation) < Reliability (manual control)"

**Practical lesson:**

> "12 hours of debugging teaches you more than 12 weeks of smooth sailing."

---

## Policy Changes

**New policies added:**

### Authentication Policy (docs/POLICIES.md)

- ❌ Never use sudo in automated scripts
- ❌ Never prompt for passwords at boot
- ✅ Always require manual confirmation for sudo

### Automation Policy (docs/POLICIES.md)

- ❌ Never schedule scripts at boot
- ❌ Never use systemd timers for maintenance
- ✅ Always require explicit user trigger
- ✅ Always use confirmation prompts

### Implementation

- Created safe-update (manual trigger, auto-recovery)
- Created weekly-check (confirmation prompt)
- Removed all systemd user timers
- Added manual-only philosophy to documentation

---

## Related

- **Policy:** docs/POLICIES.md (Authentication & Automation sections)
- **Code Changes:** Removed systemd timers, created safe-update
- **Documentation:** PASSWORD-SOLUTION.md (original investigation)
- **System Changes:** v3.1.0 → v3.2.0 transformation

---

## Recovery Procedure

If this happens again (it shouldn't, but if it does):

```bash
# 1. Check if account is locked
sudo faillock --user $USER

# 2. Unlock account
sudo faillock --user $USER --reset

# 3. Check for boot-time sudo attempts
systemctl --user list-timers
journalctl --user -u '*timer*' --since today

# 4. Disable problematic timers
systemctl --user disable <timer-name>
systemctl --user stop <timer-name>

# 5. Check auth logs
sudo grep -i "authentication failure" /var/log/auth.log
sudo grep -i "faillock" /var/log/auth.log

# 6. Verify resolution
sudo echo "test"  # Should work

# 7. Remove automation permanently
# Never re-enable boot-time sudo automation!
```

---

## Impact on Architecture

This incident fundamentally changed the 0-core philosophy:

**Before:**

- Some automation was acceptable
- Convenience over predictability
- "Set and forget" mentality

**After:**

- Manual control always
- Predictability over convenience
- "You control when things run" philosophy

**Result:**

- More reliable system
- Easier to debug
- No mysterious failures
- User always in control

---

**Reviewed:** 2025-12-17  
**Next Review:** 2026-01-14 (1 month)

---

**This incident created the foundation for 0-core's manual-control philosophy. Every policy we have exists because of lessons like this one.**

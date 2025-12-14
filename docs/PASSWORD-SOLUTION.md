# Password Issue - SOLVED (December 14, 2025)

## Problem
After every reboot, sudo would fail with password rejection.

## Root Cause
Systemd user timers running at boot:
- dotfiles-backup.timer
- weekly-maintenance.timer

These scripts tried to run sudo without credentials, triggering pam_faillock which locked the account after 3 failed attempts.

## Solution
1. Stopped automation timers:
```bash
   systemctl --user stop dotfiles-backup.timer
   systemctl --user stop weekly-maintenance.timer
```

2. Disabled them from auto-starting:
```bash
   systemctl --user disable dotfiles-backup.timer
   systemctl --user disable weekly-maintenance.timer
```

## Current Working State
- Password: Contains @ symbol (works everywhere)
- Login: ✅ Works
- su -: ✅ Works
- sudo: ✅ Works
- Survives reboot: ✅ YES!

## Permanent Fixes Applied
1. /etc/sudoers - wheel group enabled
2. /etc/nsswitch.conf - passwd: files (no systemd)
3. /etc/pam.d/system-auth - systemd_home commented out
4. Boot automation disabled

## Lesson Learned
NEVER run sudo in boot-time automation without proper credentials!
Use systemctl --user for user services carefully.

## Debugging Duration
~12 hours over 2 days
Multiple reboots, PAM investigations, snapshot tests
PERSEVERANCE PAID OFF! 🏆

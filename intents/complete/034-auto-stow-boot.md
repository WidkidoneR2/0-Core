---
id: 034
date: 2026-01-10
type: future
title: "Auto-Stow on Boot"
status: complete
tags: [rust, stow, systemd, boot, safety]
---

## The Problem
If stow packages drift or symlinks break, the system can be in an inconsistent state without knowing it.

## The Solution
Automatic stow verification and repair on boot/login.

## Implementation Options

### Option A: Systemd User Service
```ini
[Unit]
Description=Faelight Stow Verification
After=graphical-session.target

[Service]
Type=oneshot
ExecStart=/home/christian/0-core/scripts/faelight-stow-check

[Install]
WantedBy=default.target
```

### Option B: Login Hook
Add to `.zshrc` or Sway startup:
```bash
faelight stow verify --quiet --fix
```

### Option C: Dedicated Tool
`faelight-stow` Rust tool:
- `faelight stow verify` - Check all symlinks
- `faelight stow fix` - Re-stow broken packages
- `faelight stow status` - Show stow health

## Behavior
1. On boot/login, verify all stow symlinks
2. If broken links found:
   - Log to `~/.local/state/faelight/stow.log`
   - Auto-fix with `stow -R <package>`
   - Send notification via faelight-notify
3. If all good, silent success

## Safety
- Never delete user files
- Backup before re-stowing
- Log all changes
- Dry-run option

## Integration
- Add `stow` subcommand to faelight unified binary
- Hook into dot-doctor health check
- Optional systemd service

## Success Criteria
- [ ] Detects broken symlinks on boot
- [ ] Auto-repairs without data loss
- [ ] Sends notification on issues
- [ ] Logs all actions
- [ ] Silent when healthy

---
_The forest maintains its roots._ 🌲🔗

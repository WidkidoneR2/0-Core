---
id: 2026-02-03-systemd
date: 2026-02-03
type: incident
title: "Numbered Gravity Breaking systemd Service Paths"
status: resolved
severity: high
tags: [systemd, numbered-gravity, daemon]
---

# Incident: Numbered Gravity Breaking systemd Service Paths


## Date: 2026-02-03
## Severity: HIGH (blocked shutdown)
## Status: RESOLVED

## Summary
faelight-daemon.service prevented system shutdown due to hardcoded path that broke during numbered gravity migration.

## Root Cause
Service file referenced:
```
ExecStart=%h/0-core/target/release/faelight-daemon
```

But numbered gravity moved target/ to:
```
ExecStart=%h/0-core/04-runtime/target/release/faelight-daemon
```

## Impact
- Service crashed in loop (34 restarts in ~3 minutes)
- Blocked clean shutdown
- Required manual power off

## Detection
User noticed laptop wouldn't shutdown. Investigation showed:
```
Feb 02 11:10:56 fealight (faelight-daemon)[4051]: faelight-daemon.service: 
Failed at step EXEC spawning /home/christian/0-core/target/release/faelight-daemon: 
No such file or directory
```

## Resolution
1. Stopped service: `systemctl --user stop faelight-daemon.service`
2. Disabled service: `systemctl --user disable faelight-daemon.service`
3. Fixed path in service file
4. Added service to stow for version control

## Prevention
- [ ] Audit ALL systemd service files for hardcoded paths (Intent 076)
- [ ] Create systemd-user stow package
- [ ] Add path validation to bump-system-version pre-flight
- [ ] Consider faelight-core paths module for service files

## Related
- Intent 076: Path Resilience Audit
- Intent 077: Tool Hardening Sprint
- Numbered gravity migration (v8.7.0)

## Files Affected
- `~/.config/systemd/user/faelight-daemon.service` (FIXED)
- Now in: `03-interfaces/stow/systemd-user/.config/systemd/user/`

## Lessons Learned
Numbered gravity broke MORE than just tool paths:
- Service definitions
- Possibly other config files
- Need systematic audit of ALL path references

This validates Intent 076 - path resilience is CRITICAL.

---
id: 2026-02-03-shutdown
date: 2026-02-03
type: incident
title: "Shutdown Broken - Numbered Gravity Migration"
status: resolved
severity: critical
tags: [shutdown, systemd, numbered-gravity]
---

# Incident: System Shutdown Broken After Numbered Gravity

**Date:** 2026-02-02  
**Duration:** ~4 hours  
**Severity:** CRITICAL - System couldn't shutdown without holding power button  
**Status:** RESOLVED  

## Summary

After numbered gravity restructuring, system shutdown via faelight-menu completely failed, requiring manual power button hold to shutdown.

## Root Causes

1. **Missing logs/ directory** - `~/0-core/logs/` didn't exist, causing graceful-poweroff to exit with `set -e` when `tee` failed
2. **Sway exit killed script** - When graceful-poweroff exited Sway manually, systemd-logind killed the script before systemctl poweroff could run

## What Was Broken

- ❌ faelight-menu → Shutdown didn't work
- ❌ graceful-poweroff script couldn't create log (no logs/ dir)
- ❌ Script died when Sway session ended
- ✅ Sway config paths (already fixed)
- ✅ systemd service paths (already fixed)

## Solution

1. Created `~/0-core/logs/` directory
2. Removed manual Sway exit from scripts - let systemctl handle it
3. Added setsid wrapper in faelight-menu (was actually not needed in final solution)

## What We Learned

### Red Herrings (took hours but weren't the issue):
- Path restructuring (paths were actually fine via setsid)
- Sway deadlock theory (wasn't the real problem)
- systemd-run needed (not needed)
- Complex process tree detachment (overcomplicated)

### Actual Issues:
- Missing directory caused silent failure (set -e is unforgiving)
- Exiting Sway manually killed user session before poweroff could execute

## Files Changed

- `scripts/graceful-poweroff` - v3→v4 (removed Sway exit, let systemd handle)
- `scripts/graceful-reboot` - v3→v4 (same fix)
- `rust-tools/faelight-menu/src/paths.rs` - Added centralized paths module
- `rust-tools/faelight-menu/src/main.rs` - Simplified shutdown/reboot actions
- `logs/` - Created directory (should be in bootstrap)

## Action Items

- [ ] Add logs/ directory creation to bootstrap process
- [ ] Intent 076: Comprehensive path audit CRITICAL
- [ ] Intent 078: Better error handling in scripts (fail gracefully, not silently)
- [ ] Consider: Scripts should create their own log directories if missing

## Related

- Numbered Gravity migration (caused this)
- Intent 076 (Path Resilience Audit)
- Intent 077 (Tool Hardening Sprint) - faelight-menu now v2.0.0

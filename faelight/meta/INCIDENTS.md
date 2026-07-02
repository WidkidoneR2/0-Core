
## [2026-02-11] Sudo Authentication Failure

**Symptoms:**
- Sudo password rejected despite being correct
- PAM conversation failures in logs
- Login password worked fine
- All privilege escalation (sudo, su, pkexec) failed

**Root Cause:**
- PAM authentication state corruption in memory
- Possibly triggered by faelight-daemon testing
- systemd-logind or PAM module stuck state

**Resolution:**
- Simple reboot resolved the issue
- No configuration changes needed

**Prevention:**
- Created `check-auth-health` script
- Created `reset-auth` emergency script
- Added monitoring aliases
- Document to reboot if auth fails

**Related Tools:**
- scripts/check-auth-health
- scripts/reset-auth
- aliases: auth-health, reset-auth

### Solution Implemented:
- Created auto-clearing faillock monitoring
- Added daily silent faillock reset
- `auth-health` now auto-fixes issues
- `reset-auth` clears faillock immediately

### Prevention:
- Faillock auto-resets daily
- Monitoring catches issues early
- Scripts available: auth-health, reset-auth

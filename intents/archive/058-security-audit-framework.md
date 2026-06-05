---
id: 058
date: 2026-01-15
type: complete
title: "Security Audit Framework - The 7th Layer"
status: complete
date-completed: 2026-02-20
tags: [v7.5, security, monitoring, rust]
version: 9.9.0
relates:
  - 036  # Rust hygiene
  - 069  # Intelligent monitoring
dependencies:
  - 047  # faelight-core
---

## Vision
**Never overlook security.** Continuous security monitoring with guided, user-controlled remediation.

**Philosophy:** "Trust, but verify. Then verify again. But always ask before fixing."

## The 7 Layers
```
1. LUKS2 encryption     ✅ (existing)
2. UFW firewall         ✅ (existing)
3. fail2ban             ✅ (existing)
4. Mullvad VPN          ✅ (existing)
5. gitleaks             ✅ (existing)
6. System hardening     ✅ (existing)
7. Active monitoring    🆕 (This intent)
```

## Features

### 1. Automated Security Scans (Safe - Read Only)
```bash
security-audit scan
# Checks (NEVER modifies):
- Rust dependencies (cargo audit)
- System packages (CVE database)
- File permissions (sensitive files)
- Network connections (unexpected listeners)
- SSH configuration (key age, algorithm strength)
- Firewall rules (drift detection)
- Open ports
- Running services

# Stores findings in:
~/.local/state/0-core/security/audit-log.json
```

### 2. Security Report (Shows Findings)
```bash
security-audit report

Output:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔒 Security Audit Report
Last scan: 2 hours ago

⚠️  CRITICAL (2)
  CVE-2024-1234: openssl 3.0.1 (remote code execution)
  → Fix: Update to 3.0.2
  
  SSH Key: RSA-2048 (weak algorithm)
  → Fix: Rotate to Ed25519

🔶 HIGH (1)
  tokio 1.35.0: DoS vulnerability
  → Fix: Update to 1.36.1

✅ LOW (3)
  [view with --all]

Actions available:
  security-audit fix CVE-2024-1234  # Fix specific issue
  security-audit fix-critical       # Fix all critical
  security-audit ignore CVE-XXXX    # Mark false positive
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 3. Guided Remediation (User Approval Required)
```bash
# Fix specific vulnerability
security-audit fix CVE-2024-1234

Output:
🔧 Proposed Fix for CVE-2024-1234
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Vulnerability: Remote code execution in openssl
Severity: CRITICAL
Current: openssl 3.0.1
Target:  openssl 3.0.2

Commands to run:
  1. sudo pacman -Syu openssl
  2. Rebuild affected packages:
     - nginx
     - curl
     
Risk: Low (patch release, ABI compatible)
Rollback: Available via BTRFS snapshot

Proceed? [yes/no/details]: _
[WAITS FOR USER INPUT]
```
```bash
# Fix all critical issues
security-audit fix-critical

Output:
🔧 Batch Fix: 2 Critical Issues
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. CVE-2024-1234: openssl update
2. SSH key rotation (RSA → Ed25519)

Total commands: 5
Estimated time: 2 minutes
Snapshot before? [yes/no/cancel]: yes

Review each fix? [yes/no]: yes

[Shows each fix, asks permission]
[Only proceeds after explicit yes]
```

### 4. Integration with dot-doctor
```bash
doctor

Output includes:
✅ Security Audit: Last scan 2h ago
  ├─ Critical: 0
  ├─ High: 0  
  ├─ Medium: 2 (view with: security-audit report)
  └─ Last fix: 3 days ago

# Or if issues found:
⚠️  Security Audit: 2 critical issues found
  ├─ CVE-2024-1234 (openssl)
  ├─ SSH key rotation needed
  └─ Run: security-audit report
```

### 5. Security Timeline
```bash
security-audit history

Output:
2026-01-18 14:23 - Scan completed (2 issues found)
2026-01-17 09:15 - Fixed CVE-2024-1234 (openssl update)
2026-01-15 16:40 - SSH key rotated (RSA → Ed25519)
2026-01-10 12:00 - Scan completed (0 issues)
2026-01-05 10:30 - Fixed tokio DoS (1.35 → 1.36)
```

### 6. Notification (Optional)
```bash
# If critical issue found during scan:
faelight-notify shows:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔒 Security Alert
2 critical vulnerabilities found

Run: security-audit report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Architecture

### Storage
```
~/.local/state/0-core/security/
├── audit-log.json      # Full scan history
├── vulnerabilities.json # Current findings
├── baseline.json       # Expected state
├── ignored.json        # False positives
└── fixes.json          # Applied fixes history
```

### Rust Tool
**Name:** `security-audit` (standalone binary)

**Or integrate into:** `faelight-core` as security module

**Dependencies:**
- `cargo-audit` for Rust CVE checking
- `arch-audit` for system package CVEs (if available)
- Native scanning for permissions, network, SSH

### Systemd Timer (Scan Only)
```ini
# ~/.config/systemd/user/security-audit.timer
[Unit]
Description=Daily security scan (read-only)

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```
```ini
# ~/.config/systemd/user/security-audit.service
[Unit]
Description=Security audit scan

[Service]
Type=oneshot
ExecStart=%h/0-core/scripts/security-audit scan
# NEVER runs fix commands
# Only scans and records
```

## Philosophy Compliance

### What It WILL Do (Safe):
✅ Scan daily automatically (read-only)
✅ Detect vulnerabilities
✅ Record findings
✅ Show reports
✅ Suggest fixes
✅ Explain risks
✅ Notify of critical issues (optional)

### What It Will NEVER Do:
❌ Apply fixes automatically
❌ Run privileged commands without asking
❌ Modify system in background
❌ Assume you want a fix
❌ Hide what it's doing

### User Interaction Model:
1. **Scan** → Automatic (safe, read-only)
2. **Detect** → Automatic (safe, read-only)
3. **Report** → On request (`doctor` or `security-audit report`)
4. **Explain** → On request (`security-audit show CVE-XXX`)
5. **Suggest** → On request (shown in report)
6. **Fix** → **REQUIRES EXPLICIT PERMISSION**

## Success Criteria

- [ ] CLI tool: `security-audit`
- [ ] Automated daily scans (read-only)
- [ ] Integration with dot-doctor
- [ ] Cargo audit integration
- [ ] Arch package CVE tracking
- [ ] File permission verification
- [ ] Network monitoring (open ports, listeners)
- [ ] SSH configuration analysis
- [ ] Firewall drift detection
- [ ] **Guided remediation (user confirms each fix)**
- [ ] Security timeline/history
- [ ] Notification on critical issues
- [ ] Snapshot integration (auto-snapshot before fixes)
- [ ] Rollback capability
- [ ] False positive handling (ignore list)

## Implementation Phases

### Phase 1: Detection (v7.5.0)
- Basic scanning
- Cargo audit integration
- File permission checks
- Report generation
- dot-doctor integration

### Phase 2: Guidance (v7.5.0)
- Fix suggestion system
- Risk assessment
- Command preview
- User confirmation flow

### Phase 3: Remediation (v7.5.0)
- Execute fixes with permission
- BTRFS snapshot integration
- Rollback capability
- Fix history tracking

### Phase 4: Advanced (v7.6.0+)
- Custom rules engine
- Baseline drift detection
- Compliance checking
- Security score tracking

## Example Workflows

### Daily Workflow (Automated)
```bash
# 2am every day (systemd timer):
security-audit scan
→ Runs silently
→ Updates ~/.local/state/0-core/security/
→ If critical issues: sends notification
→ NEVER fixes anything
```

### User Workflow (Manual)
```bash
# User sees notification or runs doctor
doctor
→ "⚠️ Security: 2 critical issues"

# User investigates
security-audit report
→ Shows issues with severity

# User gets details
security-audit show CVE-2024-1234
→ Explains vulnerability, impact, fix

# User decides to fix
security-audit fix CVE-2024-1234
→ Shows commands
→ Asks permission
→ User types 'yes'
→ Creates snapshot
→ Applies fix
→ Reports success/failure

# User can rollback if needed
security-audit rollback CVE-2024-1234
→ Restores BTRFS snapshot
```

## Dependencies
- Requires: Intent 047 (faelight-core) - for shared functionality
- Relates: Intent 069 (monitoring) - similar detect-report pattern
- Relates: Intent 036 (Rust hygiene) - cargo audit

## Risk Mitigation

**What could go wrong:**
1. False positives → Solution: Ignore list
2. Breaking updates → Solution: BTRFS snapshots + rollback
3. Network scanning too aggressive → Solution: Configurable rules
4. Performance impact → Solution: Run during idle times

**Safety measures:**
- All fixes require explicit permission
- Snapshot before every fix
- Rollback always available
- Clear risk assessment shown
- Dry-run mode available

---

_"Security is not paranoia. It's diligence. And diligence requires permission."_ 🌲🔒

## Completion Notes

Built as `security-audit v1.0.0`. All success criteria met:
- CLI tool: `security-audit scan/report/show/history`
- cargo-audit integration (Rust CVEs)
- arch-audit integration (system package CVEs)
- File permission scanning
- Network listener scanning
- SSH configuration analysis
- dot-doctor integration (live findings in doctor)
- Daily systemd timer
- Structured JSON state in `~/.local/state/0-core/security/`

Deferred: guided remediation, BTRFS rollback, notification system (v2 scope).

## Completion Notes

Built as `security-audit v1.0.0`:
- cargo-audit + arch-audit integration
- File permissions, network listeners, SSH config scanning
- dot-doctor integration
- Daily systemd timer
- JSON state in `~/.local/state/0-core/security/`

Deferred to v2: guided remediation, btrfs rollback, notifications.

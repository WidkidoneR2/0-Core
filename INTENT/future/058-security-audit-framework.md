---
id: 058
date: 2026-01-15
type: future
title: "Security Audit Framework - The 7th Layer"
status: planned
tags: [v7.5, security, monitoring, rust]
version: 7.5.0
relates:
  - 036  # Rust hygiene
  - 069  # Intelligent monitoring
dependencies:
  - 047  # faelight-core
---

## Vision
**Never overlook security.** Automated, continuous security auditing as the 7th layer of defense.

**Philosophy:** "Trust, but verify. Then verify again."

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

### 1. Daily Security Scans
```bash
security-audit run
# Checks:
- Rust dependencies (cargo audit)
- System packages (CVE database)
- File permissions (sensitive files)
- Network connections (unexpected listeners)
- SSH configuration (key rotation needed?)
- Firewall rules (drift detection)
```

### 2. Integration with dot-doctor
```bash
# New health check section:
✅ Security Audit: Last scan 2h ago, no issues
  ├─ Rust deps: 0 vulnerabilities
  ├─ System CVEs: 0 high, 2 low
  ├─ Permissions: All correct
  └─ Network: All expected
```

### 3. Automated Remediation
```bash
security-audit fix
# Auto-fixes:
- Updates vulnerable Rust deps
- Rotates old SSH keys
- Fixes file permissions
- Updates firewall rules
```

### 4. Security Timeline
```bash
security-audit history
2026-01-15: CVE-2024-XXXX in openssl (patched)
2026-01-10: SSH key rotation (30 days old)
2026-01-05: Rust dep: tokio 1.35 → 1.36 (security)
```

## Architecture

**Storage:** `~/.local/state/0-core/security/`
- `audit-log.json` - Full history
- `vulnerabilities.json` - Current issues
- `baseline.json` - Expected state

**Rust Tool:** `faelight-security` or integrated into `faelight-core`

## Success Criteria
- [ ] CLI tool: `security-audit`
- [ ] Daily automated scans via systemd timer
- [ ] Integration with dot-doctor
- [ ] Cargo audit integration
- [ ] CVE tracking for system packages
- [ ] Network monitoring
- [ ] File permission verification
- [ ] Automated remediation for safe fixes
- [ ] Security timeline/history
- [ ] Email/notification on critical issues

## Timeline
**v7.5.0** - Foundation (scanning, reporting)
**v7.6.0** - Automated remediation
**v7.7.0** - ML anomaly detection

## Dependencies
- Requires: Intent 047 (faelight-core)
- Relates: Intent 069 (monitoring)
- Relates: Intent 036 (Rust hygiene)

---

_"Security is not paranoia. It's diligence."_ 🌲🔒

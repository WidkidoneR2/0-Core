# 🏥 Dot-Doctor v3.2.0

**Comprehensive system health monitoring for Linux desktop environments**

Dependency-aware health checking engine that validates your entire desktop stack - from stow symlinks to security hardening.

## ✨ Features

### Smart Health Checks (19 Total)
- 🔗 **Stow Symlinks** - Verifies all dotfile packages properly deployed
- 💔 **Broken Symlinks** - Comprehensive scan of ~/.config and stow directory
- 📋 **Intent Ledger** - Validates intent system with all categories (decisions, experiments, philosophy, future, cancelled, deferred, incidents)
- 🔒 **Security Hardening** - Checks UFW firewall, fail2ban, Mullvad VPN, SSH hardening
- 🔧 **System Services** - Monitors running daemons
- 📦 **Binary Dependencies** - Ensures all required tools available
- 🎨 **Yazi Plugins** - File manager plugin validation
- ⚙️ **Faelight Config** - TOML configuration validation
- ⌨️ **Sway Keybinds** - Duplicate keybinding detection
- 📝 **Scripts** - Core script availability
- 🎯 **Path Resilience** - Migration tracking
- 💿 **Disk Space** - Storage monitoring
- 🦀 **Rust Toolchain** - Development environment check
- 🔀 **Git Repository** - Uncommitted changes detection
- 👤 **Profile System** - Profile configuration validation
- 🎭 **Theme Packages** - Theme availability
- 📊 **Alias Coverage** - Shell alias verification
- ⚙️ **Tool Installation** - Key tool installation status
- 📦 **Package Metadata** - Documentation validation

### Dependency Awareness
- Checks have explicit dependencies (e.g., broken_symlinks depends on stow)
- Failed dependencies block dependent checks
- Dependency graph visualization with `--explain`

### Exit Codes
- `0` - All checks passed
- `1` - One or more checks failed
- `2` - (with --fail-on-warning) Warnings present

## 📦 Installation

### Standalone (Any Linux System)
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/0-Core.git
cd 0-Core/rust-tools/dot-doctor

# Build and install
cargo install --path .

# Verify
dot-doctor --version
```

### Dependencies

**Required:**
- None! (self-contained)

**Optional (for specific checks):**
- `stow` - For stow symlink verification
- `mullvad` - For VPN status check
- `fail2ban` - For security hardening check
- `yazi` - For file manager plugin check

## 🚀 Usage

### Basic Operations
```bash
# Run all health checks
dot-doctor

# Output:
# 🏥 0-Core Health Check - Faelight Forest v9.4.0
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ✅ Stow Symlinks: All 13/13 packages properly stowed
# ❌ Broken Symlinks: 7 broken symlinks found
# ✅ Intent Ledger: 42 intents (10 complete, 7 planned)
# ✅ Security Hardening: 3 protections active
# ...
# Health: 89%

# Quiet mode (for scripts)
dot-doctor --quiet
echo $?  # 0 = healthy, 1 = issues

# Show dependency graph
dot-doctor --explain

# Fail on warnings (stricter)
dot-doctor --fail-on-warning
```

### Examples

#### Daily Health Check
```bash
# Morning routine
dot-doctor
# Review any failed checks
```

#### CI/CD Integration
```bash
#!/bin/bash
# System integrity check in CI pipeline
if ! dot-doctor --quiet; then
    echo "System health issues detected!"
    dot-doctor  # Show details
    exit 1
fi
```

#### Cron Job Monitoring
```bash
# /etc/cron.daily/system-health
#!/bin/bash
if ! /usr/local/bin/dot-doctor --quiet; then
    /usr/local/bin/dot-doctor | mail -s "System Health Issues" admin@example.com
fi
```

#### Pre-commit Hook
```bash
# .git/hooks/pre-commit
#!/bin/bash
# Verify system health before commit
dot-doctor --fail-on-warning || {
    echo "Fix system health issues before committing"
    exit 1
}
```

## 🔍 Health Checks Explained

### Stow Symlinks
Verifies GNU Stow has properly deployed all dotfile packages. Checks that expected symlinks exist and point to correct targets.

### Broken Symlinks
**COMPREHENSIVE SCAN** (v3.2.0+):
- Searches ~/.config (depth 6)
- Searches stow directory (depth 6)
- Reports exact broken link paths
- Previously only checked 5 specific directories - now finds ALL broken links!

### Intent Ledger
**ACCURATE COUNTING** (v3.2.0+):
- Scans ALL categories: decisions, experiments, philosophy, future, cancelled, deferred, incidents
- Previously missed "cancelled" and "deferred" categories
- Validates proper intent file format

### Security Hardening
Checks multiple security layers:
- UFW firewall active
- fail2ban running
- Mullvad VPN connected
- SSH root login disabled

## 💡 Troubleshooting

### Broken Symlinks Found
```bash
# See which symlinks are broken
dot-doctor | grep -A10 "Broken Symlinks"

# Find and remove broken symlinks
find ~/.config -xtype l -delete
```

### Intent Ledger Shows Wrong Count
Fixed in v3.2.0! Now scans all categories correctly.

### Stow Package Failed
```bash
# Re-stow the package
cd ~/0-core
stow -R <package-name> --dir=03-interfaces/stow
```

## 🎯 Design Philosophy

**Trust, but verify** - Every check is validated against reality:
- Intent Ledger: Scans actual intent files in all categories
- Broken Symlinks: Comprehensive filesystem scan, not just specific dirs
- Security: Checks actual service status, not config files
- Stow: Verifies actual symlink targets

**Dependency awareness** - Checks run in dependency order, failed deps block dependent checks.

**Exit codes for automation** - Silent mode with clear exit codes for scripting.

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## 🤝 Part of 0-Core

Dot-doctor is part of the 0-Core ecosystem but works perfectly standalone for any Linux desktop environment!

## 📄 License

Intentional Stewardship - Manual control over automation.

## v4.0.0 - Flagship Intelligent Monitoring

### New Power Features

**Watch Mode - Continuous Monitoring:**
```bash
doctor --watch              # Monitor every 60 seconds
doctor --watch --interval 30  # Custom interval
```

**Dry-Run Fixes:**
```bash
doctor --fix-dry-run        # Preview fixes without applying
doctor --fix                # Apply fixes (existing)
```

**Selective Checks:**
```bash
doctor --skip git           # Skip git checks
doctor --skip security --skip disk  # Skip multiple
doctor --check stow         # Run only stow check (existing)
```

**Health Thresholds:**
```bash
doctor --min-health 95      # Fail if health < 95%
# Perfect for CI/CD pipelines
```

### Integration Examples

**CI/CD Pipeline:**
```yaml
- name: Health Check
  run: doctor --min-health 90 --fail-on-warning
```

**Cron Monitoring:**
```bash
# Monitor every hour, alert if <95%
0 * * * * doctor --min-health 95 || notify-send "Health Alert"
```

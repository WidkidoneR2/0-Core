# core-diff v1.2.0

📊 **Package-Aware Diff Tool** - Risk-based change analysis for 0-Core with shell policy enforcement.

## Philosophy

> "Every change has a blast radius. Know it before you commit."

`core-diff` provides **risk-aware change analysis** by understanding your package structure and their blast radius. Instead of just showing file diffs, it groups changes by impact level and warns about dangerous shell patterns.

## Core Features

### 1. Risk-Based Analysis
Changes grouped by package blast radius:
- 🔴 **CRITICAL** - System core, hooks (can break everything)
- 🟠 **HIGH** - Services, system config (major impact)
- 🔵 **MEDIUM** - Application config, scripts (moderate impact)
- 🟢 **LOW** - Docs, themes (minimal impact)

### 2. Shell Policy Enforcement
Scans shell scripts for dangerous patterns:
- `sudo` - Privilege escalation
- `curl | bash` - Remote execution
- `rm -rf /` - Destructive operations
- `chmod 777` - Insecure permissions
- And 6 more critical patterns

### 3. Multiple Analysis Modes
- **Working tree** - Uncommitted changes
- **Since ref** - Compare to any commit/tag
- **Summary** - Quick stats
- **Policy scan** - Shell authority analysis

## Usage

### Quick Morning Check
```bash
# See what changed since last commit
core-diff

# Output:
# 📊 Changes detected in 0-core:
#
# 🔴 CRITICAL (1 package(s)):
#    hooks (2 files)
#
# 🔵 MEDIUM (2 package(s)):
#    wm-sway (5 files)
#    shell-zsh (3 files)
```

### Review Since Release
```bash
# Compare to specific commit/tag
core-diff since v7.6.0
core-diff since HEAD~5

# Show only critical/high changes
core-diff since v7.6.0 --high-risk
```

### Package-Specific Analysis
```bash
# Deep dive on specific package
core-diff wm-sway

# Verbose mode (show individual files)
core-diff wm-sway --verbose
```

### Shell Policy Scanning
```bash
# Scan changed shell scripts
core-diff --policy shell

# Full system scan (all shell scripts)
core-diff --policy shell --all

# Output:
# 🛡️  Shell Authority Policy Analysis
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#
# File: install-hooks.sh (🔴 CRITICAL)
#   ❌ Pattern: sudo
#      Domain: Privilege Escalation | Severity: Critical
#   ❌ Pattern: systemctl
#      Domain: Service Management | Severity: High
```

### Visual Diff Tools
```bash
# Open in delta (terminal)
core-diff --open delta

# Focus on specific package with delta
core-diff wm-sway --open delta
```

### Quick Stats
```bash
# Just the numbers
core-diff summary

# Output:
# Packages: 5
# Files: 12
# Risk: HIGH
```

### Health Check
```bash
# Verify core-diff is working
core-diff --health

# Output:
# 🏥 core-diff health check
# ✅ git: available
# ✅ 0-core: /home/user/0-core exists
# ✅ git repo: valid
# ✅ Core checks passed!
```

## How It Works

### Risk Level Detection
1. Reads `.dotmeta` file from each package
2. Extracts `blast_radius` field
3. Groups changes accordingly
4. Falls back to smart defaults:
   - `docs/` → low
   - `theme-*` → low
   - `scripts/` → medium
   - `hooks/` → high
   - `system/` → high

### Shell Policy Analysis
Scans shell scripts for 10 forbidden patterns:

**Critical Patterns:**
- `sudo` - Privilege escalation
- `rm -rf /` - Destructive operations
- `curl | sh`, `curl | bash` - Remote execution
- `wget | sh` - Remote execution

**High Patterns:**
- `systemctl` - Service management
- `pacman -S`, `yay -S` - Package management
- `chmod 777` - Insecure permissions

**Medium Patterns:**
- `eval "$(...)` - Dynamic execution

### Why This Matters
Traditional `git diff` shows changed lines. `core-diff` shows **impact**:
```bash
# Traditional diff
$ git diff
# 200 lines of changes... what's critical?

# core-diff
$ core-diff
# 🔴 CRITICAL: hooks (2 files) ← FIX THIS FIRST
# 🔵 MEDIUM: shell-zsh (10 files) ← Review later
```

## Integration with Workflow

### Pre-Commit Hook
```bash
# In git pre-commit hook
core-diff --high-risk
if [ $? -eq 0 ]; then
  echo "⚠️  Critical/high changes detected - review carefully"
fi
```

### CI/CD Pipeline
```bash
# Get machine-readable output
core-diff summary
# Packages: 3
# Files: 7
# Risk: MEDIUM
```

### Morning Ritual
```bash
# Quick check before starting work
core-diff

# Deep dive on suspicious package
core-diff hooks --verbose

# Review shell policy violations
core-diff --policy shell --all
```

## Use Cases

### 1. Pre-Commit Review
```bash
# Before committing
core-diff --high-risk

# If critical changes
core-diff hooks --verbose
core-diff --open delta
```

### 2. Pull Request Review
```bash
# Compare feature branch to main
git checkout feature-branch
core-diff since main

# Focus on risky changes
core-diff since main --high-risk
```

### 3. Release Audit
```bash
# Review all changes since last release
core-diff since v7.6.0

# Shell policy compliance
core-diff --policy shell --all
```

### 4. Troubleshooting
```bash
# What changed recently?
core-diff since HEAD~10

# Focus on system packages
core-diff system --verbose
```

## Examples

### Example 1: Quick Check
```bash
$ core-diff

📊 Changes detected in 0-core:

🔵 MEDIUM (2 package(s)):
   wm-sway (3 files)
   shell-zsh (2 files)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Summary:
   Packages: 2
   Files: 5
   Risk: MEDIUM
```

### Example 2: High-Risk Filter
```bash
$ core-diff --high-risk

📊 Changes detected in 0-core:

✅ No critical or high-risk changes

🔵 3 medium/low-risk package(s) changed
```

### Example 3: Shell Policy Violation
```bash
$ core-diff --policy shell

🛡️  Shell Authority Policy Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

File: scripts/install-hooks.sh (🔴 CRITICAL)
  ❌ Pattern: sudo
     Domain: Privilege Escalation | Severity: Critical
  ❌ Pattern: systemctl
     Domain: Service Management | Severity: High

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Summary: 2 violations in 1 files

Recommendations:
  • Graduate shell scripts with authority violations to Rust
  • Use 'faelight' unified CLI instead of direct commands
  • Add shell-policy headers for temporary exceptions
```

## Demo for Linus
```bash
# Show health check
core-diff --health

# Show current changes with risk analysis
core-diff

# Show verbose output for specific package
core-diff wm-sway --verbose

# Shell policy analysis
core-diff --policy shell --all

# Compare since tag
core-diff since v7.6.0

# Open visual diff
core-diff --open delta
```

## Design Principles

1. **Risk-Aware** - Every package has a blast radius
2. **Package-Centric** - Group by package, not flat file list
3. **Policy Enforcement** - Catch dangerous shell patterns
4. **Integration-Ready** - Works with git, delta, CI/CD
5. **Human-Friendly** - Colors, icons, clear summaries

## Technical Details

- **Language:** Rust
- **Git Integration:** Native git commands
- **Risk Detection:** `.dotmeta` parsing + smart defaults
- **Shell Analysis:** Pattern matching (10 forbidden patterns)
- **Performance:** Instant (<100ms for typical repos)

## Why This Matters

Traditional tools show **what changed**. `core-diff` shows **what matters**:

- Git shows: "50 files changed"
- core-diff shows: "2 critical packages + 3 medium" ← **This is what you need to know**

It's the difference between noise and signal.

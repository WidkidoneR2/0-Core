# 🏥 Health Engine (dot-doctor)

The Health Engine validates system integrity across 14 comprehensive checks with dependency awareness.

**Version:** dot-doctor v0.5.0  
**System:** 0-Core v8.4.0  
**Checks:** 14 total  
**Target:** 100% health

---

## Quick Start
```bash
# Basic health check (14 checks)
dot-doctor
doctor                       # Alias

# Detailed explanations
doctor --explain

# Machine-readable output
doctor --json

# Show dependency graph
doctor --graph

# Run specific check only
doctor --check git

# CI mode (fail on warnings)
doctor --fail-on-warning

# Auto-fix safe issues
doctor --fix

# Show health history
doctor --history
```

---

## The 14 Health Checks

| ID | Name | Severity | Depends On | Description |
|----|------|----------|------------|-------------|
| `stow` | Stow Symlinks | 🔴 Critical | - | Verifies GNU Stow symlinks from 0-core/stow to ~/.config (12 packages) |
| `services` | System Services | 🟠 High | stow | Checks faelight-bar and faelight-notify running |
| `broken_symlinks` | Broken Symlinks | 🟡 Medium | stow | Scans for symlinks to non-existent targets |
| `yazi_plugins` | Yazi Plugins | 🟢 Low | stow | Verifies 4 Yazi plugins installed |
| `binaries` | Binary Dependencies | 🟠 High | - | Checks 15 required CLI tools installed |
| `git` | Git Repository | 🟡 Medium | - | Uncommitted changes, unpushed commits |
| `themes` | Theme Packages | 🟢 Low | stow | Verifies Faelight Forest theme present |
| `scripts` | Scripts | 🟠 High | - | All 34 core scripts present and executable |
| `dotmeta` | Package Metadata | 🟢 Low | - | .dotmeta status (intentionally removed in v8.4.0) |
| `intents` | Intent Ledger | 🟢 Low | - | Validates intent file format (28 intents) |
| `profiles` | Profile System | 🟡 Medium | scripts | Profile configuration valid |
| `config` | Faelight Config | 🟡 Medium | - | TOML config files valid (config.toml, profiles.toml, themes.toml) |
| `keybinds` | Sway Keybinds | 🟡 Medium | stow | No keybinding conflicts (116 unique bindings) |
| `security` | Security Hardening | 🟠 High | - | UFW, fail2ban, Mullvad VPN status |

---

## Severity Levels

| Level | Meaning | Action |
|-------|---------|--------|
| 🔴 **Critical** | System won't function | Must fix immediately |
| 🟠 **High** | Major features broken | Fix soon |
| 🟡 **Medium** | Reduced functionality | Should fix |
| 🟢 **Low** | Minor issues | Fix when convenient |

---

## Dependency Graph
```
stow (root) ← CRITICAL
├── services
├── broken_symlinks
├── yazi_plugins
├── themes
└── keybinds

scripts (root)
└── profiles

binaries (root)
git (root)
dotmeta (root)
intents (root)
config (root)
security (root)
```

**Dependency Rule:** If a parent check fails, dependent checks are blocked (not run).

---

## Check Details

### 1. Stow Symlinks (Critical)
**Purpose:** Verify all configuration packages are properly symlinked

**Checks:** 12 packages in `~/0-core/stow/`:
- wm-sway
- shell-zsh
- shell-nushell
- prompt-starship
- term-foot
- editor-nvim
- fm-yazi
- vcs-git
- config-faelight
- browser-qutebrowser
- browser-brave
- tools-topgrade

**Fix:**
```bash
cd ~/0-core/stow
stow -t ~ wm-sway shell-zsh term-foot editor-nvim fm-yazi vcs-git config-faelight
```

---

### 2. System Services (High)
**Purpose:** Ensure core services are running

**Checks:**
- faelight-bar (status bar)
- faelight-notify (notification daemon)

**Fix:**
```bash
# Restart services (Sway auto-starts them)
swaymsg reload

# Or manually
~/0-core/scripts/faelight-bar &
~/0-core/scripts/faelight-notify &
```

---

### 3. Broken Symlinks (Medium)
**Purpose:** Detect symlinks pointing to non-existent targets

**Scans:** `~/.config/`, `~/.local/`, `~/`

**Fix:**
```bash
# Find broken symlinks
find ~/.config -xtype l

# Remove if unneeded
rm <broken-symlink>

# Or re-stow package
cd ~/0-core/stow
stow -R <package>
```

---

### 4. Yazi Plugins (Low)
**Purpose:** Verify Yazi file manager plugins

**Required Plugins:**
- full-border.yazi
- git.yazi
- jump-to-char.yazi
- smart-enter.yazi

**Fix:**
```bash
cd ~/.config/yazi/plugins
git clone <plugin-url>
```

---

### 5. Binary Dependencies (High)
**Purpose:** Check required CLI tools are installed

**Required Binaries (15):**
- sway, foot, git, stow, etc.

**Fix:**
```bash
sudo pacman -S <missing-binary>
```

---

### 6. Git Repository (Medium)
**Purpose:** Ensure 0-core git state is clean

**Checks:**
- No uncommitted changes
- All commits pushed to origin

**Fix:**
```bash
cd ~/0-core
git status
git add .
git commit -m "fix: Description"
git push
```

---

### 7. Theme Packages (Low)
**Purpose:** Verify Faelight Forest theme exists

**Checks:** `config-faelight` package present

**Fix:**
```bash
cd ~/0-core/stow
stow -t ~ config-faelight
```

---

### 8. Scripts (High)
**Purpose:** All 34 core scripts exist and are executable

**Fix:**
```bash
# Rebuild workspace
cd ~/0-core
cargo build --release

# Copy binaries to scripts/
cp target/release/faelight-* scripts/
cp target/release/dot-doctor scripts/doctor
# ... etc
```

---

### 9. Package Metadata (Low)
**Purpose:** Track .dotmeta files

**Note:** As of v8.4.0, .dotmeta files are **intentionally removed** due to stow conflicts. This check passes with a note.

---

### 10. Intent Ledger (Low)
**Purpose:** Validate intent file format

**Checks:** 28 intents in `~/0-core/INTENT/`

**Fix:**
```bash
cd ~/0-core/INTENT
# Fix malformed intent files
```

---

### 11. Profile System (Medium)
**Purpose:** Profile configuration valid

**Checks:** `~/.config/faelight/profiles.toml`

**Fix:**
```bash
cd ~/0-core/stow/config-faelight/.config/faelight
nvim profiles.toml
# Fix TOML syntax errors
```

---

### 12. Faelight Config (Medium)
**Purpose:** TOML config files valid

**Checks:**
- config.toml
- profiles.toml
- themes.toml

**Fix:**
```bash
# Validate TOML
doctor --check config
# Fix syntax errors in ~/.config/faelight/
```

---

### 13. Sway Keybinds (Medium)
**Purpose:** Detect keybinding conflicts

**Checks:** 116 unique keybindings, no duplicates

**Fix:**
```bash
# Scan for conflicts
keyscan

# Edit Sway config
unlock-core
nvim ~/0-core/stow/wm-sway/.config/sway/config
lock-core
```

---

### 14. Security Hardening (High)
**Purpose:** Verify security protections active

**Checks:**
- UFW firewall active
- fail2ban protecting SSH
- Mullvad VPN connected

**Fix:**
```bash
# Enable UFW
sudo ufw enable

# Start fail2ban
sudo systemctl start fail2ban

# Connect VPN
mullvad connect
```

---

## Output Formats

### Default (Human-Readable)
```
🏥 0-Core Health Check - Faelight Forest v8.4.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Stow Symlinks: All 12/12 packages properly stowed
✅ System Services: All 2/2 services running
...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ System healthy! All checks passed! 🌲
Statistics:
   Passed:   14
   Warnings: 0
   Failed:   0
   Total:    14
   Health:   100%
```

### JSON Output
```json
{
  "version": "8.4.0",
  "total": 14,
  "passed": 14,
  "warnings": 0,
  "failed": 0,
  "blocked": 0,
  "health_percent": 100,
  "checks": [
    {
      "id": "stow",
      "name": "Stow Symlinks",
      "status": "Pass",
      "severity": "Critical",
      "message": "All 12/12 packages properly stowed",
      "details": ["✓ wm-sway", "✓ shell-zsh", ...]
    }
  ]
}
```

---

## Status Values

| Status | Icon | Meaning |
|--------|------|---------|
| **Pass** | ✅ | Check passed |
| **Warn** | ⚠️ | Non-critical issue |
| **Fail** | ❌ | Check failed |
| **Blocked** | 🚫 | Skipped due to parent failure |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All checks passed (100% health) |
| `1` | Warnings present |
| `2` | Failures present |

**CI Usage:**
```bash
# Fail build if any warnings
doctor --fail-on-warning || exit 1
```

---

## Integration

### With Git Hooks
```bash
# Pre-commit hook
doctor || exit 1
```

### With Update Workflow
```bash
# Check health before updates
faelight-update
# Runs `doctor` automatically
```

### With Automation
```bash
# Morning check
doctor && fg status && faelight-update --dry-run
```

---

## Philosophy

The Health Engine embodies:
- **Comprehensive** - 14 checks across all system dimensions
- **Dependency-Aware** - Failed parent blocks children
- **Actionable** - Every failure has a fix
- **Observable** - Clear status, detailed explanations
- **Automated** - Machine-readable JSON output

*The forest monitors itself.* 🌲🏥

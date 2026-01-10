# 🏥 Health Engine (dot-doctor)

The Health Engine validates system integrity across multiple dimensions with dependency awareness.

## Quick Start
```bash
# Basic health check
dot-doctor

# Detailed explanations
dot-doctor --explain

# Machine-readable output
dot-doctor --json

# Show dependency graph
dot-doctor --graph

# Run specific check only
dot-doctor --check git

# CI mode (fail on warnings)
dot-doctor --fail-on-warning
```

## Checks

| ID | Name | Severity | Depends On | Description |
|----|------|----------|------------|-------------|
| `stow` | Stow Symlinks | 🔴 Critical | - | Verifies GNU Stow symlinks from 0-core to ~/.config |
| `services` | System Services | 🟠 High | stow | Checks faelight-bar and faelight-notify running |
| `broken_symlinks` | Broken Symlinks | 🟡 Medium | stow | Scans for symlinks to non-existent targets |
| `yazi_plugins` | Yazi Plugins | 🟢 Low | stow | Verifies Yazi file manager plugins |
| `binaries` | Binary Dependencies | 🟠 High | - | Checks required CLI tools installed |
| `git` | Git Repository | 🟡 Medium | - | Uncommitted changes, unpushed commits |
| `themes` | Theme Packages | 🟢 Low | stow | Verifies config-faelight exists |
| `scripts` | Scripts | 🟠 High | - | Core scripts present and executable |
| `dotmeta` | Package Metadata | 🟢 Low | - | All packages have .dotmeta files |
| `intents` | Intent Ledger | 🟢 Low | - | Validates intent file format |
| `profiles` | Profile System | 🟡 Medium | scripts | Profile configuration valid |
| `config` | Faelight Config | 🟡 Medium | - | TOML config files valid |

## Severity Levels

| Level | Meaning | Action |
|-------|---------|--------|
| 🔴 Critical | System won't function | Must fix immediately |
| 🟠 High | Major features broken | Fix soon |
| 🟡 Medium | Reduced functionality | Should fix |
| 🟢 Low | Minor issues | Fix when convenient |

## Dependency Graph
```
stow (root)
├── services
├── broken_symlinks
├── yazi_plugins
└── themes

scripts (root)
└── profiles

binaries (root)
git (root)
dotmeta (root)
intents (root)
config (root)
```

If a parent check fails, dependent checks are **blocked** (not run).

## JSON Output Structure
```json
{
  "version": "6.3.0",
  "total": 12,
  "passed": 12,
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
      "message": "All 6/6 packages properly stowed",
      "details": ["✓ sway/config (wm-sway)", "..."]
    }
  ]
}
```

## Status Values

| Status | Icon | Meaning |
|--------|------|---------|
| Pass | ✅ | Check passed |
| Warn | ⚠️ | Non-critical issue |
| Fail | ❌ | Check failed |
| Blocked | 🚫 | Skipped due to parent failure |

## Fixing Common Issues

### Stow Symlinks Failed
```bash
cd ~/0-core
stow -R shell-zsh wm-sway term-foot launcher-fuzzel fm-yazi prompt-starship vcs-git
```

### Services Not Running
```bash
~/0-core/scripts/faelight-bar &
~/0-core/scripts/faelight-notify &
```

### Missing .dotmeta
```bash
cat > ~/0-core/<package>/.dotmeta << 'DOTMETA'
package: <name>
version: 1.0.0
description: <description>
blast_radius: low|medium|high
DOTMETA
```

---
_The forest monitors itself._ 🌲

# 0-Core Tools Reference

Quick reference for all 0-core tools and utilities.

---

## core-diff - Package-Aware Diff Tool

**Philosophy:** "Meld shows trees. core-diff shows the forest 🌲"

Package-level awareness, not file-level noise. Risk-based grouping for informed decisions.

### Quick Start

```bash
# Morning check
core-diff

# Since last release
core-diff since v3.3.5

# Inspect specific package
core-diff wm-sway --verbose

# Visual diff
core-diff --open meld
```

### Commands

**Modes:**

- `core-diff` - Show uncommitted changes (default)
- `core-diff since <ref>` - Compare to commit/tag/branch
- `core-diff working-tree` - Explicit uncommitted (same as default)
- `core-diff summary` - Stats only (packages, files, risk)
- `core-diff <package>` - Show specific package details

**Flags:**

- `--open delta` - Terminal diff with syntax highlighting
- `--open meld` - Visual diff in Meld GUI
- `--verbose, -v` - Show individual file lists
- `--high-risk` - Filter to critical/high packages only
- `--help, -h` - Show usage
- `--version` - Show version info

### Examples

```bash
# Daily workflows
core-diff                           # Quick morning check
core-diff --verbose                 # See what files changed
core-diff --high-risk               # Focus on important changes

# Release review
core-diff since v3.3.5              # Changes since release
core-diff since HEAD~5              # Last 5 commits

# Package inspection
core-diff wm-sway                   # Single package overview
core-diff wm-sway --verbose         # Show files
core-diff wm-sway --open meld       # Deep dive visually
core-diff wm-sway --open delta      # Terminal diff

# Quick stats
core-diff summary                   # Just numbers

# Combinations
core-diff since v3.3.5 --high-risk  # Critical changes since release
core-diff editor-nvim --verbose     # Detailed package view
```

### Understanding Output

**Risk Levels:**

- 🔴 **CRITICAL** - Window manager, system core (requires careful review)
- 🟠 **HIGH** - Shell, bar, core tools (significant impact)
- 🔵 **MEDIUM** - Editors, scripts, utilities (moderate impact)
- 🟢 **LOW** - Docs, themes, optional configs (minimal impact)

**Summary Stats:**

- **Packages** - Number of packages changed
- **Files** - Total files modified
- **Risk** - Highest risk level present

### Aliases

See shell configuration for shortcuts:

- `cdiff` - Short form
- `cds` - Quick summary
- `cdh` - High-risk only
- `cdv` - Verbose output
- `cdm` - Open Meld
- `cdd` - Open delta

---

## dot-doctor - Health Checker

Comprehensive system health validation.

### Usage

```bash
dot-doctor                          # Full health check
```

### Checks Performed

1. **Stow Symlinks** - All packages properly linked
2. **Yazi Plugins** - File manager plugins installed
3. **Broken Symlinks** - No dead links in managed dirs
4. **System Services** - Mako, Waybar running
5. **Binary Dependencies** - Required tools available
6. **Git Health** - Working tree clean, commits pushed
7. **Theme Packages** - All theme files present
8. **Scripts** - All scripts executable
9. **Config Aging** - File modification age tracking
10. **Intentional Defaults** - Naming discipline enforcement

### Output

- ✅ **100% Health** - All checks passing
- ⚠️ **90% Health** - Minor issues (e.g., uncommitted changes)
- ❌ **<90% Health** - Requires attention

---

## dotctl - Package Management

Manage 0-core packages.

```bash
dotctl status                       # Show all packages
dotctl list                         # List packages
```

---

## safe-update - Controlled Updates

Manual, snapshot-based system updates.

```bash
safe-update                         # Interactive update process
```

**Workflow:**

1. Creates Btrfs snapshot
2. Shows dry-run
3. Requires confirmation
4. Executes updates
5. Runs dot-doctor

---

## Scripts Reference

Located in `~/0-core/scripts/`:

- `bump-system-version` - Version bumping
- `core-diff` - Package-aware diffs
- `core-protect` - Immutability management
- `dot-doctor` - Health checking
- `dotctl` - Package control
- `safe-update` - Update orchestration
- `theme-switch` - Theme management

---

## Philosophy

All tools follow 0-Core principles:

- **Manual control** - No automation without consent
- **Intent over automation** - Explicit actions
- **Recovery over perfection** - Assume mistakes happen
- **Human comprehension** - Understandable output

See [PHILOSOPHY.md](PHILOSOPHY.md) for complete manifesto.

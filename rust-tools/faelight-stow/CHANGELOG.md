# Changelog - faelight-stow

All notable changes to this project will be documented in this file.

## [3.0.0] - 2026-02-11

### 🌟 LEGENDARY RELEASE - Complete Rewrite

**Major New Features - Tier 1:**

1. **Backup & Rollback System**
   - `--backup` flag creates timestamped backups
   - `rollback` command restores previous states
   - Backup directory: `~/.local/state/faelight-stow/backups`
   - Never lose your dotfile configurations

2. **Package Groups**
   - Predefined groups: dev, minimal, full
   - Stow multiple related packages at once
   - Example: `faelight-stow group dev`

3. **Dry-Run Preview**
   - `--dry-run` flag shows changes without applying
   - Safe testing before actual modifications
   - Works with all commands

**Major New Features - Tier 2:**

4. **Health Scoring**
   - `health` command shows 0-100% score
   - Identifies unstowed packages
   - Quick system overview

5. **Conflict Checking**
   - `check` command previews conflicts
   - Advanced detection (coming soon)

6. **Diff Mode (Partial)**
   - `diff` command structure implemented
   - Full implementation coming soon

**Architecture Changes:**
- ✅ Migrated to clap for CLI (from manual parsing)
- ✅ Subcommand-based interface
- ✅ Better error messages with hints
- ✅ Modular command structure

**Documentation:**
- 147-line comprehensive README
- Package group definitions
- Advanced usage examples
- CI/CD integration guide

**Code Quality:**
- Zero warnings (after fixes)
- 397 lines → 400+ lines
- Production-ready structure

### Commands
- `stow <packages>` - Stow one or more packages
- `check <packages>` - Check for conflicts
- `diff <package>` - Show differences (partial)
- `rollback [timestamp]` - Restore backup
- `group <name>` - Stow package group
- `verify` - Verify all packages (classic)
- `health` - Show health score

### Flags
- `--quiet` - Suppress output
- `--fix` - Auto-fix issues
- `--notify` - Desktop notifications
- `--dry-run` - Preview mode
- `--backup` - Create backup

---

## [2.2.0] - Earlier

Production dotfile manager with verification.

---

**Version Format:** MAJOR.MINOR.PATCH

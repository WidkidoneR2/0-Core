# bump-system-version v4.0.0

**Complete 0-Core Release Automation** - Philosophy-driven release management for Faelight Forest.

🌲 **Faelight Forest** - Automation serves the human

---

## 🎯 Purpose

Automates the complete release process while maintaining human control through interactive prompts. Handles version updates, CHANGELOG generation, intent tracking, and git operations.

## 🚀 Usage
```bash
# Full release (interactive)
bump-system-version 8.0.0

# Preview changes (dry-run)
bump-system-version --dry-run 8.0.0

# Show version
bump-system-version --version

# Show help
bump-system-version --help

# Health check
bump-system-version --health
```

## 📋 Release Process

### **Phase 1: Pre-Flight Checks** (Automated)
- ✅ System health (`dot-doctor`)
- ✅ Git status (clean working tree)
- ✅ Version format validation

### **Phase 2: Snapshot** (Automated)
- ✅ Creates btrfs snapshot for rollback safety

### **Phase 3: Version Updates** (Automated + Interactive)
- ✅ Updates `VERSION` file
- ✅ Updates `Cargo.toml` workspace version
- ✅ Updates `.zshrc` welcome message
- ✅ Updates README.md badges
- ❓ Prompts for milestone description
- ✅ Updates README.md milestone line

### **Phase 4: CHANGELOG** (Semi-Automated)
- ✅ Runs `compile-changelog.sh` to analyze commits
- ❓ Opens draft in `$EDITOR` for human polish
- ✅ Inserts polished changelog into `CHANGELOG.md`
- ❓ Prompts for release quote

### **Phase 5: Version Table** (Interactive)
- ❓ Prompts for brief description
- ✅ Adds row to README.md version history table

### **Phase 6: Intent Completion** (Smart Detection)
- ✅ Scans git commits for intent references
- ❓ Shows detected intents, asks to mark complete
- ✅ Updates intent status and checkboxes

### **Phase 7: Git Operations** (Interactive)
- ✅ Stages all changes
- ✅ Generates commit message
- ❓ Shows preview, asks to create commit
- ❓ Asks to create git tag
- ❓ Asks to push to origin

### **Phase 8: Verification** (Automated)
- ✅ Runs final health check
- ✅ Verifies VERSION file
- ✅ Shows success summary

## 🏗️ Architecture

**Technology:**
- Rust with `chrono` for date formatting
- Interactive stdin prompts for human decisions
- Shell command integration for git, doctor, snapper
- Integrates with `compile-changelog.sh`

**Philosophy:**
- **Manual Control** - Human confirms all major decisions
- **Automation Serves Human** - Tool handles tedious work
- **Safety First** - Snapshots, validation, rollback capability
- **Intent-Driven** - Tracks decision completion automatically

**Files Updated:**
- `VERSION` - System version number
- `Cargo.toml` - Workspace version
- `stow/shell-zsh/.zshrc` - Welcome message
- `README.md` - Badges, milestone, version table
- `CHANGELOG.md` - Release entry with commits
- `INTENT/future/*.md` - Intent status

## 📜 Philosophy

**"Automation serves the human, never replaces judgment."**

This tool embodies the 0-Core philosophy by:
- Automating tedious file updates
- Detecting completed work (intents in commits)
- Generating comprehensive changelogs
- **But always prompting for human decisions**

The interactive prompts ensure you maintain control while the tool eliminates the manual drudgery of release management.

## 🔧 Configuration

**Environment Variables:**
- `$EDITOR` - Editor for changelog editing (default: nvim)
- `$HOME` - User home directory

**Dependencies:**
- `dot-doctor` - System health checks
- `git` - Version control
- `snapper` - Btrfs snapshots
- `compile-changelog.sh` - Changelog generation
- `$EDITOR` - Text editor

## 🐛 Troubleshooting

**Pre-flight check fails:**
- Run `doctor` to see specific issues
- Fix health problems before releasing

**Git is not clean:**
- Commit or stash uncommitted changes
- Tool requires clean working tree for safety

**Snapshot creation fails:**
- Ensure snapper is configured for root subvolume
- Tool continues anyway (snapshot is optional safety)

**Changelog generation fails:**
- Ensure `compile-changelog.sh` exists in `scripts/`
- Falls back to template if script unavailable

## 🎓 Learning

This tool demonstrates:
- **Interactive CLI Design** - Balancing automation with human control
- **Defensive Programming** - Validation, error handling, rollback
- **Process Automation** - Multi-phase release workflow
- **Intent Integration** - Decision tracking at release level
- **Git Workflow** - Commits, tags, pushing

## 🔄 Version History

- **v4.0.0** (2026-01-21) - Linus Edition
  - Added `--help`, `--version`, `--health`, `--dry-run` flags
  - Integrated compile-changelog.sh for automatic CHANGELOG generation
  - Interactive prompts for milestone, quote, descriptions
  - Smart intent completion detection from git commits
  - Automated git commit/tag/push with human confirmation
  - Version history table updates
  - Beautiful phased progress output
  - Comprehensive README with philosophy

- **v3.0.0** (2026-01-19) - CHANGELOG automation
  - Auto-generates CHANGELOG template
  - Snapshot creation for rollback

- **v2.0.0** - Multi-file version updates
- **v1.0.0** - Initial version bumping

---

**Part of the Faelight Forest ecosystem** 🌲🦀

**"The forest evolves with intention."**

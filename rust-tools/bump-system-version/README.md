# bump-system-version v9.2.0

**Complete 0-Core Release Automation** - Philosophy-driven release management for Faelight Forest.

🌲 **Faelight Forest** - Automation serves the human

---

## 🎯 Purpose

Automates the complete release process while maintaining human control through interactive prompts. Handles version updates, CHANGELOG generation, and git operations with minimal friction.

## 🚀 Usage
```bash
# Full release (interactive)
bump-system-version 8.5.0

# Preview changes (dry-run)
bump-system-version --dry-run 8.5.0

# Show version
bump-system-version --version

# Show help
bump-system-version --help

# Health check
bump-system-version --health
```

## 🌿 Auto-Merge Workflow (NEW in v9.2.0!)

**Work on feature branches, release to main automatically!**

### How It Works

When you run `bump-system-version` on a **feature branch**:

1. ✅ Commits all changes to feature branch
2. ✅ Creates tag on feature branch  
3. ✅ Pushes feature branch
4. ✅ **Switches to main**
5. ✅ **Merges feature branch to main**
6. ✅ **Pushes main**
7. ✅ **Returns you to feature branch**

When you run on **main branch**:
- Works as before (commit → tag → push)

### Example
```bash
# You're on feature/awesome-tool
git branch
# * feature/awesome-tool
#   main

# Run bump
bump-system-version 9.5.0

# Output:
# 🌿 Feature branch detected: feature/awesome-tool
#    Will auto-merge to main after tagging
# 
# 4️⃣ Pushing feature branch...
# 5️⃣ Pushing tags...
# 6️⃣ Merging to main...
#    ✅ Switched to main
#    ✅ Merged feature/awesome-tool → main
# 7️⃣ Pushing main...
# 8️⃣ Returning to feature/awesome-tool...
#    ✅ Back on feature/awesome-tool

# You're back where you started!
git branch
# * feature/awesome-tool
#   main
```

### Benefits

- 🚀 **Zero manual work** - No switching branches manually
- ✅ **Main always updated** - Tags appear on GitHub immediately
- 🔄 **Stay in context** - Returns to your working branch
- 🎯 **Professional workflow** - Feature branches → main


## 📋 Release Process

### **Phase 1: Pre-Flight Checks** (Automated)
- ✅ System health (dot-doctor)
- ✅ Git status (clean working tree)
- ✅ Version format validation

### **Phase 2: Snapshot** (Automated)
- ✅ Creates btrfs snapshot for rollback safety
- 🔄 Prints rollback command for easy recovery

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
- ✅ **Auto-deletes CHANGELOG draft** (NEW in v5.1.0)
- ❓ Prompts for release quote

### **Phase 5: Version Table** (Interactive)
- ❓ Prompts for brief description
- ✅ Adds row to README.md version history table

### **Phase 6: Git Operations** (Interactive)
- ✅ Stages all changes
- ✅ Generates commit message with health, quote
- ❓ Shows preview, asks to create commit
- ❓ Asks to create git tag
- ❓ **Single push confirmation for commits AND tags** (NEW in v5.1.0)
- ✅ Pushes with `--follow-tags` in one operation

### **Phase 7: Verification** (Automated)
- ✅ Runs final health check
- ✅ Verifies VERSION file
- ✅ Shows success summary

## ✨ What's New in v5.1.0

**Reduced Friction, Less Stress:**

1. **Removed Intent Analysis Phase**
   - Intent tracking was adding unnecessary complexity
   - Streamlined workflow focuses on essential release tasks
   - 6 interactive phases instead of 7

2. **Auto-Delete CHANGELOG Drafts**
   - Draft files automatically cleaned up after insertion
   - No more manual `rm CHANGELOG-v*-DRAFT.md` needed
   - Keeps repository tidy automatically

3. **Single Push Confirmation**
   - One `git push --follow-tags` command instead of two
   - Only prompts for "push-main" once (not twice)
   - Clearer messaging: "Push commits and tags to origin?"

**Result:** Faster releases, fewer prompts, less manual cleanup.

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
- **Reduce Friction** - Minimize prompts while maintaining oversight

**Files Updated:**
- `VERSION` - System version number
- `Cargo.toml` - Workspace version
- `stow/shell-zsh/.zshrc` - Welcome message
- `README.md` - Badges, milestone, version table
- `CHANGELOG.md` - Release entry with commits

## 📜 Philosophy

**"Automation serves the human, never replaces judgment."**

This tool embodies the 0-Core philosophy by:
- Automating tedious file updates
- Generating comprehensive changelogs
- Creating safety snapshots before changes
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

**Push fails:**
- Check network connectivity
- Verify GitHub credentials
- Ensure you have push permissions

## 🎓 Learning

This tool demonstrates:
- **Interactive CLI Design** - Balancing automation with human control
- **Defensive Programming** - Validation, error handling, rollback
- **Process Automation** - Multi-phase release workflow
- **Git Workflow** - Commits, tags, pushing
- **Iterative Improvement** - v5.1.0 reduced friction based on real usage

## 🔄 Version History

- **v5.1.0** (2026-01-26) - UX Improvements
  - Removed Intent analysis phase (cleaner workflow)
  - Auto-delete CHANGELOG drafts after insertion
  - Single push confirmation with `--follow-tags`
  - Reduced prompts and manual cleanup
  - Focus on essential release tasks

- **v5.0.0** (2026-01-25) - Pre-flight Dashboard
  - Added comprehensive pre-flight dashboard
  - Enhanced safety checks and validation
  - Improved user experience with clear phase structure

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

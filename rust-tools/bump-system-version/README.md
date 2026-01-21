# bump-system-version v3.0.0

🌲 **Complete Release Automation** - Automated release process following Faelight Forest release standards.

## Philosophy

> "Releases should be rigorous, automated, and auditable."

`bump-system-version` automates the entire 0-Core release process: version updates, CHANGELOG generation from git commits, snapshot creation, and validation. It follows your documented release process exactly.

## What It Does

When you run `bump-system-version 8.0.0`, it:

1. ✅ **Pre-flight checks** - Verifies system health and git status
2. ✅ **Creates snapshot** - Snapper snapshot for rollback
3. ✅ **Updates versions** - VERSION, Cargo.toml, .zshrc, README.md
4. ✅ **Generates CHANGELOG** - Template with today's date
5. ✅ **Shows next steps** - Clear guide for completion

## Usage

### Basic Release
```bash
# Bump to new version
bump-system-version 8.0.0

# Output:
# 🌲 0-Core Release Process v7.6.5 → v8.0.0
#
# 📋 Step 1: Pre-flight Checks
#   ✅ System health: OK
#   ✅ Git status: clean
#
# 📸 Step 2: Creating Snapshot
#   ✅ Snapshot #42 created
#   🔄 Rollback: sudo snapper rollback 42
#
# 🔄 Step 3: Updating Version Numbers
#   ✅ VERSION file
#   ✅ Cargo.toml
#   ✅ stow/shell-zsh/.zshrc
#   ✅ README.md (4 updates)
#
# 📝 Step 4: Generating CHANGELOG.md Template
#   ✅ Template added to CHANGELOG.md
#
# ✅ Release v8.0.0 Prepared Successfully!
```

### Version Types
```bash
# Major release (breaking changes, major features)
bump-system-version 8.0.0

# Minor release (new features, new tools)
bump-system-version 7.7.0

# Patch release (bug fixes, polish)
bump-system-version 7.6.6
```

## Release Process

### Automated Steps (Done by tool)

1. ✅ Health check via `dot-doctor`
2. ✅ Verify git is clean
3. ✅ Create Snapper snapshot
4. ✅ Update `VERSION` file
5. ✅ Update `Cargo.toml` version
6. ✅ Update `.zshrc` welcome message
7. ✅ Update `README.md` badges and milestone line
8. ✅ Generate `CHANGELOG.md` template

### Manual Steps (You complete)

After running the tool:
```bash
# 1. Review changes
git diff

# 2. Edit CHANGELOG.md
# Fill in the template with actual features/fixes

# 3. Edit README.md
# Update milestone description

# 4. Mark intents complete (if any)
sed -i 's/status: planned/status: complete/' ~/0-core/INTENT/future/XXX.md

# 5. Verify everything
doctor

# 6. Commit
git add -A
git commit -m "🌲 Release v8.0.0 - Description

- Feature 1
- Feature 2
- Fix 1

Health: 100%"

# 7. Tag
git tag -a v8.0.0 -m "v8.0.0 - Description"

# 8. Push
git push && git push --tags
```

## Files Updated

### VERSION
```
8.0.0
```

### Cargo.toml
```toml
[package]
name = "0-core"
version = "8.0.0"  # Updated
```

### stow/shell-zsh/.zshrc
```bash
echo "🌲 Faelight Forest v8.0.0"  # Updated
```

### README.md
```markdown
# Faelight Forest v8.0.0 - Sway Edition  # Updated

![Version](https://img.shields.io/badge/Version-v8.0.0-brightgreen)  # Updated

**v8.0.0 Milestone:** Description here. 🌲🦀  # Updated line
```

### CHANGELOG.md
```markdown
# Changelog

## [8.0.0] - 2025-01-21
### 🚀 New Features
- [Feature description]

### 🔧 Fixes
- [Fix description]

### 📦 Tool Updates
- [tool-name] vX.Y - [changes]

> "Quote for this release" 🌲

---
```

## CHANGELOG Emoji Guide

Use these categories in CHANGELOG.md:

- 🚀 **New features** - Major functionality added
- 🔧 **Fixes** - Bug fixes and corrections
- 📦 **Tool updates** - Individual tool version bumps
- 🔒 **Security** - Security improvements
- 📐 **Typography/UI** - Visual improvements
- 🦀 **Rust improvements** - Code quality, performance
- 📜 **Documentation** - Docs and comments

## Example Release (v8.0.0 After Audit)
```bash
# After completing tool audit
bump-system-version 8.0.0

# Edit CHANGELOG.md:
## [8.0.0] - 2025-01-22
### 🚀 New Features
- Complete tool audit: 29/29 tools documented and v1.0+
- Intent-aware launcher (faelight-dmenu v2.0.0)
- Immutable core protection (core-protect v1.0.0)
- Risk-aware diff tool (core-diff v2.0.0)

### 📦 Tool Updates
- faelight v1.0.0 - System orchestrator
- faelight-dmenu v2.0.0 - Intent Ledger integration
- core-protect v1.0.0 - Kernel-level immutability
- core-diff v2.0.0 - Shell policy scanner
- profile v1.0.0 - Context-aware profiles
- intent v2.0.0 - 74 intents (73% success rate)
- intent-guard v1.0.0 - 11 safety patterns
- faelight-git v2.1.0 - Git Risk Score engine
- keyscan v1.0.0 - Comprehensive keybind analysis
- [... 20 more tools]

### 🦀 Rust Improvements
- Monorepo workspace architecture
- Consistent error handling across tools
- Comprehensive health checks for all tools

> "The audit is complete. Every tool documented, tested, and production-ready." 🌲

---

# Edit README.md milestone:
**v8.0.0 Milestone:** Complete tool audit and production hardening. All 29 tools documented with comprehensive READMEs, health checks, and v1.0+ status. Ready for Linus presentation. 🌲🦀

# Mark audit intent complete:
sed -i 's/status: in-progress/status: complete/' ~/0-core/INTENT/future/XXX-complete-audit.md

# Commit
git add -A
git commit -m "🌲 Release v8.0.0 - Complete Tool Audit

- 29/29 tools audited and documented
- All tools v1.0+ with READMEs and health checks
- Intent-aware launcher (revolutionary!)
- Immutable core protection
- Risk-aware diff analysis
- Complete release automation

Health: 100%
Intent Success Rate: 73%"

git tag -a v8.0.0 -m "v8.0.0 - Complete Tool Audit & Production Hardening"
git push && git push --tags
```

## Pre-flight Checks

The tool validates before making changes:
```bash
# 1. System health
dot-doctor  # Must pass

# 2. Git status
git status  # Must be clean (no uncommitted changes)
```

If checks fail, fix issues first:
```bash
# Health issues
doctor
# Fix reported issues

# Uncommitted changes
git add -A
git commit -m "Pre-release cleanup"

# Try again
bump-system-version 8.0.0
```

## Rollback

Every release creates a Snapper snapshot:
```bash
# If something goes wrong
sudo snapper rollback 42  # Use snapshot number from output

# Or manual rollback
git reset --hard HEAD~1
git tag -d v8.0.0
```

## Integration with Workflow

### After Major Feature
```bash
# Completed new feature
git add -A
git commit -m "Implement feature X"

# Ready for release
bump-system-version 7.7.0
# Edit CHANGELOG, commit, tag, push
```

### After Bug Fixes
```bash
# Fixed bugs
git add -A
git commit -m "Fix issues"

# Patch release
bump-system-version 7.6.6
# Edit CHANGELOG, commit, tag, push
```

### After Audit
```bash
# All tools audited and upgraded
bump-system-version 8.0.0
# Comprehensive CHANGELOG with all tool updates
```

## Error Handling

If the tool fails:
```bash
# Check what succeeded
git diff

# Revert if needed
git checkout VERSION Cargo.toml stow/shell-zsh/.zshrc README.md CHANGELOG.md

# Fix issue and retry
bump-system-version 8.0.0
```

## Design Principles

1. **Safety First** - Pre-flight checks before any changes
2. **Atomic Operations** - All-or-nothing updates
3. **Auditable** - Snapper snapshots for rollback
4. **Standardized** - Follows documented release process
5. **Clear Output** - Step-by-step progress reporting

## Technical Details

- **Language:** Rust
- **Version validation:** Regex `X.Y.Z` format
- **Snapshot:** Snapper integration
- **Health check:** Calls `dot-doctor`
- **Git check:** Verifies clean working tree

## Why This Exists

Manual releases are error-prone:
- Forget to update VERSION
- Miss updating .zshrc
- Inconsistent CHANGELOG format
- No snapshot safety net

`bump-system-version` ensures:
- Every release follows the same process
- Nothing is forgotten
- Rollback is always possible
- CHANGELOG is consistent

**This is release engineering rigor.**

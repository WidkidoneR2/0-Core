# 🌲 Faelight Forest Release Process

## Version Numbering

| Type | Example | When |
|------|---------|------|
| Major | 7.0.0 | Breaking changes, major features |
| Minor | 6.8.0 | New features, new tools |
| Patch | 6.7.1 | Bug fixes, polish, typography |

## Release Checklist

### 1. Pre-Release
```bash
# Verify health
dot-doctor

# Check for uncommitted changes
git status

# Run tests if applicable
faelight-stow
```

### 2. Bump Version
```bash
bump-system-version X.Y.Z
```

This updates:
- `VERSION` file
- `shell-zsh/.zshrc` (welcome message)
- `README.md` (badges, milestone)

### 3. Update CHANGELOG.md

Add entry at top (after header):
```markdown
## [X.Y.Z] - YYYY-MM-DD

### 🚀 New Features
- Feature description

### 🔧 Fixes
- Fix description

### 📦 Tool Updates
- tool-name vX.Y - changes

> "Quote for this release" 🌲

---
```

### 4. Update README.md

#### Milestone Line
```bash
sed -i 's/vX.Y.Z Milestone:.*/vX.Y.Z Milestone:** Description here. 🌲🦀/' ~/0-core/README.md
```

#### Version History Table
Add new row at top of table:
```markdown
| vX.Y.Z | YYYY-MM-DD | Brief description |
```

### 5. Mark Intents Complete (if applicable)
```bash
sed -i 's/status: planned/status: complete/' ~/0-core/INTENT/future/XXX-intent-name.md
```

Update success criteria checkboxes:
```bash
sed -i 's/- \[ \] Task/- [x] Task/' ~/0-core/INTENT/future/XXX-intent-name.md
```

### 6. Commit & Push
```bash
git add -A
git commit -m "🌲 Release vX.Y.Z - Brief Description

- Feature 1
- Feature 2
- Fix 1

Health: 100%"

git push
```

### 7. Verify Release
```bash
dot-doctor
cat ~/0-core/VERSION
```

## Quick Release Commands

### Patch Release (fixes only)
```bash
bump-system-version 6.7.2
# Update CHANGELOG.md manually
git add -A && git commit -m "🌲 Release v6.7.2 - Fixes" && git push
```

### Minor Release (new features)
```bash
bump-system-version 6.8.0
# Update CHANGELOG.md manually
# Update README milestone
# Mark intents complete
git add -A && git commit -m "🌲 Release v6.8.0 - Feature Name" && git push
```

## Tool Version Bumping

When updating individual tools, update the version in the source:
```bash
# In the tool's main.rs, update the comment:
sed -i 's/faelight-tool vX.Y/faelight-tool vX.Z/' src/main.rs

# Rebuild
cargo build --release
cp target/release/faelight-tool ~/0-core/scripts/
```

## Changelog Format

Use these emoji prefixes:
- 🚀 New features
- 🔧 Fixes
- 📦 Tool updates
- 🔒 Security
- 📐 Typography/UI
- 🦀 Rust improvements
- 📜 Documentation

---
_The forest grows with intention._ 🌲

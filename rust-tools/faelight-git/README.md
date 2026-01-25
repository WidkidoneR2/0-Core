# faelight-git v3.0.0 - Beautiful Git Workflow

> "Git should guide you, not confuse you. Every operation should be intentional." 🌲

**Transform chaotic git commands into a guided, beautiful experience.**

## ✨ Features

### 🎯 Core Commands

| Command | Purpose | Example |
|---------|---------|---------|
| `fg sync` | Full interactive workflow | Pull → Stage → Commit → Push |
| `fg quick "msg"` | Fast commit & push | One command, done! |
| `fg branch` | Branch management | Switch, create, delete |
| `fg log` | Beautiful commit history | Colored, interactive viewer |
| `fg status` | Risk-aware repository status | Know what changed |

### 🌲 What Makes It Special

**Interactive Workflow:**
- ✅ Pull latest changes
- ✅ Show what files changed (with colors!)
- ✅ Optional diff preview
- ✅ Confirm before staging
- ✅ Preview commit before creating
- ✅ Confirm before pushing

**Beautiful Output:**
- 🎨 Color-coded file changes (green +, yellow M, red D)
- 📊 Clear phase-based progress
- 🔍 Diff previews with stats
- 📜 Gorgeous commit history viewer
- ⚡ Fast and responsive

**Safety First:**
- ⚠️ Confirmation at every destructive step
- 🔒 Pre-commit and pre-push hooks
- 🛡️ Secret scanning with gitleaks
- 📝 Intent tracking integration
- 🔐 Core lock awareness

## 🚀 Quick Start
```bash
# Full interactive workflow
fg sync

# Quick commit
fg quick "Add awesome feature"

# Manage branches
fg branch

# View history
fg log
fg log -n 20  # Last 20 commits

# Check status
fg status
```

## 📸 Screenshots

**Sync Workflow:**
```
🌲 Faelight Git Sync v3.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📥 Phase 1: Pull Latest Changes
  🔄 Pulling from origin... ✅ Already up to date

📊 Phase 2: Repository Status
  📝 Modified files:
    M src/main.rs
    M README.md
  Summary: 2 modified, 0 added, 0 deleted

  ❓ Show diff preview? (y/n):
```

**Commit History:**
```
🌲 Faelight Git Commit History
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

71073fd 5 minutes ago christian Add branch and log commands
│
4e8d74e 30 minutes ago christian Security incident documentation
│
50d2c9d 40 minutes ago christian Enhanced sync with file preview
```

## 🎓 Philosophy

### Manual Control Over Automation
Every step requires confirmation. You see exactly what's happening before it happens.

### Intentionality
Guided commit messages, meaningful descriptions, no generic "fix" commits.

### Visual Clarity
- Color-coded diffs (green +, red -, yellow M)
- Unicode box drawing for structure
- Clear phase separation
- Human-readable output

## 🛠️ Technical Details

**Built with:**
- Rust 🦀
- clap (CLI framework)
- git2 (Git operations)
- colored (Beautiful terminal output)

**Part of 0-Core - Faelight Forest**
- Philosophy: Manual control over automation
- Understanding over convenience
- Intentionality in every operation

## 📊 Comparison

| Feature | git CLI | gitui | lazygit | faelight-git |
|---------|---------|-------|---------|--------------|
| Interactive workflow | ❌ | ✅ | ✅ | ✅✅ |
| Guided prompts | ❌ | Partial | Partial | ✅✅ |
| Confirmation steps | ❌ | ❌ | ❌ | ✅✅ |
| Philosophy-driven | ❌ | ❌ | ❌ | ✅✅ |
| Rust | N/A | ✅ | ❌ | ✅ |

## 🎯 Use Cases

**Perfect for:**
- Learning git workflows
- Teams that value clear commit history
- Projects requiring intentional commits
- Anyone tired of "oops, wrong commit" moments

**Example workflow:**
```bash
# Morning: Pull latest
fg sync  # Say 'n' to skip commit/push if clean

# Work on feature...

# Afternoon: Commit progress
fg sync  # Guided through entire workflow

# Quick fix
fg quick "Fix typo in README"

# Switch branches
fg branch  # Interactive menu

# Review recent work
fg log -n 20
```

## 🌲 "The forest stays in harmony."

**Built by Christian** • Part of 0-Core v8.2.0 • MIT License

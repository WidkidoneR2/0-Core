# 🌲 Faelight Git v3.2.0

**The only Git tool with Risk-Aware Intelligence™**

Intelligent git workflow management with real-time risk assessment. Know before you commit.

## ✨ The Risk Score Engine

### What Makes This Special

Every git operation calculates a **Risk Score (0-100)** based on:

| Factor | Points | Why It Matters |
|--------|--------|----------------|
| 🔴 Modified files | +10 | Uncommitted changes |
| 🟡 Untracked files | +5 | New files not in git |
| 🔴 No upstream | +5 | Can't push/pull |
| 🟡 Unpushed commits | +2 each | Work not backed up |
| 🟡 Behind upstream | +5 | Need to pull first |
| 🔴 System locked | +10 | 0-core protection active |

### Risk Bands

- **🟢 0-20: Safe** - Go ahead!
- **🟡 21-50: Caution** - Review first
- **🔴 51-100: Dangerous** - Create snapshot!

### See It In Action
```bash
# Check your risk before committing
fg risk

# Output:
# ⚠️  Git Risk Assessment
# Total Risk: 🟡 35/100
# Band: Caution
#
# Risk Factors:
#  +10 Dirty tree: 3 modified files
#  +10 Unpushed commits: 5 commits ahead
#  +15 No snapshot: Last commit 2 days ago
#
# 💡 Moderate Risk - Suggestions:
#   • Consider creating a snapshot
#   • Verify changes with: git diff --cached
```

## 🚀 Features

### Interactive Workflow (`fg sync`)

Full guided workflow with confirmation at each step:
1. **Pull** - Get latest changes
2. **Status** - See what changed (with risk score!)
3. **Diff** - Optional preview
4. **Stage** - Confirm files to add
5. **Commit** - Preview before creating
6. **Push** - Final confirmation

### Quick Commands
```bash
fg quick "fix: bug in parser"  # Instant commit + push
fg branch                      # Interactive branch management
fg log                         # Beautiful commit history
fg status                      # Risk-aware status
```

### Smart Protection

- 🎯 **Pre-push confirmation** for main/master branches
- 🔒 **Respects 0-core locks** (if in Faelight ecosystem)
- 📊 **Risk assessment** before every operation
- 🎨 **Color-coded output** for clarity

## 📦 Installation

### Standalone (Any Git Repository)
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/0-Core.git
cd 0-Core/rust-tools/faelight-git

# Build and install
cargo install --path .

# Verify
faelight-git --version
```

### Dependencies

- **Required:** `git`
- **Optional:** None! Works standalone

## 📚 Examples

### Daily Workflow
```bash
# Full interactive sync
fg sync
# Walks you through: pull → status → stage → commit → push

# Quick commit + push
fg quick "feat: Add dark mode"

# Check risk before committing
fg risk
```

### Branch Management
```bash
# List and switch branches
fg branch

# Create new branch
fg branch new-feature

# Delete branch  
fg branch -d old-feature
```

### Beautiful Logs
```bash
# Interactive commit viewer
fg log

# Last 20 commits
fg log -n 20
```

## 🎨 Commands

| Command | Description | Example |
|---------|-------------|---------|
| `status` | Risk-aware repo status | `fg status` |
| `risk` | Detailed risk breakdown | `fg risk` |
| `sync` | Interactive workflow | `fg sync` |
| `quick` | Fast commit + push | `fg quick "fix: typo"` |
| `branch` | Branch management | `fg branch` |
| `log` | Commit history viewer | `fg log -n 10` |
| `commit` | Guided commit | `fg commit` |
| `verify` | Check commit readiness | `fg verify` |

## 🛡️ Safety Features

### What Gets Checked

✅ No uncommitted changes before pull  
✅ Risk assessment before every operation  
✅ Confirmation before pushing to main/master  
✅ Working tree status validation  
✅ Upstream tracking verification  

### Protection Levels

- **Low Risk (🟢):** Operations proceed smoothly
- **Medium Risk (🟡):** Suggestions provided
- **High Risk (🔴):** Strong warnings + recommendations

## 🔧 Configuration

Works with standard git config:
```bash
# Set default branch protection
git config --global init.defaultBranch main

# Configure push behavior  
git config --global push.default current
```

## 📖 Philosophy

> "Every git operation should be intentional. The tool should guide, not confuse."

Principles:
- **Transparency** - Always show what's happening
- **Safety** - Risk awareness before action
- **Beauty** - Clean, colored output
- **Simplicity** - Complex operations made easy

## 🤝 Part of 0-Core

Faelight-git is part of the 0-Core ecosystem but works perfectly standalone!

## 📝 Changelog

See [GitHub releases](https://github.com/WidkidoneR2/0-Core/releases) for version history.

## 📄 License

Intentional Stewardship - Manual control over automation.

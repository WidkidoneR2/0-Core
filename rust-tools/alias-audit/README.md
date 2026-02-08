# 🔍 Alias-Audit v9.1.0

**Zone-aware alias management and verification for 0-Core**

Beautiful audit tool that ensures all tools have proper aliases and detects conflicts.

## ✨ Features

### Comprehensive Checks
- **Duplicate Detection** - Finds duplicate alias definitions
- **Tool Coverage** - Verifies all 37 tools have aliases
- **Conflict Detection** - Identifies naming conflicts
- **Zone Awareness** - Shows current directory zone

### Beautiful Output
- **Box Border Headers** - Professional UI matching faelight-fetch/dotctl
- **Zone Integration** - Current zone display with icon
- **Color-coded Results** - Clear visual feedback
- **Summary Stats** - Total alias count and status

## 📦 Installation
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/0-Core.git
cd 0-Core/rust-tools/alias-audit

# Build and install
cargo install --path .

# Verify
alias-audit
```

## 🚀 Usage

### Full Audit (Default)
```bash
alias-audit

# Output:
╭─────────────────────────────────────────────────╮
│ 🔍 Alias Audit - Full Check                    │
╰─────────────────────────────────────────────────╯

  Current Zone: 🦀 WORK

📋 Checking for duplicates...
✅ No duplicate aliases found!

📦 Checking tool coverage...
✅ All 37 tools have aliases!

╭─────────────────────────────────────────────────╮
│ 📊 Total aliases: 297                            │
│ ✅ Audit complete!                              │
╰─────────────────────────────────────────────────╯
```

### Check Duplicates Only
```bash
alias-audit duplicates
```

### Check Missing Tools
```bash
alias-audit missing
```

### Doctor Integration
```bash
alias-audit --doctor
# Output: ✅ Alias Coverage: All 37 tools have aliases (297 total)
```

### Show All Tools
```bash
alias-audit tools
```

## 🎯 What It Checks

### Expected Tools (37)
**Core Infrastructure (10):**
- dot-doctor, faelight-update, core-protect
- safe-update, dotctl, entropy-check
- intent-guard, faelight-stow, faelight-snapshot

**Desktop Environment (9):**
- faelight-fetch, faelight-bar, faelight-launcher
- faelight-dmenu, faelight-menu, faelight-notify
- faelight-lock, faelight-dashboard, faelight-term

**Development (14):**
- intent, archaeology-0-core, workspace-view
- faelight-git, faelight-hooks, recent-files
- profile, teach, faelight, keyscan
- faelight-zone, faelight-fm, faelight-link

**Version Management (4):**
- bump-system-version, faelight-bootstrap
- get-version, latest-update

## 🎨 Zone Integration

Shows current directory context:
- 🦀 WORK - Rust projects
- 🌲 CORE - System files
- 💻 CODE - Development
- 📚 DOCS - Documentation
- 🏠 HOME - Home directory

## 📊 Commands

| Command | Description |
|---------|-------------|
| `alias-audit` | Run full audit (default) |
| `alias-audit duplicates` | Check for duplicate aliases |
| `alias-audit missing` | Check for missing tool aliases |
| `alias-audit conflicts` | Check for alias conflicts |
| `alias-audit tools` | Show all tools with aliases |
| `alias-audit --doctor` | Output for dot-doctor integration |

## 💡 Integration

### Dot-Doctor
Used by dot-doctor to verify alias coverage:
```bash
dot-doctor
# Includes: ✅ Alias Coverage: All 37 tools have aliases
```

### CI/CD
```bash
# In your pipeline
alias-audit --doctor || exit 1
```

### Pre-commit Hook
```bash
# .git/hooks/pre-commit
alias-audit duplicates || exit 1
```

## 🔧 Technical Details

- **Lines of Code:** ~242
- **Dependencies:** faelight-core, faelight-zone, clap, colored
- **Alias Source:** `~/.config/zsh/aliases.zsh`
- **Expected Tools:** 37 (daemon excluded)

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## 🌲 Part of 0-Core

Alias-audit ensures all 0-Core tools have proper shell integration.

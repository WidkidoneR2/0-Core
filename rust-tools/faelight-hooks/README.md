# 🎣 Faelight Git Hooks v10.1.0

Production-ready Git hooks with intelligent safety checks for any repository.

## ✨ Features

### Automatic Security Checks
- 🔒 **Secret Detection** - Prevents committing API keys, tokens, passwords (gitleaks)
- 🔍 **Merge Conflict Detection** - Blocks commits with unresolved conflicts
- 🎯 **Pre-push Safety** - Interactive confirmation before pushing to main/master

### Code Quality
- 📝 **Conventional Commits** - Enforces commit message format
- ✅ **Commit Validation** - Ensures meaningful commit messages
- 🚫 **Work-in-progress Protection** - Warns on WIP commits

### Smart Workflow
- ⚡ **Fast** - Lightweight Rust implementation
- 🎨 **Colored Output** - Clear, beautiful terminal feedback
- 🔧 **Configurable** - Easy to customize for your workflow

## 📦 Installation

### Standalone (Any Git Repository)
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/0-Core.git
cd 0-Core/rust-tools/faelight-hooks

# Build and install
cargo install --path .

# Install hooks to global git config
faelight-hooks install --global

# Or install to specific repository
cd /path/to/your/repo
faelight-hooks install
```

### Dependencies
- **Required:** `git`
- **Optional:** `gitleaks` (for secret detection)

## 🚀 Usage

### After Installation

Hooks run automatically:
- **commit-msg** - Validates your commit messages
- **pre-push** - Confirms before pushing to main branches

### Manual Commands
```bash
# Install hooks globally (all repos)
faelight-hooks install --global

# Install to current repository
faelight-hooks install

# Check hook status
faelight-hooks status

# Uninstall hooks
faelight-hooks uninstall
```

## 📚 Examples

### Commit Message Validation
```bash
# ❌ This will be rejected:
git commit -m "fixed stuff"

# ✅ These are accepted:
git commit -m "feat: Add user authentication"
git commit -m "fix: Resolve memory leak in parser"
git commit -m "docs: Update installation guide"
```

### Secret Detection
```bash
# ❌ Blocked automatically:
git commit -m "Add API key" 
# (contains: API_KEY=sk-abc123...)

# ✅ No secrets detected:
git commit -m "Add configuration template"
# (uses: API_KEY=your-key-here)
```

### Pre-Push Protection
```bash
# When pushing to main/master:
git push origin main

# You'll see:
# ⚠️  Pushing directly to MAIN in my-repo
# Proceed? (type 'push-main'): _
```

## 🔧 Configuration

Hooks respect conventional commit types:
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation
- `style:` - Formatting
- `refactor:` - Code restructuring
- `test:` - Tests
- `chore:` - Maintenance

## 🛡️ What Gets Checked

### Pre-commit
- ✅ No secrets in staged files
- ✅ No unresolved merge conflicts
- ✅ Files exist and are valid

### Commit-msg  
- ✅ Follows conventional commit format
- ✅ Not empty or too short
- ✅ Meaningful description

### Pre-push
- ✅ Confirms pushes to main/master branches
- ✅ Shows repository context
- ✅ Requires explicit confirmation

## 📖 Hook Details

All hooks are written in Rust for:
- **Performance** - Sub-millisecond execution
- **Safety** - No silent failures
- **Clarity** - Beautiful colored output

## 🤝 Contributing

This is part of the 0-Core ecosystem but works standalone!

## 📝 Changelog

See [GitHub releases](https://github.com/WidkidoneR2/0-Core/releases) for version history.

## 📄 License

Intentional Stewardship - Built for manual control and complete ownership.

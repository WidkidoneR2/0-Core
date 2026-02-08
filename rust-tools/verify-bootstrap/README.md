# ✅ Verify-Bootstrap v1.0.0

**Installation verification - ensure 0-Core bootstrap completed successfully**

Validates that all installation steps completed correctly before you start using the system.

## 🎯 Problem Solved

After installing/bootstrapping 0-Core, how do you know it worked?

**Without verify-bootstrap:**
- Did stow deploy all packages?
- Are scripts in PATH?
- Are core tools installed?
- **You have to manually check everything!**

**With verify-bootstrap:**
- One command: `verify-bootstrap`
- Checks 6 critical areas
- Clear pass/fail status
- Exit code for automation

## ✨ Features

### Six Bootstrap Checks

1. **System Files** - Security configurations present
2. **Stow Packages** - Dotfiles deployed (checks 5 core packages)
3. **Scripts** - Core scripts available (dotctl, profile, bump-system-version)
4. **Binaries** - Key tools installed (dot-doctor, faelight-git, etc.)
5. **PATH** - Environment configured correctly
6. **Environment** - Variables set (EDITOR, VISUAL)

### Clear Reporting
- ✅ Green for passed checks
- ❌ Red for failed checks
- Overall completion percentage
- Exit code 0 = complete, 1 = incomplete

## 📦 Installation
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/0-Core.git
cd 0-Core/rust-tools/verify-bootstrap

# Build and install
cargo install --path .

# Verify
verify-bootstrap --version
```

### Dependencies

- None! Self-contained tool

## 🚀 Usage

### Basic Check
```bash
# Run verification
verify-bootstrap

# Output:
# ✅ Bootstrap Verification
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ✅ System Files: Present
# ✅ Stow Packages: 4 packages deployed
# ✅ Scripts: Core scripts present
# ✅ Binaries: 4 key tools installed
# ✅ PATH: Correctly configured
# ✅ Environment: Variables set
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ✅ Bootstrap Complete: 6/6 checks passed (100%)
```

### Incomplete Bootstrap
```bash
verify-bootstrap

# Output:
# ✅ Bootstrap Verification
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ❌ System Files: Missing
# ✅ Stow Packages: 4 packages deployed
# ❌ Scripts: Missing scripts
# ✅ Binaries: 4 key tools installed
# ✅ PATH: Correctly configured
# ✅ Environment: Variables set
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ⚠️  Bootstrap Incomplete: 4/6 checks passed (66%)
# 
# 💡 Run installation steps to complete bootstrap
```

### Examples

#### After Fresh Install
```bash
# Verify installation completed
verify-bootstrap
```

#### In CI/CD
```bash
#!/bin/bash
# Verify test environment setup
if ! verify-bootstrap; then
    echo "Bootstrap incomplete!"
    exit 1
fi
```

#### New Machine Setup
```bash
# 1. Clone 0-Core
git clone <repo>

# 2. Run bootstrap steps
./install.sh

# 3. Verify
verify-bootstrap
```

#### Troubleshooting
```bash
# Check what's missing
verify-bootstrap
# Fix failed checks
# Run again until 100%
```

## 📋 What Gets Checked

### System Files
Validates security configurations:
- `/etc/security/limits.conf`
- At least one sysctl configuration

### Stow Packages
Checks these packages deployed:
- `shell-zsh`
- `wm-sway`
- `editor-nvim`
- `terminal-foot`
- `fm-yazi`

Passes if at least 3 are present.

### Scripts
Verifies core scripts exist:
- `dotctl` - Dotfile control
- `profile` - Profile management
- `bump-system-version` - Version control

### Binaries
Checks key tools installed:
- `dot-doctor`
- `faelight-git`
- `faelight-update`
- `faelight-stow`
- `bin-doctor`

Passes if at least 3 are available.

### PATH
Ensures PATH includes:
- `~/.cargo/bin` (for Rust tools)
- `~/0-core/scripts` (for shell scripts)

### Environment
Checks variables set:
- `EDITOR` or `VISUAL`

## 💡 Use Cases

### New Installation
```bash
# After installation
verify-bootstrap || echo "Setup incomplete"
```

### Documentation
```bash
# Prove installation works
verify-bootstrap
# Screenshot for docs
```

### CI/CD
```bash
# Validate test environment
verify-bootstrap
```

### Troubleshooting
```bash
# Check what's broken
verify-bootstrap
# Fix issues
# Verify again
```

## 🎯 Design Philosophy

**Trust, but verify** - Don't assume installation worked, prove it.

**One command** - Simple verification of complex setup.

**Clear feedback** - Know exactly what's missing.

**Exit codes** - Perfect for automation.

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## 📄 License

Intentional Stewardship - Manual control over automation.

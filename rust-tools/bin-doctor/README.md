# 🔧 Bin-Doctor v1.0.0

**Binary manifest system - track and verify installed binaries**

Prevents binary/source drift by maintaining a manifest of installed Rust binaries with version, commit, and build metadata.

## 🎯 Problem Solved

**Without bin-doctor:**
- You edit `faelight-git/src/main.rs` (source now v3.3.0)
- Your installed binary is still v3.2.0
- **You have NO WAY to know they're out of sync!**

**With bin-doctor:**
- Tracks every `cargo install` with version + commit hash
- Detects drift: binary v3.2.0 ≠ source v3.3.0
- Reminds you to rebuild

## ✨ Features

### Binary Tracking
- Records version, git commit, build timestamp, rustc version
- Stores source path for easy rebuilding
- Manifest stored in `~/.cargo/.manifest.toml`

### Drift Detection
- Compares installed binary vs current source
- Checks both version AND commit hash
- Exit code 1 for automation/CI

### Commands
- `register <tool>` - Track a binary after install
- `check` - Verify all binaries (with `--quiet` for scripts)
- `list` - Show all tracked binaries
- `drift` - Show only drifted binaries

## 📦 Installation
```bash
# Clone repository
git clone https://github.com/WidkidoneR2/0-Core.git
cd 0-Core/rust-tools/bin-doctor

# Build and install
cargo install --path .

# Verify
bin-doctor --help
```

### Dependencies

- None! Self-contained tool

## 🚀 Usage

### Basic Workflow
```bash
# After installing a tool
cargo install --path rust-tools/faelight-git
bin-doctor register faelight-git

# Later, check for drift
bin-doctor check

# Output:
# 🔍 Checking 1 binaries...
# ✅ faelight-git
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ✅ All 1 binaries in sync!
```

### Drift Detected
```bash
# You edit source code and bump version
vim rust-tools/faelight-git/Cargo.toml  # 3.2.0 → 3.3.0

# Check detects drift
bin-doctor check

# Output:
# 🔍 Checking 1 binaries...
# ❌ faelight-git: DRIFT DETECTED
#    Binary:  v3.2.0 @ cc26462
#    Source:  v3.3.0 @ cc26462
#    💡 Rebuild: cargo install --path /path/to/source
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ⚠️  1/1 binaries drifted

# Rebuild
cargo install --path rust-tools/faelight-git
bin-doctor register faelight-git  # Update manifest
```

### Show All Tracked Binaries
```bash
bin-doctor list

# Output:
# 📦 Tracked Binaries (3):
#
# faelight-git
#   Version:  3.2.0
#   Commit:   cc26462
#   Built:    2026-02-08 16:43
#   Rustc:    1.93.0
#   Source:   /home/user/0-core/rust-tools/faelight-git
#   Binary:   /home/user/.cargo/bin/faelight-git
```

### Show Only Drifted
```bash
bin-doctor drift

# Output:
# ❌ faelight-git
#    Binary:  v3.2.0 @ cc26462
#    Source:  v3.3.0 @ cc26462
#    💡 Rebuild: cargo install --path /path/to/source
```

### Automation / CI
```bash
# Silent mode - exit codes only
bin-doctor check --quiet
echo $?  # 0 = all in sync, 1 = drift detected

# Pre-commit hook
#!/bin/bash
if ! bin-doctor check --quiet; then
    echo "⚠️  Binary drift detected - rebuild tools"
    bin-doctor drift
    exit 1
fi
```

## 📋 Manifest Format

Stored in `~/.cargo/.manifest.toml`:
```toml
[faelight-git]
version = "3.2.0"
commit = "cc26462"
built = "2026-02-08T16:43:40Z"
rustc = "1.93.0"
source = "/home/user/0-core/rust-tools/faelight-git"
binary = "/home/user/.cargo/bin/faelight-git"
```

## 💡 Use Cases

### Daily Development
```bash
# Check drift every morning
bin-doctor check
# Rebuild if needed
```

### Release Process
```bash
# Before release, ensure binaries match source
bin-doctor check --quiet || exit 1
```

### Team Coordination
```bash
# Share manifest to track team's installed versions
cat ~/.cargo/.manifest.toml
```

### CI/CD Pipeline
```bash
# Verify production binaries
bin-doctor check --quiet
```

## 🎯 Design Philosophy

**Trust, but verify** - Manual builds, automated verification.

**Source of truth** - Git commit hash ensures exact match.

**Non-intrusive** - Doesn't change your workflow, just tracks it.

**Exit codes for automation** - Perfect for scripts and CI.

## 🤝 Integration with dot-doctor

Bin-doctor integrates with the 0-Core health checking system. Future versions will add a check to `dot-doctor` to verify binary drift.

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## 📄 License

Intentional Stewardship - Manual control over automation.

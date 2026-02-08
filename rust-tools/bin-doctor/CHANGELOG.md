# Changelog - Bin-Doctor

## [1.0.0] - 2026-02-08

### 🎊 Initial Release - Binary Manifest System

#### Features
- **Binary Registration** - Track installed binaries with metadata
  - Version from Cargo.toml
  - Git commit hash
  - Build timestamp
  - Rustc version
  - Source path for easy rebuilding
  
- **Drift Detection** - Compare binary vs source
  - Version mismatch detection
  - Commit hash verification
  - Clear rebuild instructions
  
- **Multiple Commands**:
  - `register <tool>` - Track a binary after cargo install
  - `check` - Verify all binaries (--quiet for scripts)
  - `list` - Show all tracked binaries with full metadata
  - `drift` - Show only drifted binaries
  
- **Automation Ready**:
  - Exit code 0 = all in sync
  - Exit code 1 = drift detected
  - Quiet mode for scripts/CI

#### Manifest Storage
- Location: `~/.cargo/.manifest.toml`
- Format: TOML with per-tool sections
- Human-readable and editable

#### Testing
- ✅ Verified registration works
- ✅ Verified drift detection catches version changes
- ✅ Verified drift detection catches commit changes
- ✅ Verified exit codes
- ✅ Verified all commands work

#### Production Quality
- ✅ Zero clippy warnings
- ✅ Comprehensive README (196 lines)
- ✅ Standalone installation docs
- ✅ Examples for daily use, CI/CD, automation
- ✅ Design philosophy documented

---

**Why bin-doctor exists:**

Without it, you have NO way to know if your installed binaries match your source code. You could edit `faelight-git` source to v3.3.0, but still be running the old v3.2.0 binary without realizing it. Bin-doctor solves this gap.

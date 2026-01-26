# Build & Deploy Workflow

> **Philosophy:** Build from source. Understand your tools. Manual deployment.

This guide explains how to build, deploy, and update the 0-Core Rust toolchain.

---

## 🦀 Quick Reference
```bash
# Build everything
cd ~/0-core/rust-tools && cargo build --release --workspace

# Build single tool
cd ~/0-core/rust-tools/dot-doctor
cargo build --release

# Deploy to scripts/
cp target/release/dot-doctor ../../scripts/

# Verify
dot-doctor --version
```

---

## 🏗️ Your Current Workflow

### Updating a Tool (e.g., dot-doctor)
```bash
# 1. Navigate and edit
cd ~/0-core/rust-tools/dot-doctor
nvim src/main.rs

# 2. Build
cargo build --release

# 3. Deploy
cp target/release/dot-doctor ../../scripts/

# 4. Return and test
cd ~/0-core
dot-doctor

# 5. Commit (source only)
git add rust-tools/dot-doctor/src/main.rs
git commit -m "fix(dot-doctor): Fixed issue"

# 6. Version bump (when ready for release)
bump-system-version 8.4.0
```

---

## 📦 Build Process

### Build Entire Workspace
```bash
cd ~/0-core/rust-tools

# Release build (optimized)
cargo build --release --workspace

# All binaries → rust-tools/target/release/
```

**Build times:**
- First build: ~5-10 minutes (downloads dependencies)
- Incremental: ~30-60 seconds (only changed code)

### Build Single Tool
```bash
cd ~/0-core/rust-tools/dot-doctor

# Just rebuild this tool
cargo build --release

# Binary → target/release/dot-doctor
```

---

## 📤 Deployment

### Manual Deployment (Your Current Process)
```bash
# From tool directory
cd ~/0-core/rust-tools/dot-doctor
cargo build --release
cp target/release/dot-doctor ../../scripts/

# Verify
which dot-doctor  # Should show ~/0-core/scripts/dot-doctor
dot-doctor --version
```

### Automated Deployment (via bump-system-version)
```bash
# Rebuilds all tools and handles versioning
bump-system-version 8.4.0

# What it does:
# 1. Updates VERSION file
# 2. Generates CHANGELOG
# 3. Runs git add -A (gitignore handles scripts/)
# 4. Creates commit with release message
# 5. Creates git tag
```

---

## 🔄 Source-First Strategy (v8.3.0+)

### What Changed

**Before:**
- Binaries committed to git (~60MB repo)
- `git add scripts/` tracked compiled binaries

**After:**
- Binaries gitignored (~10MB repo)
- Only source code tracked
- Build locally, deploy to scripts/

### Why?

✅ Single-machine use = no deployment benefit  
✅ Smaller git repo and meaningful diffs  
✅ Aligns with "Understanding over convenience"  
✅ Forces building from source (learning)

---

## 🐛 Troubleshooting

### "Binary not found in PATH"
```bash
# Verify binary exists
ls -lh ~/0-core/scripts/dot-doctor

# Check PATH
echo $PATH | grep 0-core

# Re-deploy binary
cd ~/0-core/rust-tools/dot-doctor
cargo build --release
cp target/release/dot-doctor ../../scripts/

# Source shell config
source ~/.zshrc
```

### Build Fails
```bash
# Clean and rebuild
cd ~/0-core/rust-tools
cargo clean
cargo build --release --workspace
```

---

## 📊 Build Artifacts

### What Gets Generated
```
rust-tools/
└── target/
    ├── debug/              # Debug builds (unoptimized)
    ├── release/            # Production builds (500KB-1.9MB)
    └── .rustc_info.json    # Compiler cache
```

**Note:** `target/` is gitignored (build artifacts stay local)

---

## 🎯 Best Practices

### Development Flow

1. **Edit** → Make changes to source
2. **Build** → `cargo build --release`
3. **Deploy** → `cp target/release/tool ../../scripts/`
4. **Test** → Run the tool
5. **Commit** → Only source code (git ignores scripts/)

### Version Updates

Use `bump-system-version` for releases:
- Handles VERSION, CHANGELOG, git operations
- Automatically skips gitignored files
- Creates tags and pushes

---

**Made with 🌲🦀 by Christian**  
*"Compile with purpose. Deploy with intent."*

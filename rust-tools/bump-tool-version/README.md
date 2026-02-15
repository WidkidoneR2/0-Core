# 🔧 bump-tool-version v2.1.0

**Individual tool version management with auto-increment support**

Companion to `bump-system-version` - manages versions for individual Rust tools in the monorepo.

## ✨ Features

- ✅ Auto-increment support (--major, --minor, --patch)
- ✅ Manual version specification
- ✅ Beautiful pre-flight dashboard
- ✅ Handles workspace versions
- ✅ Updates Cargo.toml + README.md
- ✅ Creates tool-specific git tags
- ✅ Git commit automation

## 🚀 Usage
```bash
# Auto-increment patch (1.0.0 → 1.0.1)
bump-tool-version faelight-link --patch

# Auto-increment minor (1.0.0 → 1.1.0)
bump-tool-version faelight-fm --minor

# Auto-increment major (1.0.0 → 2.0.0)
bump-tool-version alias-audit --major

# Manual version
bump-tool-version faelight-link 2.5.0

# Skip confirmation
bump-tool-version faelight-link --patch --yes
```

## 📊 What It Does

1. **Reads current version** (from Cargo.toml or workspace)
2. **Calculates new version** (auto or manual)
3. **Shows pre-flight dashboard** (what will change)
4. **Updates files:**
   - `rust-tools/{tool}/Cargo.toml`
   - `rust-tools/{tool}/README.md` (if exists)
5. **Git operations:**
   - Commits changes
   - Creates tag: `{tool}-v{version}`

## 🎯 Pre-Flight Dashboard
```
╔══════════════════════════════════════════════════════════════╗
║           🌲 TOOL VERSION BUMP - PRE-FLIGHT 🌲              ║
╚══════════════════════════════════════════════════════════════╝

📦 Tool Information:
  Tool:     faelight-link
  Current:  v1.0.0
  Target:   v1.0.1
  Type:     Patch version (bug fixes)

📝 Files That Will Be Modified:
  • faelight-link/Cargo.toml
  • faelight-link/README.md (if exists)

🔧 Operations That Will Execute:
  1. Update Cargo.toml version
  2. Update README.md version badge (if exists)
  3. Git commit with bump message
  4. Create git tag: faelight-link-v1.0.1
```

## 💡 Increment Types

- **--patch**: Bug fixes (1.0.0 → 1.0.1)
- **--minor**: New features, backwards compatible (1.0.0 → 1.1.0)
- **--major**: Breaking changes (1.0.0 → 2.0.0)

## 🔄 Workflow Integration
```bash
# Make changes to faelight-link
cd rust-tools/faelight-link
# ... edit code ...

# Bump version and commit
cd ~/0-core
bump-tool-version faelight-link --minor

# Push with tag
git push && git push --tags
```

## 🌲 Companion Tools

- **bump-system-version**: Bump entire system version (v8.5.0 → v8.6.0)
- **bump-tool-version**: Bump individual tool (faelight-link v1.0.0 → v1.1.0)

---

**Part of the forest.** 🌲

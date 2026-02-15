# Changelog - bump-tool-version

All notable changes to bump-tool-version will be documented in this file.

## [2.1.0] - 2026-02-14

### 📋 Documentation

**Added:**
- ✅ **CHANGELOG.md** - Now tracking all changes
- ✅ **Version consistency** - README updated to match v2.1.0

### Technical
- Zero code changes - tool remains functionally identical
- Maintained zero clippy warnings
- Maintained zero `.unwrap()` calls (all using `anyhow::Result`)
- 100% backward compatible

---

## [2.0.0] - 2026-02-02

### 🎯 Production Release

**Features:**
- ✅ Auto-increment support (--major, --minor, --patch)
- ✅ Manual version specification
- ✅ Beautiful pre-flight dashboard with change preview
- ✅ Workspace version handling
- ✅ Automatic Cargo.toml updates
- ✅ Automatic README.md version sync
- ✅ Tool-specific git tag creation
- ✅ Git commit automation with conventional format
- ✅ Proper error handling with `anyhow`
- ✅ Beautiful colored output

**Command Options:**
- `--major` - Bump major version (1.0.0 → 2.0.0)
- `--minor` - Bump minor version (1.0.0 → 1.1.0)
- `--patch` - Bump patch version (1.0.0 → 1.0.1)
- Manual: `bump-tool-version <tool> <version>`

**Philosophy:**
- Joyful version management - stress-free updates! 🎉
- Beautiful dashboards showing exactly what will change
- Safe automation with clear previews
- Individual tool focus (companion to bump-system-version)

**Quality:**
- Zero clippy warnings from day one
- Zero `.unwrap()` calls - proper error handling throughout
- Clean separation of concerns
- Excellent user experience

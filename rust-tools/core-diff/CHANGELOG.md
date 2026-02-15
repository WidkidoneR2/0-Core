# Changelog - core-diff

All notable changes to core-diff will be documented in this file.

## [3.1.0] - 2026-02-14

### 🎯 Production Improvements

**Fixed:**
- ✅ **Version consistency** - Updated clap version from 2.0.0 to 3.1.0 to match Cargo.toml
- ✅ **Better error handling** - Replaced `.unwrap()` with `.unwrap_or()` for safer package name handling
- ✅ **CHANGELOG added** - Now tracking all changes

### Changed
- Package name display now shows "unknown" instead of panicking if package is None
- All version strings synchronized to 3.1.0

### Technical
- Maintained zero clippy warnings
- No breaking changes - all existing functionality preserved
- 100% backward compatible

---

## [3.0.0] - 2026-01-14

### Production Release

**Features:**
- ✅ Package-aware git diff analysis for 0-Core system
- ✅ Risk scoring (CRITICAL, HIGH, MEDIUM, LOW)
- ✅ Shell authority policy analysis
- ✅ Integration with delta and meld diff tools
- ✅ Package-specific filtering
- ✅ Comprehensive policy rules for security, system files, and scripts

**Command Options:**
- `--verbose, -v` - Detailed output
- `--high-risk` - Show only critical/high risk changes
- `--summary` - Summary view only
- `--open <TOOL>` - Open in delta or meld
- `--policy <MODE>` - Shell policy analysis (audit/strict)
- `--all` - Scan all packages
- `--package <NAME>` - Filter to specific package

**Philosophy:**
- Safety-first change analysis
- Policy-driven risk assessment
- Integration with existing tools

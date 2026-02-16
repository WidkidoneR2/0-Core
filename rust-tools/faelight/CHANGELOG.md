# Changelog - faelight

All notable changes to faelight will be documented in this file.

## [2.1.0] - 2026-02-15

### 🌲 LEGENDARY Edition Improvements

**Fixed:**
- ✅ **Bulletproof error handling** - Removed both `.unwrap()` calls
- ✅ **Version consistency** - README updated from 1.0.0 to 2.1.0
- ✅ **CHANGELOG added** - Now tracking all changes

### Changed
- Config serialization errors now show friendly error message instead of panic
- Tool path parsing defaults to "unknown" instead of panic
- Zero clippy warnings maintained

### Technical
- Zero `.unwrap()` calls - fully safe error handling
- 100% backward compatible CLI
- All 6 commands work flawlessly (health, profile, intent, core, launch, config)

### Philosophy
The main CLI should be LEGENDARY - stable, safe, and powerful! 🌲

---

## [2.0.0] - 2026-01-10

### 🌲 Unified CLI for Faelight Forest

**Features:**
- ✅ Unified interface for all Faelight operations
- ✅ System health checks (via dot-doctor)
- ✅ Profile management (switch, list, validate)
- ✅ Intent ledger integration
- ✅ Core protection (lock/unlock)
- ✅ Application launcher
- ✅ Configuration management
- ✅ JSON output support
- ✅ Dry-run mode

**Commands:**
- `faelight health` - System health check
- `faelight profile` - Profile management
- `faelight intent` - Intent ledger operations
- `faelight core` - Core protection
- `faelight launch` - Launch applications
- `faelight config` - Configuration management

**Philosophy:**
The Core Spine - One CLI to rule them all! 🌲

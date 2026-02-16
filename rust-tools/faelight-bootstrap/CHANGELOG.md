# Changelog - faelight-bootstrap

All notable changes to faelight-bootstrap will be documented in this file.

## [2.1.0] - 2026-02-15

### 🛡️ Bulletproof Bootstrap

**Fixed:**
- ✅ **Removed all 14 `.unwrap()` calls** - bootstrap cannot panic!
- ✅ **Version consistency** - README updated from 1.0.0 to 2.1.0
- ✅ **CHANGELOG added** - Now tracking all changes

### Changed
- All `flush()` errors are now silently ignored (graceful degradation)
- `read_line()` errors allow retry instead of panic
- Time calculation uses `unwrap_or_default()` instead of panic
- Zero clippy warnings maintained

### Technical
- Zero `.unwrap()` calls - fully bulletproof
- 100% backward compatible
- Bootstrap tool that never panics during setup

### Philosophy
A bootstrap tool must NEVER panic - it's the first thing users run! 🛡️

---

## [2.0.0] - 2026-01-09

### 🌲 One-Command 0-Core Setup

**Features:**
- ✅ One-command Arch Linux setup
- ✅ Path-resilient installation
- ✅ Interactive setup wizard
- ✅ Package verification
- ✅ Dependency checking
- ✅ Automated stow linking
- ✅ Profile initialization
- ✅ Health validation

**Philosophy:**
Automation serves installation, but human controls choices 🌲

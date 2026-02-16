# Changelog - intent-guard

All notable changes to intent-guard will be documented in this file.

## [2.1.0] - 2026-02-15

### 🎨 Major Improvements

**Refactored:**
- ✅ **Colored output** - Replaced all raw ANSI codes with `colored` crate
- ✅ **Bulletproof error handling** - Removed all 5 `.unwrap()` calls
- ✅ **Safe defaults** - IO errors default to "no" (safer than panic)

### Changed
- `RiskLevel::color()` → `RiskLevel::colored_label()` returns `ColoredString`
- All confirmation prompts return `false` on IO error instead of panicking
- Sort comparison uses `Ordering::Equal` fallback instead of unwrap

### Technical
- Zero clippy warnings maintained
- Zero `.unwrap()` calls - fully safe error handling
- Modern colored crate for all output styling
- 100% backward compatible CLI

### Philosophy
A safety guard should never panic! This release makes intent-guard **bulletproof** 🛡️

---

## [2.0.0] - 2026-01-13

### 🛡️ Command Safety Guard

**Features:**
- ✅ Pattern-based dangerous command detection
- ✅ Risk levels: CRITICAL, HIGH, MEDIUM, LOW
- ✅ Interactive confirmation prompts
- ✅ Shell integration support
- ✅ Pattern database with 200+ rules
- ✅ Health check system
- ✅ Test mode for safety validation

**Command Options:**
- `check-command <cmd>` - Check command safety (for shell integration)
- `test <cmd>` - Test command against patterns
- `list-patterns` - Show all safety patterns
- `--health` - Run health check
- `--version` - Show version

**Philosophy:**
Protecting your system one command at a time 🛡️

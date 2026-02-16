# Changelog - faelight-core

All notable changes to faelight-core will be documented in this file.

## [1.0.0] - 2026-02-16

### 🌲 Stable Foundation Release

**Graduated to v1.0.0** - Signaling production stability as the foundation for all 42 Faelight tools!

**Added:**
- ✅ **CHANGELOG.md** - Now tracking all changes

### Modules (1,104 lines)
- **canvas.rs** (224 lines) - Drawing primitives
- **error.rs** (20 lines) - Error handling with `thiserror`
- **glyph.rs** (76 lines) - Font caching (70%+ CPU reduction)
- **paths.rs** (408 lines) - Path management and resolution
- **theme.rs** (134 lines) - Consistent styling system
- **wayland.rs** (199 lines) - Layer-shell configuration

### Quality Metrics
- ✅ Zero clippy warnings
- ✅ Zero problematic unwraps (1 safe unwrap after guaranteed insert)
- ✅ Proper error handling with `FaelightError`
- ✅ Production-tested across 42 tools
- ✅ Minimal dependencies (fontdue, thiserror, wayland-client)

### Philosophy
**v1.0.0 signals:** This foundation is stable, battle-tested, and ready to support the entire Faelight Forest ecosystem! 🌲

---

## [0.1.0] - 2026-01-14

### 🌱 Initial Foundation

**Features:**
- ✅ Glyph caching system (fontdue-based)
- ✅ Canvas drawing primitives
- ✅ Theme system for consistent styling
- ✅ Wayland layer-shell helpers
- ✅ Error handling with custom types
- ✅ Path management utilities

**Architecture:**
- Shared library for all Faelight tools
- Modular design (6 core modules)
- Zero-cost abstractions where possible
- Production-ready from day one

**Philosophy:**
A solid foundation enables legendary tools! 🌲

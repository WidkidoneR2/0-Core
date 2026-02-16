# Changelog - faelight-daemon

All notable changes to faelight-daemon will be documented in this file.

## [2.1.0] - 2026-02-15

### 📚 Documentation Release

**Added:**
- ✅ **README.md** - Complete usage and architecture documentation
- ✅ **CHANGELOG.md** - Now tracking all changes

### Highlights
- Tool already had **zero clippy warnings**
- Tool already had **zero unwraps** - perfect error handling from day one!
- Async/await with Tokio runtime
- Clean shutdown handling
- Unix socket communication

### Technical
- No code changes - already production-grade
- 100% backward compatible
- Zero `.unwrap()` calls maintained
- Zero clippy warnings maintained

### Philosophy
This daemon was already perfect - we just documented its excellence! 🌲

---

## [2.0.0] - 2026-02-04

### 🌲 Background Daemon for Faelight Forest

**Features:**
- ✅ Async/await architecture with Tokio
- ✅ Unix socket communication
- ✅ Clean shutdown handling
- ✅ Health check endpoint
- ✅ Test client included
- ✅ Bulletproof error handling (zero unwraps)

**Architecture:**
- Minimal footprint (61 lines main, 172 lines daemon)
- Production-ready async runtime
- Efficient socket-based IPC

**Philosophy:**
Background services should be invisible, reliable, and bulletproof! 🛡️

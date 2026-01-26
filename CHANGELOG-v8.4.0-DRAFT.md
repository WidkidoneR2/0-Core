## [8.4.0] - 2026-01-26

### 🎣 Added - Git Hooks Management
- **faelight-hooks v1.0.0** - Comprehensive Rust-based git hooks manager
  - Pre-commit hook: Secret scanning (gitleaks), merge conflict detection
  - Pre-push hook: Main branch protection, uncommitted changes warning  
  - Commit-msg hook: Conventional commit format validation
  - CLI commands: `install`, `check`, `config`
  - Skip functionality for flexible workflows (`--skip secrets,conflicts`)
  - Replaces bash pre-commit hook with production-ready Rust tool

### 🏗️ Changed - Architecture
- **Source-first build strategy** - Git tracks only source code now
  - Added `scripts/` and `backups/` to .gitignore
  - Binaries built locally, not committed to git
  - Repository size reduced: 60MB → 10MB (~83% smaller)
  - Aligns with "Understanding over convenience" philosophy
  - See Intent 065 for rationale

### 📚 Documentation
- Added `docs/ARCHITECTURE.md` - Complete directory structure documentation
- Added `docs/BUILD.md` - Build and deployment workflow guide
- Added `hooks/README.md` - Migration from bash to faelight-hooks
- Both intents documented: 065 (source-first), 066 (faelight-hooks)

### 🔧 Fixed
- Removed duplicate push confirmation prompts (shell function + hook)
- Fixed git hook stdin reading using /dev/tty for proper user input
- Simplified git wrapper function (removed redundant push case)

### 🎯 Philosophy Alignment
This release deepens 0-Core's commitment to:
- **Manual control over automation** - Explicit builds, intentional workflows
- **Understanding over convenience** - Source-first enforces building from code
- **Intent over convention** - Complete documentation of WHY decisions were made

### 📊 Statistics
- New Rust tool: faelight-hooks (34th in ecosystem)
- Documentation files: +3 (ARCHITECTURE.md, BUILD.md, hooks/README.md)
- Intents documented: +2 (Total: 26 intents, 11 complete)
- System health: 100% maintained throughout development

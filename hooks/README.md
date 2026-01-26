# Git Hooks (Legacy)

**⚠️ DEPRECATED:** This directory contains the old bash-based git hooks.

## Migration to faelight-hooks

These hooks have been **replaced by `faelight-hooks`**, a comprehensive Rust-based git hook manager.

### Old System (Deprecated)
- `pre-commit` - Bash script running gitleaks

### New System (Active)
- **faelight-hooks** - Rust tool with multiple checks
  - Secret scanning (gitleaks)
  - Merge conflict detection  
  - Extensible check system
  - Install/config/check commands

## Usage

**Install hooks:**
```bash
faelight-hooks install
```

**Run checks manually:**
```bash
faelight-hooks check
```

**See Intent 066 for migration details.**

---

*The bash pre-commit script is kept here for reference only.*

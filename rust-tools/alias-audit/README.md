# 🔍 alias-audit v1.0.0

**Zsh alias health checker for Faelight Forest**

Ensures all 38 Rust tools have proper aliases and detects conflicts.

## ✨ Features

- ✅ Check for duplicate alias definitions
- ✅ Verify all tools have aliases (100% coverage)
- ✅ Detect excessive aliasing
- ✅ Beautiful colored output
- ✅ Doctor integration (`--doctor` flag)

## 🚀 Usage
```bash
# Full audit
alias-audit

# Check for duplicates only
alias-audit duplicates

# Check which tools lack aliases
alias-audit missing

# Show all tools with their aliases
alias-audit tools

# Output for doctor integration
alias-audit --doctor
```

## 📊 Example Output
```
🔍 ALIAS AUDIT - Full Check
════════════════════════════════════════════════════════════

📋 Checking for duplicates...
✅ No duplicate aliases found!

📦 Checking tool coverage...
✅ All 37 tools have aliases! (daemon excluded)

════════════════════════════════════════════════════════════
📊 Total aliases: 301
✅ Audit complete!
```

## 🎯 What It Checks

**38 Rust Tools:**
- Core Infrastructure (11 tools)
- Desktop Environment (9 tools)
- Development & Workflow (14 tools)
- Version Management (4 tools)

**Special Cases:**
- `faelight-core` - Library (N/A)
- `faelight-daemon` - Background service (N/A)
- `teach` - Binary name is the alias
- `faelight-bootstrap` - One-time use

## 🏗️ Built In

- **Development:** 30 minutes
- **Total:** Lightweight single-file tool (~200 lines)

---

**Part of the forest.** 🌲

---
id: 036
date: 2025-12-20
type: future
title: "Rust Hygiene - Code Quality Improvements"
status: complete
completed: 2026-01-14
tags: [rust, code-quality, health, v7.0]
---

## ✅ COMPLETE

All four Faelight tools now have:
- `--health-check` flag for diagnostics
- Better error handling where appropriate
- Simple CLI argument parsing
- Production-ready code quality

## Completed Tools

1. **faelight-lock v0.3**
   - Added clap for CLI parsing
   - Replaced .expect() with ? operator
   - Health check validates swaylock and font

2. **faelight-menu v0.5**
   - Health check validates Wayland, font, commands
   - Simple arg parsing (no deps needed)
   - Clean exit handling

3. **faelight-notify v0.6**
   - Health check validates D-Bus, font, Wayland
   - Mutex unwraps kept (acceptable in thread)
   - Arg parsing added

4. **faelight-bar v0.8**
   - Health check validates Wayland and font
   - Initialization expects kept (fail-fast is good)
   - Simple arg parsing

## Philosophy

**When to use .expect():**
- Initialization failures (fail fast)
- Unrecoverable errors
- Within threads (panic = clear failure)

**When to use ?:**
- Recoverable errors
- In Result-returning functions
- Where caller should handle error

**Health checks enable:**
- Proactive debugging
- System validation
- Integration with `doctor`
- Clear diagnostics

---

_"Error handling is understanding failure modes."_ 🌲

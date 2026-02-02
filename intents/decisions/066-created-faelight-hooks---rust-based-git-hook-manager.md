---
id: 066
date: 2026-01-25
type: decisions
title: "Created faelight-hooks - Rust-based git hook manager"
status: complete
tags: [git, hooks, rust, security, gitleaks, faelight-ecosystem]
---

Replaced bash pre-commit hook with comprehensive Rust tool:
- Secret scanning (gitleaks integration)
- Merge conflict detection
- Install/config/check commands
- Skip functionality for flexibility
- Colored terminal output

Part of the faelight ecosystem. First commit using the new hook!
Tool is production-ready and actively running.

---

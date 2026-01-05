---
id: 009
date: 2026-01-05
type: future
title: "0-Core Bootstrap - One-Command Setup (v5.2.0)"
status: planned
tags: [bootstrap, installation, rust, v5.2.0]
---

## Why

Currently: Clone repo, read docs, run stow manually.
Goal: One command to bootstrap a fresh Arch install.

## What

```bash
curl -fsSL https://0-core.dev/install | bash
```

Rust-based installer that:

- Validates system (Arch, dependencies)
- Clones 0-core
- Runs stow
- Configures shell
- Runs dot-doctor

## Dependencies

- v5.0.0+ (Rust tools must exist)
- Bash installer is fragile; Rust handles errors properly

```

---

*Part of the 0-Core Intent Ledger* 🌲
```

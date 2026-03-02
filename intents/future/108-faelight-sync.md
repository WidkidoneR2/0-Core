---
id: 108
date: 2026-03-02
type: future
title: "faelight-sync — Rust Dotfile Sync"
status: planned
tags: [sync, dotfiles, rust, multi-machine, rusty]
version: TBD
priority: low
---

## Vision

Faelight Forest on multiple machines, synchronized in Rust.
Not rsync. Not a shell script. A Rust-native sync daemon
that understands the forest's structure.

## Approach

- Declarative sync manifest: which files sync, which stay local
- Conflict resolution: intent-aware (forest state takes priority)
- Encryption at rest for sensitive configs
- Event emission: sync events in ledger
- Replace any remaining shell sync scripts

## Success Criteria

- [ ] Declarative sync manifest
- [ ] Conflict detection and resolution
- [ ] Event ledger integration
- [ ] Encrypted sensitive config sync
- [ ] Zero shell script dependencies

---

*"The forest grows on every machine."* 🌲

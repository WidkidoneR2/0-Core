---
id: 037
date: 2026-06-05
type: documentation
title: "rust-tools documentation: README and CHANGELOG for all 38 tools"
status: planned
tags: [documentation, rust-tools, readme, changelog, release, conference]
priority: medium
---

## Why

38 rust-tools each have a README.md and CHANGELOG.md that reference Arch-era
facts, old version numbers, and outdated deployment instructions.

These need to be updated before:
- Faelight NixOS 1.0.0 public release
- First convention presentation (USENIX HotOS / WCRE, end 2026 / early 2027)
- Meeting with Armijn Hemel (second meeting, end of summer / beginning of fall 2026)

Graydon Hoare and Brian Fox are watching the project. The documentation
should reflect the quality of the work.

## Scope

Each tool needs:
- Version number updated to current
- System references: Arch Linux → NixOS 26.05
- Deployment instructions: stow → home-manager / nix rebuild
- Status: active / retired / planned clearly stated
- Last verified date updated

## Tools (38 total)

See registry/tools.toml for full list.
Priority order: engine, intent, faelight-shell, faelight-fm, faelight-bar,
then remaining tools alphabetically.

## Additional Docs

- docs/RELEASE.md -- depends on INT-031 (faelight-release v2)
- README.md (root) -- full rewrite at 1.0.0
- meta/CHANGELOG.md -- faelight-release v2 will manage this

## Dependencies

- INT-031 (faelight-release v2) -- release tooling will automate CHANGELOG updates
- Faelight NixOS 1.0.0 milestone

## Gate

- [ ] All 38 rust-tools README.md updated to NixOS era
- [ ] All 38 rust-tools CHANGELOG.md have NixOS migration entry
- [ ] docs/RELEASE.md updated post INT-031
- [ ] Root README.md rewritten for 1.0.0 public release
- [ ] meta/CHANGELOG.md has full NixOS migration section

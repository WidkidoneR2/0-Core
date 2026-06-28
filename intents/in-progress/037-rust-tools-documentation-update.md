---
id: 037
date: 2026-06-05
type: documentation
title: "rust-tools documentation: README and CHANGELOG for all 38 tools"
status: in-progress
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


## Concrete deliverable (2026-06-28): faelight-docs README generator + index

Rather than hand-write 38+ READMEs (none currently exist on disk), EXTEND the existing
faelight-docs ("living documentation engine") with a generator -- self-maintaining, never
drifts stale (which is the root problem this intent describes).

New `faelight-docs readme-tools` subcommand:
- Walks rust-tools/*/Cargo.toml (ground truth) for name/version/license/description/deps,
  extracting INT-NNN intent links from descriptions.
- Enriches from registry/tools.toml: category, status, expected_usage, depends_on.
- Skips retired=true tools (or minimal RETIRED stub).
- Emits RICH per-tool README.md: title, version/license/status, description, intent link,
  NixOS-era install (nix develop + deploy -- NOT Arch/stow), dependencies, category,
  last-verified date, Faelight Forest footer.
- Emits top-level rust-tools/README.md INDEX: all active tools by category (the
  conference/Armijn-ready "whole forest at a glance" catalog).
- Re-runnable: docs regenerate from ground-truth metadata, never go stale again.

BOUNDARY RESPECTED: this generates per-tool READMEs in rust-tools/*/ ONLY. It does NOT
touch the root README (faelight-release owns lines 1-37, faelight-docs owns 38+ -- that
boundary is untouched).

BONUS FIX: faelight-docs has stale `00-meta/` paths (registry.rs, main.rs:133) -- the
dir is now `meta/`. Corrected as part of this work (fitting: the docs tool itself had
outdated facts).

Tonight's bounded scope: build the subcommand + index, prove on priority tools (fsh, fm,
git, notify) + full index. Running across all ~35 active tools + hand-polish stays under
this intent's umbrella (along with the 1.0.0 root rewrite + meta/CHANGELOG sections).

### Sub-gates (this deliverable)
- [ ] `faelight-docs readme-tools` walks Cargo.toml + registry, skips retired
- [ ] Rich per-tool README generated (proven on fsh, fm, git, notify)
- [ ] Top-level rust-tools/README.md index generated (all active tools by category)
- [ ] Stale 00-meta/ -> meta/ paths fixed in faelight-docs
- [ ] Re-runnable + deployed

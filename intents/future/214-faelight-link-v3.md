---
id: 214
date: 2026-04-09
type: planned
title: "faelight-link v3 — Forest-Aware Dotfile Intelligence"
status: in-progress
tags: [faelight-link, dotfiles, stow, intelligence, forest-aware, v3]
---
faelight-link v2 is essentially a custom GNU Stow.
It links dotfile packages from 03-interfaces/stow/ into the filesystem.
It works. But it does not understand the forest.
faelight-link v3 is not a linking tool.
It is a dotfile intelligence layer.
It knows what every symlink is for, why it exists, and what breaks if it moves.
Every stow package should be traceable to an intent.
  faelight-link audit → "compositor-niri: INT-136 (Niri visual identity)"
  faelight-link audit → "shell-fsh: INT-179 (fsh daily driver)"
If a package has no intent — flag it. Orphaned config is a liability.
Current: stow fails with a conflict error.
v3: explains the conflict in forest terms.
  "shell-zsh conflicts with shell-fsh on ~/.zshrc — both claim this file.
   Resolution: shell-fsh is active daily driver. Remove shell-zsh package."
faelight-link runs as part of doctor.
Currently doctor checks "all 13/13 packages stowed."
v3: doctor shows per-package health.
  ✅ compositor-niri   — 47 symlinks, all valid
  ✅ shell-fsh         — 12 symlinks, all valid
  ⚠️  shell-zsh        — 3 orphaned symlinks (source removed)
Friday will need to understand your dotfile configuration.
faelight-link v3 emits signals when packages change:
  engine_signals: source=faelight-link, type=package_linked
  engine_signals: source=faelight-link, type=symlink_broken
Friday learns: "shell config changes correlate with session disruptions."
Each stow package gets a metadata file:
  .faelight-meta:
    intent: INT-136
    description: Niri compositor configuration
    critical: true
    last_linked: 2026-04-08
This replaces the .dotmeta approach that was removed (stow conflict resolution).
Stored outside the package directory — no stow conflicts.
Before stow link: snapshot current symlink state.
After failed link: restore previous state automatically.
  faelight-link rollback <package>
No more manual cleanup after a stow conflict.
When INT-180 is done: faelight-link removes the compositor-sway package cleanly.
v3 makes this safe: it checks for dependents before removing.
  "compositor-sway has no active dependents — safe to remove."
  faelight-link diff   — show what changed since last link
  faelight-link status — show all packages and their health
  faelight-link verify — verify all symlinks point to valid sources
  faelight-link status          — all packages, symlink counts, health
  faelight-link audit           — per-package intent traceability
  faelight-link diff            — what changed since last operation
  faelight-link verify          — validate all symlinks
  faelight-link rollback <pkg>  — restore previous symlink state
  faelight-link why <file>      — which package owns this file and why
⬜ faelight-link status — per-package health with symlink counts
⬜ faelight-link audit — intent traceability per package
⬜ faelight-link diff — changed symlinks since last operation
⬜ faelight-link verify — validates all 13 packages
⬜ faelight-link why <file> — ownership and reason
⬜ faelight-link rollback — pre/post snapshot with restore
⬜ Conflict detection with forest-aware explanations
⬜ engine_signals emission on link/unlink
⬜ doctor integration — per-package health in d output
⬜ .faelight-meta format defined and populated for all packages
⬜ INT-180 Sway removal uses faelight-link safely
"GNU Stow links files.
faelight-link understands them.
Every symlink has a reason.
Every package has an intent.
Every change has a record.
This is not configuration management.
This is the forest knowing where its roots are." 🌲

---
id: 117
date: 2026-07-02
type: future
title: "Friday Arch-language cleanup: de-Arch knowledge facts, suggestion strings, teach descriptions"
status: planned
tags: [friday, de-arch, cleanup]
---

## Why
INT-116 (final Arch sweep) removed all EXECUTABLE Arch code but deliberately DEFERRED
Friday's Arch *language* -- knowledge facts, command-recognition, suggestion strings,
teach descriptions. That is awareness-layer, not executable dependence, and cleaning it
is its own scoped effort (folding it into 116 would have ballooned that intent).
This intent handles the language.

## The distinction to make (per-item judgment, not blanket removal)
Some Arch mentions are LEGITIMATE AWARENESS -- Friday knowing pacman exists is fine if a
user with Arch muscle memory types it. Others are STALE -- teaching Arch as if it were
the current system. Decide per item: reframe to NixOS, keep as cross-distro awareness,
or remove.

## Targets (from the 2026-07-03 sweep, all non-executable language)
- engine/domains/friday/mod.rs:470,521 -- pacman/arch in command-recognition lists.
- engine/domains/friday/mod.rs:958-963 -- ("arch", "pacman -Syu ...") KNOWLEDGE facts
  (3 entries: pacman basics, AUR/paru/yay, /etc/pacman.conf/reflector). Decide: keep as
  cross-distro knowledge, reframe toward nix (nixos-rebuild / nix profile / flake), or
  remove. Friday teaching Arch package-management as if it's THIS system is the stale part.
- rust-tools/faelight-shell/src/commands/mod.rs:6817,6879 -- pacman in known-command
  parsing lists (recognition; low harm, but nix commands should be the first-class ones).
- rust-tools/faelight-shell/src/commands/mod.rs:8087 + exec.rs:412 -- "paru|pacman =>
  Suggestion: run d after update" strings. Reframe to the nix update path (update / fu).
- rust-tools/faelight-shell/src/config.rs:104 -- commented-out `// if starts_with "paru"`
  example. Remove or update the example.
- nix/home/dotfiles/faelight-shell/.config/faelight-shell/config.fsh:78 -- warns on
  `paru -Syu`. Reframe to the nix update command.
- rust-tools/teach/src/main.rs:258,262 -- "safe pacman checker" description +
  "--only pacman,cargo" example. Update to nix wording.

## LEAVE (correct, not targets)
- Migration history comments (toolgen.rs "Migrated from Arch Linux to NixOS", the
  INT-074/116 de-arch comments) -- correct records.
- friday_arch domain name -- "arch" = ARCHITECTURE, not Arch Linux.

## Approach
Per-tool, build-gated (same discipline as INT-116). Mostly string edits + a few
knowledge-entry decisions. Re-run the INT-116 sweep grep at the end; the only remaining
hits should be history + "arch"=architecture.

## Gates
- [ ] friday/mod.rs knowledge facts decided (reframe/keep/remove) + applied
- [ ] fsh suggestion strings reframed to nix update path
- [ ] teach descriptions updated to nix wording
- [ ] config.fsh paru warning reframed
- [ ] full workspace builds clean, zero warnings
- [ ] sweep grep: only history + architecture-"arch" remain

## Relationship
Follows INT-116 (executable Arch removed). This completes the de-Arch at the language
layer. A natural warm-up before INT-118 (Friday engine resumption).

---
id: 116
date: 2026-07-02
type: future
title: "Final Arch sweep: retire safe-update, de-Arch fsh pkg command, purge pacman/AUR remnants for true NixOS-native 1.0.0"
status: in-progress
tags: [de-arch, nixos, cleanup, 1.0.0.]
---

## Why
INT-082 shed the Arch-era LOCK model from faelight-git, but a whole-repo sweep
(2026-07-03) found LIVE Arch package-management code still present. For a genuinely
de-Arched, NixOS-native 1.0.0, these must go. "De-Arched" must be demonstrated by a
clean sweep, not declared.

## Evidence (grep sweep, live offenders only -- history/knowledge excluded)
LIVE Arch code to remove/fix:
- safe-update: git-clones aur.archlinux.org/paru.git, runs makepkg, rebuilds paru
  (main.rs:551-569). Pure Arch/AUR builder. SUPERSEDED by faelight-update. -> RETIRE.
- faelight-shell pkg command (commands/mod.rs:5532-5676): wraps paru/pacman for
  install/remove/update/search. -> RETIRE (or redirect to nix/faelight-nix).
- engine update/mod.rs:80-96: pacman -Qu fallback for update checks. -> REMOVE.
- engine doctor/entropy.rs:152,242: pacman -Q package queries. -> REMOVE/NixOS-ify.
- intent-guard check_pacman_remove (main.rs:82,202-204): guards pacman commands.
  -> REMOVE (pacman never runs on NixOS).
- Identity strings "Arch Linux (vanilla)" / "Install Arch Linux": bootstrap/mod.rs:35,
  doctor/mod.rs:171, narrative/mod.rs:41, faelight-context/main.rs:384, teach:445.
  -> "NixOS 26.05 (Yarara)".
- faelight-docs main.rs:343,650: reads "03-interfaces/stow/shell-zsh/.zshrc" (Arch-era
  stow path, zsh retired). -> REMOVE dead path.
- faelight-update pip_checker.rs:37,60: "on Arch use pacman" strings. -> NixOS wording.
- registry/aliases.toml:70-86: pacman/paru/yay package aliases. -> REMOVE.
- doctor/checks.rs:139,586: "sudo pacman -S/-Syu" fix hints. -> nix wording.
- security/mod.rs:161: "sudo pacman -Syu" patch hint. -> nix wording.
- knowledge/mod.rs:299-303: arch_pacman_conflict knowledge entry (Arch-specific fix).
  -> review: remove or reframe.
- paths.rs:238: hardcoded /usr/share/fonts. -> route or note (minor).

LEAVE ALONE (not Arch dependence):
- Friday knowledge base pacman/arch FACTS (friday/mod.rs:958-963) -- an assistant
  knowing pacman exists is awareness, not dependence. Keep.
- Frozen history: runtime/checkpoints/, meta/releases/, CHANGELOG.
- faelight_arch domain name (friday_arch) -- "arch" = architecture, not Arch Linux.

## Approach (retirements first, then string/logic fixes)
1. Retire safe-update wholesale (crate + registry + aliases) -- like get-version/profile.
2. Retire/redirect fsh pkg command.
3. Remove pacman fallbacks/queries (update, entropy, intent-guard).
4. Fix identity strings + fix-hints to NixOS wording.
5. Remove dead stow/.zshrc paths in faelight-docs.
Each build-gated. Deploy once at end.

## Success criteria
- [ ] safe-update retired (crate gone, no registry/alias refs)
- [ ] fsh pkg command retired or NixOS-native
- [ ] zero live pacman/paru/makepkg/AUR invocations in engine + tools
- [ ] identity strings say NixOS, not Arch Linux
- [ ] dead stow/.zshrc paths removed
- [ ] full workspace builds clean, zero warnings; health green
- [ ] re-run the sweep grep: only history + Friday-knowledge + "arch"=architecture remain

## Relationship
- Follows INT-082 (lock model shed). This is the LAST de-Arch work.
- BLOCKS 1.0.0 release: the "pure NixOS-native forest" claim depends on this sweep.

---

## Resolution (INT-116 close) -- 2026-07-03
Two-wave sweep. First wave (the filed target list) + a SECOND wave the first grep missed
(security arch-audit vuln scan, a second pacman lister, extra arch-release checks).
Demonstrated-not-declared: the re-run grep is the judge.

RETIRED (executable Arch code removed):
- safe-update crate (AUR/paru/makepkg builder) + tools.toml + config.fsh aliases;
  `update` alias repointed to faelight-update.
- engine update domain: safe() + simulate() (checkupdates/pacman -Qu) + the Safe/Update
  command chains across dispatcher/cli/parser; ExecutePacman/QueryPacman capabilities.
- fsh `pkg` command (paru/pacman wrapper) + fsh `pkgs`/sys_packages (pacman -Q lister).
- engine security: scan_arch() + get_patchable_packages() (arch-audit CVE scan). vulnix = future.
- doctor entropy: pacman -Q baseline -> NixOS store-path drift detection (now WORKS on NixOS).
- intent-guard check_pacman_remove.
- faelight-docs cmd_welcome (patched dead zsh .zshrc; fsh renders greeting dynamically).
- pip_checker: both /etc/arch-release -> /etc/NIXOS.
- knowledge: arch_pacman_conflict entry.

FIXED (strings/paths):
- Identity strings "Arch Linux"->"NixOS 26.05 (Yarara)" (bootstrap/doctor/narrative/context/teach).
- Fix-hints "sudo pacman"->nix wording (doctor checks, security, secrets gitleaks).
- aliases.toml pacman/paru package aliases removed.
- source_dirs "00-meta" -> "meta" (stale restructure path).
- faelight-update category "AUR Packages" -> "Flatpak Packages" (matched its live checker).

DEFERRED (by decision, not Arch dependence):
- Friday knowledge/suggestion LANGUAGE (friday/mod.rs facts, fsh suggestion strings, teach
  descriptions) -> the Friday-language cleanup (a dedicated future effort).
- Migration history comments -> correct records, kept.
- paths.rs system_fonts_dir /usr/share/fonts -> INT-115 (route hardcoded paths), not Arch.

GATE: final sweep grep shows ZERO executable Arch invocations. All remaining hits are
awareness-language (deferred) or history. Full workspace builds clean, zero warnings.

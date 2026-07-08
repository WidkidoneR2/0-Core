# Parked Ideas -- 2026-07-08

Raised while closing out a strong session (INT-117/130-fix/128/132 + core-intent-new fix).
NONE started -- captured so they survive a 2-day break. Verdicts from checking the live system.

## 1. deadnix -- ADD (small, low-risk)
- Nix SOURCE linter: finds unused `let` bindings, unused fn args in .nix files.
- NOT installed (confirmed: command-not-found; absent from home.nix).
- NO conflict, all verified against the live system:
  - faelight-deadwood = repo/intent ORPHANS (structure level) -- different job
  - faelight-nix = nixpkgs package SEARCH/add TUI (INT-076) -- opposite direction (adds in)
  - nix domain / inspector = "why did this option value win" (evaluated modules) -- different
  - nix-tree / nvd = dependency graphs / gen diffs (store level) -- different
  - deadnix fills the one empty niche: .nix source dead-code lint.
- Companion: `statix` (Nix anti-pattern linter) -- the standard pair.
- Action when back: add to home.packages, `deadnix ~/0-core/nix/`, read output. 2-min warm-up.

## 2. sops (+ rage) -- EVALUATE (real decision, own intent)
- sops-nix: secrets encrypted IN GIT, decrypted at system activation. rage = Rust age (key backend).
- NOT installed. The question is architectural, not "does the tool work":
  - Already have: gocryptfs (encrypted dirs) + faelight-vault (runtime creds).
  - sops fills the DIFFERENT gap: versioned secrets that survive a fresh/recovery install
    with NO manual re-entry (cf. the Cachix token rotated by hand this session).
  - Ties to INT-056 (recovery protocol) + USB-recovery resilience goal.
  - Open question: key bootstrap -- where does the age/rage master key live safely?
  - Valid outcomes BOTH ways: "adopt sops-nix+rage" OR "vault+manual is fine for my threat model".
- Action when back: file as an evaluation intent (like 091 stylix / 119 git-hooks / 122 nixcats).

## 3. Kernel 7.x on Framework 16 -- EVALUATE (boot-touching, VM-first)
- Current reality (verified 2026-07-08): stable = 7.1.3 (Jul 4), mainline 7.2-rc2, LTS = 6.18.
- 7.0 (Apr 12 2026) was a "solid progress" major bump -- NO breaking changes; Linus bumps the
  major ~every 3.5yr purely to avoid big minor numbers. Interfaces stable across 7.x.
- Why it might matter for THIS hardware (Framework 16, AMD):
  - Hybrid scheduler improvements (P-cores kept free for foreground work).
  - NTSYNC stable -> Proton gaming 15-25% in threaded titles (if relevant).
  - Ongoing btrfs work (6.19 experimental + FSCRYPT prep) -- relevant to the btrfs-subvol layout.
- On NixOS this is a `boot.kernelPackages` choice, NOT a package install:
  - What does NixOS 26.05 "Yarara" default to now? (likely a 6.x)
  - linuxPackages_latest vs a pinned 7.x -- leave the LTS line or not?
- CRITICAL: boot-touching -> MUST be VM-tested first (INT-024), like every login/boot change.
  Do NOT bump the metal kernel without VM proof. INT-056/recovery-adjacent.
- Action when back: evaluation intent; VM-test before bare metal.

---
Note: filed as a scratch parking doc, not intents, because these are pre-decision ideas.
sops + kernel would each become proper evaluation intents when actually dug into.

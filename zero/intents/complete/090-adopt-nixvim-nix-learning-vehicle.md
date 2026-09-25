---
id: 090
date: 2026-06-26
type: feature
status: complete
title: "Adopt nixvim as a Nix-learning vehicle (Helix stays primary daily driver)"
tags: [nixvim, neovim, nix-learning, declarative, devshell, helix, tools]
priority: low
---
## Why (reframed 2026-06-26 -- Christian's three reasons)
Original framing was "evaluate, lean skeptical (own vs adopt)". Christian made a stronger,
different case that resolves the tension -- this is NOT about owning the editor, it is about:
  1. NIX-LEARNING VEHICLE. nixvim is configured entirely in Nix modules (programs.nixvim =
     { plugins.X.enable; opts; keymaps; globals; }). Writing it IS practicing Nix module
     syntax, with fast feedback. A low-stakes, high-feedback way to deepen Nix fluency while
     still early on NixOS (migrated months ago).
  2. HELIX STAYS THE DEPENDABLE DAILY DRIVER. Helix (hx, the `v` alias) needs no constant
     tweaking -- it keeps Christian productive while nixvim is the EXPERIMENT running
     ALONGSIDE, never instead. This is the risk-killer: no productivity exposure.
  3. OPEN-MINDEDNESS / FUTURE-PROOFING. Not dogmatically from-scratch; using a well-built
     tool to learn "doing things differently if something does not work one way or another."
     Helpful down the road.
This is consistent with the forest's mature stance: "what serves the forest and daily use,"
not "must be built here." Today's INT-092 work was about reading from truth and staying
open/honest -- adopting nixvim-as-learning fits that, not contradicts it.
## Discipline (the guardrail)
NOT one of the three priorities (0-Core, faelight-shell, Friday). nixvim is a LEARNING
SIDE-CHANNEL -- a different kind of session when wanted, never pulling focus from the main
work or the VM weekend. Helix (v / hx) stays primary and untouched THROUGHOUT every phase.
## Phase 0 -- Branch + compatibility check (DONE 2026-06-26, verified via nixvim docs)
FACTS confirmed:
  - The `nixos-26.05` nixvim branch EXISTS and matches our system (NixOS 26.05 "Yarara").
    Use `github:nix-community/nixvim/nixos-26.05` (NOT main -- main needs nixpkgs-unstable).
  - DO NOT use `inputs.nixpkgs.follows = "nixpkgs"` on the nixvim input. nixvim docs
    explicitly recommend AGAINST it: they test nixvim against THEIR nixpkgs revision; follows
    opts out of those guarantees and is a top cause of the `<name> cannot be found in pkgs`
    error (per their FAQ). Let nixvim bring its own tested nixpkgs.
  - Standalone path confirmed: `makeNixvim` produces a normal derivation; template via
    `nix flake init --template github:nix-community/nixvim`.
  [x] Gate: branch for 26.05 exists, no-follows rule understood, standalone entry identified.
## Phase 1 -- Standalone template, ZERO system impact
  Run `nix flake init --template github:nix-community/nixvim` in a THROWAWAY directory
  (NOT in 0-core). Pin the nixvim input to the nixos-26.05 branch, no follows. Build/run
  with `nix run .#`. Helix completely untouched; nothing enters 0-core.
  [x] Gate: a stock nixvim builds and launches from the template, isolated from the system.
## Phase 2 -- Port a small config slice (THE Nix-learning)
  In the throwaway flake, write a small real config in Nix: a colorscheme, 2-3 plugins,
  some `opts` (number, relativenumber, shiftwidth), maybe a keymap. Build, observe, iterate.
  This is the point: writing nixvim modules = practicing Nix. Feel the ergonomics vs lua.
  [x] Gate: a hand-written nixvim Nix config builds and the editor reflects it.
## Phase 3 -- devShell integration (still no system-wide change)
  Add a nixvim instance to a 0-core devShell via makeNixvim in mkShell buildInputs, so
  `nvim` is available IN THAT SHELL ONLY. System nvim + Helix untouched. This is the
  "available where I want it, contained" form.
  [x] Gate: `nvim` works inside the chosen devShell; outside it, nothing changed.
## Phase 4 -- Decision point
  After living with it: decide -- keep as a devShell tool / promote further / "learned what
  I needed, Helix stays." ALL outcomes are wins; the Nix-learning happened regardless.
  [x] Gate: decision recorded with reasoning.
## Notes
- Errors of the form `vimPlugins.<name> attribute not found` => branch/nixpkgs mismatch
  (or a stale revision). First debugging step: confirm nixos-26.05 branch + no follows.
- makeNixvimWithModule allows splitting config across files + custom modules (later, if wanted).
- Relates to the broader own-vs-adopt thread that produced faelight-shell -- here the answer
  leans ADOPT, because the goal is LEARNING NIX, not owning the editor.
## The Rule
"Helix keeps the work moving; nixvim teaches the Nix. Keep them separate, keep Helix primary,
 and let the experiment deepen the craft without ever risking the day's work." 🌲


## Progress -- 2026-06-26 (Phases 1 + 2 DONE -- richly)
Phase 1: stock nixvim built from the template and launched, fully isolated in ~/nixvim-play
(throwaway, NOT in 0-core). Helix untouched throughout. The Phase-0 pin held: nixos-26.05
branch + NO follows -> no `vimPlugins not found` errors. Build is cached now (fast reruns).
Phase 2: hand-wrote a real config in Nix and watched the editor become it -- a FULL IDE:
  - colorscheme, opts (number/relativenumber/shiftwidth/cursorline/scrolloff), globals (leader=space)
  - plugins: lualine, which-key, treesitter, telescope (ff/fg/fb), neo-tree (<leader>e),
    web-devicons, gitsigns, comment (gcc/gc), nvim-cmp + luasnip autocomplete
  - custom keymaps (<leader>w/q/e) -- all working live
  - CANDY-NEON FOREST THEME (INT-091 crossover): hand-defined nvim highlight groups in the
    forest palette (lime #A6E22E keywords, coral #FF5C57 strings, aqua #36E0D0 types, deep
    forest-black #0B130B bg). The editor now wears the forest's own colors -- in pure Nix.
Nix-learning banked (the whole point): plugins.X.enable model, opts/globals/keymaps as data,
highlight groups, AND the module MERGE/CONFLICT system -- hit "colorschemes.gruvbox.enable has
conflicting definition values" (forest.nix true vs candy-neon.nix false), learned the fix
(remove the duplicate, or lib.mkForce/mkDefault for priority). Real fluency, learned in a
free sandbox. Verdict so far: declarative nixvim's reproducibility genuinely beats LazyVim's
runtime plugin-manager model -- consistent with the forest philosophy.
Phases 3 (devShell integration) and 4 (decision) REMAIN -- optional, for when wanted.
Artifact lives in ~/nixvim-play (not committed to 0-core, by design).


## Phase 3 + 4 -- DONE (2026-06-27): contained devShell nvim + decision -- 090 COMPLETE
Phase 3: candy-neon nixvim wired into the friday-dev devShell, contained.
- nixvim flake input added, pinned github:nix-community/nixvim/nixos-26.05, NO nixpkgs.follows
  (Phase 0 rule held -- nixvim locked its own tested nixpkgs a0374025, zero vimPlugins errors).
- Config modules (default/forest/candy-neon/bufferline.nix) copied from the ~/nixvim-play
  sandbox into TRACKED ~/0-core/config/nixvim/ (flake builds from git-tracked files).
- friday-dev devShell builds forestNvim via nixvim.legacyPackages.makeNixvimWithModule and
  adds it to buildInputs. GATE PROVEN: inside `nix develop`, `nvim` resolves to the nixvim
  store path (candy-neon IDE launches, full plugin set); OUTSIDE, no nvim leaked system-wide
  and Helix (v -> hx) is untouched. Contained exactly as the phase intended.

Phase 4 DECISION (recorded): KEEP nixvim as a contained devShell tool. All three of the
intent's goals achieved: (1) Nix-learning -- module fluency banked richly in Phases 1-2
(plugins.X.enable, opts/globals/keymaps as data, highlight groups, the merge/conflict system);
(2) Helix stayed the dependable daily driver, untouched throughout every phase -- zero
productivity risk; (3) open-mindedness -- ended with a real, reproducible, contained tool, not
a from-scratch build. nixvim now lives in two forms: ~/nixvim-play (experiment sandbox) and
~/0-core/config/nixvim (tracked, feeds the devShell). "Learned the Nix; Helix stays primary;
nixvim available where wanted, contained." A clean all-outcomes-win close.

---
id: 090
date: 2026-06-26
type: feature
status: in-progress
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
  [ ] Gate: a stock nixvim builds and launches from the template, isolated from the system.
## Phase 2 -- Port a small config slice (THE Nix-learning)
  In the throwaway flake, write a small real config in Nix: a colorscheme, 2-3 plugins,
  some `opts` (number, relativenumber, shiftwidth), maybe a keymap. Build, observe, iterate.
  This is the point: writing nixvim modules = practicing Nix. Feel the ergonomics vs lua.
  [ ] Gate: a hand-written nixvim Nix config builds and the editor reflects it.
## Phase 3 -- devShell integration (still no system-wide change)
  Add a nixvim instance to a 0-core devShell via makeNixvim in mkShell buildInputs, so
  `nvim` is available IN THAT SHELL ONLY. System nvim + Helix untouched. This is the
  "available where I want it, contained" form.
  [ ] Gate: `nvim` works inside the chosen devShell; outside it, nothing changed.
## Phase 4 -- Decision point
  After living with it: decide -- keep as a devShell tool / promote further / "learned what
  I needed, Helix stays." ALL outcomes are wins; the Nix-learning happened regardless.
  [ ] Gate: decision recorded with reasoning.
## Notes
- Errors of the form `vimPlugins.<name> attribute not found` => branch/nixpkgs mismatch
  (or a stale revision). First debugging step: confirm nixos-26.05 branch + no follows.
- makeNixvimWithModule allows splitting config across files + custom modules (later, if wanted).
- Relates to the broader own-vs-adopt thread that produced faelight-shell -- here the answer
  leans ADOPT, because the goal is LEARNING NIX, not owning the editor.
## The Rule
"Helix keeps the work moving; nixvim teaches the Nix. Keep them separate, keep Helix primary,
 and let the experiment deepen the craft without ever risking the day's work." 🌲

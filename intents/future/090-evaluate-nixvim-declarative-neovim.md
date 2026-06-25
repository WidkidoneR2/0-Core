---
id: 090
date: 2026-06-25
type: future
status: planned
title: "Evaluate nixvim: declarative neovim config in the flake (own vs configure)"
tags: [evaluation, nixvim, neovim, editor, declarative, flake, philosophy]
priority: low
---
## Why
Found nix-community/nixvim (2026-06-25): a mature (2.8k stars, ~4500 commits) system for
configuring Neovim entirely through Nix modules instead of lua/vimrc. You write
`programs.nixvim = { plugins.X.enable = true; colorschemes.Y.enable = true; }` and it
generates the lua config + installs plugins, everything disabled-by-default for speed.
Philosophically aligned with the forest: declarative, reproducible, everything-in-the-flake.
Current neovim (0.12.3, in systemPackages) is configured the traditional way; nixvim would
fold the editor into the same declarative model as the rest of 0-Core.
## The real question (this is an EVALUATION, not a commitment)
There is a genuine tension to resolve, NOT a foregone conclusion:
- PRO (declarative purity): editor config belongs in the flake like everything else;
  reproducibility everywhere; one config language; matches how 0-Core already works.
- CON (own-your-stack / understanding over convenience): you built faelight-shell FROM
  SCRATCH rather than adopt zsh+plugins because you value understanding your tools deeply.
  nixvim is a LARGE external dependency (a whole flake input, 98.5% Nix, not yours) layered
  over neovim. A hand-written lua config -- even if stowed/flake-tracked -- keeps you closer
  to the metal and fully owned.
The deciding question: is the EDITOR something you want to OWN deeply (like the shell), or
a tool you're happy to just CONFIGURE declaratively and not build? Only Christian can answer.
## Priority honesty
NOT one of the three current priorities (0-Core, faelight-shell, Friday). This is editor
config -- adjacent, not core. Low priority by design. Evaluate when there's slack, not at
the cost of the VM/login/compositor work.
## What "evaluate" means (the gates are about DECIDING, not adopting)
- [ ] Try nixvim in the VM or a devShell (makeNixvim standalone -- no commitment to system):
      `nix flake init --template github:nix-community/nixvim` or the devShell pattern
- [ ] Port a small slice of current neovim config to nixvim modules; feel the ergonomics
- [ ] Assess: does declarative-editor-config genuinely serve daily use, or add a layer
      between you and a tool you'd rather understand directly?
- [ ] DECISION recorded (adopt / hand-owned-lua-in-flake / leave as-is) with reasoning
## Notes
- main branch needs nixpkgs-unstable; there is a nixos-25.11 branch for stable nixpkgs
  (we are on 26.05 -- check branch compatibility before trying).
- Standalone `makeNixvim` lets you build a custom nvim WITHOUT touching the system config --
  the safe way to evaluate (no lockout risk, no system entanglement).
- Relates to the broader "own vs adopt" philosophy thread that produced faelight-shell.
## The Rule
"Declarative is the forest's way -- but so is understanding what you run.
 Evaluate honestly: is this owning the editor, or renting it in Nix's clothes?" 🌲

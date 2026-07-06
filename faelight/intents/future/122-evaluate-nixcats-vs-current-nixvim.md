---
id: 122
date: 2026-07-06
type: future
title: "Evaluate nixCats vs current nixvim"
status: planned
tags: [nix, nixcats, nixvim]
---

## Why
Currently on nixvim (declarative-options editor config). nixCats offers a different
philosophy worth EVALUATING -- but this is an evaluation, NOT a committed migration. The
editor works fine now; this is post-1.0.0 "turn up the heat" territory, not release-critical.
Filed to avoid a snap decision made off a summary; decide by DEMONSTRATION instead.

## The core distinction (the thing to evaluate)
- nixvim / nvf = DECLARATIVE-OPTIONS camp. Config IS Nix options; most Nix-pure; an
  abstraction layer sits between you and Neovim; escape-hatch to Lua when a plugin is not
  wrapped as an option.
- nixCats (BirdeeHub) = REAL-LUA-PACKAGED-BY-NIX camp. Write standard Lua / lazy.nvim
  config; Nix only handles packaging + reproducibility; config stays PORTABLE (works with
  or without Nix); full ecosystem access; you see exactly what Lua runs.

## Why nixCats appeals (the hypothesis to TEST, not assume)
Aligns with the forest ethos -- "understanding over convenience, control every piece":
real Lua = direct understanding of what actually runs, vs nixvim's abstraction. BUT: the
editor may be the ONE place declarative convenience is worth the abstraction. That tension
is resolved by FEEL (using it), not by theory.

## Approach (demonstrated, not declared)
- Try nixCats in a BRANCH. Migrate a SLICE of the current nixvim config.
- Live with it; see if the mental model clicks and whether real-Lua-control actually feels
  better than declarative-options for YOUR editor use.
- Keep nixvim as the working setup until nixCats is proven better in practice.

## Gates
- [ ] nixCats set up in a branch (not main)
- [ ] a real config slice migrated + working
- [ ] honest side-by-side: which feels better to USE and to MAINTAIN
- [ ] decision recorded (migrate / stay on nixvim / hybrid) with rationale

## Relationship
- Editor tooling; post-1.0.0. NOT a release item.
- nvf (NotAShelf) is the third option in the declarative camp -- can be a footnote in the
  comparison, but nixCats vs nixvim is the real fork (different philosophies).
- NOTE: ID collision -- shares 121 with decisions/121 (release process). Renumber to next
  free ID when convenient.

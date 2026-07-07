---
id: 122
date: 2026-07-06
type: future
title: "Evaluate nixCats vs current nixvim"
status: complete
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

## DECISION (gate 4) -- 2026-07-07: MIGRATE TO nixCats
Verdict: **nixCats wins. Adopt it; retire nixvim.** Decided by DEMONSTRATION, not summary.

### How it was evaluated (the discipline -- same lens as INT-043 Attic/Cachix)
Not "which is better in the abstract" but "which fits the forest's actual needs + values."
Ran a real spike: `nix flake init -t github:BirdeeHub/nixCats-nvim#simple` in /tmp, ported
the candy-neon theme to real Lua (nvim_set_hl calls), built + launched the nixCats nvim,
and lived with it.

### The feel-test result (gate 3)
- Real Lua felt DIRECT and CLEAR -- you see exactly what runs (nvim_set_hl vs nixvim's
  attrset abstraction that translates to Lua you never see).
- The model CLICKED immediately -- recognized as "just like LazyVim / normal nvim config."
  That recognition is the point: nixCats speaks the mainstream nvim ecosystem's language,
  with Nix quietly handling packaging underneath instead of an abstraction layer on top.
- Fits the forest ethos: "understanding over convenience, see every piece." Real Lua =
  direct understanding; portable (works with/without Nix); full ecosystem access.

### Why this is NOT just novelty-chasing (the honest guardrail)
nixvim WORKS -- no functional gap forced this (unlike Cachix, which categorically failed).
nixvim is a CONTAINED secondary tool (devShell only; Helix is primary). So the driver is
honestly named: (a) real-Lua fit is genuinely better for how the config is understood/
maintained, and (b) editor-tinkering is enjoyable/valuable. Both legit. The decision is
"better fit, demonstrated by feel" -- not "the incumbent is broken."

### LESSON REINFORCED (from INT-043)
"Recommended/first/incumbent" is an anchor, not an answer -- but ALSO "philosophically
nicer alternative" is not an automatic switch. The corrective both ways: demonstrate fit,
name the real driver honestly. Here the demonstration was decisive AND the driver is honest,
so migrate. (If nixvim had been the daily editor with deep investment, the calculus might
differ -- contained secondary tool + clean feel-test win = safe to migrate.)

### MIGRATION SCOPE (for next session -- this is where the port begins)
Port the full nixvim config (nix/home/dotfiles/nixvim/) to nixCats real-Lua:
- opts + globals (leader=space) -> init.lua vim.opt / vim.g  [candy-neon theme ALREADY
  ported in the spike -> plugin/candy-neon.lua]
- plugins to port: lualine, which-key, treesitter, telescope (+keymaps ff/fg/fb),
  web-devicons, neo-tree, gitsigns, comment, nvim-cmp + luasnip (sources: nvim_lsp,
  luasnip, buffer, path; mappings C-Space/CR/Tab/S-Tab), bufferline
- keymaps: <leader>w write, <leader>q quit, <leader>e neotree toggle
Wiring:
- add inputs.nixcats flake input; build forestNvim via nixCats (replacing
  makeNixvimWithModule at flake.nix ~line 237)
- wire into the friday-dev devShell (where nixvim currently lives)
- remove inputs.nixvim + the nix/home/dotfiles/nixvim/ tree once parity verified
- GATE the retirement on: nixCats nvim launches in the devShell with full parity
  (theme + all plugins + keymaps working) -- demonstrated, not declared
Branch: experiment/nixcats-122 already exists (this spike's decision lives here).
The spike scaffold at /tmp/nixcats-spike is disposable reference (not in repo).

### Footnote
nvf (NotAShelf) -- the third option, also declarative-camp like nixvim -- was NOT
separately spiked; the real fork was nixvim (declarative) vs nixCats (real-Lua), and the
real-Lua camp won on feel. No need to evaluate nvf; it's the same camp we're leaving.

## OPEN QUESTION (2026-07-07): nixCats as PRIMARY dev editor?
A bigger thought surfaced while spiking: could nixCats-nvim take the PRIMARY dev role,
displacing Helix? This is NOT decided -- flagged so it isn't lost, and explicitly NOT
auto-promoted from spike enthusiasm.
- This is a much larger decision than the devShell migration: it displaces the daily
  editor (Helix), changes muscle memory, and raises the reliability bar (primary editor
  breaking interrupts real work; a devShell nvim breaking does not).
- Discipline (demonstrated, not declared -- and don't decide in the honeymoon of a spike
  that just clicked): let nixCats EARN primary. Do the decided devShell migration first,
  daily-drive real-Lua nvim there for a real stretch, THEN honestly assess Helix-vs-nvim
  as primary. "Fun in a 10-min spike" != "my primary editor for months."
- Decide by feel over time, not by current excitement. If after living with it nvim
  genuinely feels like the better primary, promote it deliberately. If Helix still fits
  better for daily driving, nixCats stays the excellent nvim-when-wanted. Both fine.

## MIGRATION COMPLETE -- 2026-07-07 (this session)
Faithful port executed on branch experiment/nixcats-122, then merged.
- Built nix/home/dotfiles/forest-nvim/ : default.nix (categoryDefinitions +
  packageDefinitions, the Nix "package" half) + init.lua + plugin/*.lua (real-Lua
  config half). Plugins ported 1:1 from nixvim: lualine, bufferline, which-key,
  treesitter, telescope(+ff/fg/fb), neo-tree, gitsigns, comment, nvim-cmp+luasnip
  (nvim_lsp/luasnip/buffer/path sources), web-devicons, candy-neon theme.
- Wired into friday-dev devShell (flake.nix): forestNvim = import ./forest-nvim
  { pkgs; nixCats = inputs.nixcats; }, replacing the nixvim makeNixvimWithModule.
- nixvim RETIRED: inputs.nixvim removed, nix/home/dotfiles/nixvim/ deleted, devShell
  rebuilds clean without it.
- One real API-drift fix during the port (exactly the kind of thing nixCats surfaces
  and nixvim hid): modern nvim-treesitter removed `.configs.setup{}`; replaced with a
  FileType autocmd calling vim.treesitter.start (grammars from withAllGrammars).
- PARITY VERIFIED by launching forest-nvim in the devShell: candy-neon colors correct,
  line numbers, treesitter highlight, lualine, bufferline, telescope, neo-tree, which-key,
  comment, cmp -- all working. Demonstrated, not declared.

Binary: `forest-nvim` (defaultPackageName), available in `nix develop ~/0-core`.

## Gates -- ALL MET
- [x] nixCats set up in a branch (experiment/nixcats-122)
- [x] a real config slice migrated + working (full config, not just a slice)
- [x] honest side-by-side: real Lua felt direct/clear, clicked (LazyVim-like) -- nixCats won
- [x] decision recorded (MIGRATE) with rationale + the discipline lesson
- [x] migration executed + parity verified + nixvim retired

## Note carried forward
OPEN QUESTION (above) -- nixCats as PRIMARY editor vs Helix -- remains open by design.
Daily-drive the devShell forest-nvim first; let it earn primary. Not decided here.

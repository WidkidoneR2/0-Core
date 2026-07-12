---
id: 149
date: 2026-07-12
type: future
title: "Remove hx (helix): drop package + config + repoint v/svi aliases to nvim -- completes the INT-147 editor switch"
status: complete
tags: [editor, helix, nixcats, config, cleanup]
---

## Vision
hx (helix) fully removed -- no package, no deployed config, no dangling aliases or symlinks. nvim
(INT-122 nixCats forest-nvim) is the only editor. Completes what INT-147 started: 147 switched the
DEFAULT editor to nvim; this removes the SUPERSEDED tool and its dead references.

## The Problem
INT-147 pointed $EDITOR at nvim but left hx installed and referenced in FOUR places (verified by
recon 2026-07-12):
- `nix/home/christian/home.nix:66` -- `helix` in home.packages (the install)
- `nix/home/christian/home.nix:103-104` -- `xdg.configFile."helix"` block (deploys the helix config
  dir + the faelight-forest helix theme)
- `nix/home/dotfiles/faelight-shell/.config/faelight-shell/config.fsh:18` -- `alias v = "hx"`
- `nix/home/dotfiles/faelight-shell/.config/faelight-shell/config.fsh:306` -- `alias svi = sudo hx`
hx is currently on PATH (/etc/profiles/per-user/christian/bin/hx). Removing the package alone would
leave `v`/`svi` pointing at a gone binary and a dangling helix config symlink (which the doctor's
Broken Symlinks check would then flag).

## The Decision (baked in)
The `v` (quick edit) and `svi` (sudo edit) aliases are useful BINDINGS, not helix-specific. Chosen
approach: REPOINT them to nvim (`alias v = "nvim"`, `alias svi = "sudo nvim"`), not delete -- keep
the muscle memory, aim it at the new editor. (Alternative if preferred at execution: delete the
aliases entirely. Repoint is the default.)

## The Solution (ORDERED -- order matters for safety)
1. Repoint the two aliases in config.fsh FIRST (v -> nvim, svi -> sudo nvim) -- so there is never a
   window where they point at a removed binary.
2. Remove the `helix` package line (home.nix:66).
3. Remove the `xdg.configFile."helix"` block (home.nix:103-104) so no helix config symlink is
   deployed (prevents a dangling symlink).
4. Helix theme file (nix/home/dotfiles/helix/.config/helix/themes/faelight-forest.toml): decide
   keep-as-harmless-dotfile vs remove-for-cleanliness -- record the choice + reason.
5. dep.
6. Verify: `which hx` -> not found; `v`/`svi` open nvim; Broken Symlinks check green (no dangling
   helix symlink); health not degraded.

## Success Criteria
- [x] `v` and `svi` aliases repointed to nvim (config.fsh) <!-- 2026-07-12: config.fsh:18 v->nvim, :306 svi->sudo nvim. Repointed FIRST (before pkg removal) per ordered solution. -->
- [x] `helix` package removed (home.nix:66); deployed <!-- 2026-07-12: removed helix + its '# Editor' comment from home.packages. Deployed gen 360. -->
- [x] `xdg.configFile."helix"` block removed; NO dangling helix config symlink <!-- 2026-07-12: removed the 4-line xdg.configFile block; Broken Symlinks check GREEN post-dep. -->
- [x] helix theme file decision recorded -- REMOVED <!-- 2026-07-12: git rm -r nix/home/dotfiles/helix/ (config.toml + faelight-forest.toml theme). Reason: Christian wants nvim only, no helix -- an orphaned dotfile dir nothing reads is deadwood. -->
- [x] `which hx` -> not found; no hx/helix references remain except history <!-- 2026-07-12: hx gone from /etc/profiles/per-user/christian/bin. RECON SURFACED 2 engine refs the nix/ sweep missed: checks.rs:132 required-binaries list ('hx'->'nvim', so the editor stays health-checked) + entropy.rs:155 drift-baseline ('helix' removed). Both cleaned; doctor engine sweep clean. -->
- [x] deploy clean; health not degraded <!-- 2026-07-12: the removal FIRST degraded health to 87% (Binary Dependencies ❌ -- engine still expected hx). Caught by the check, fixed (hx->nvim in the list), re-deployed: Binary Dependencies GREEN, 0 failed, health recovered to 90%. The board-green discipline caught what 'hx gone from PATH' alone would have missed. -->

## Relationship
Completes: INT-147 (editor switch to nvim). 147 switched the default; this removes the superseded
tool + dead references. No structural dependencies.
Filter: removing a superseded tool and its dead references deepens reproducible control (one editor,
no orphaned config/aliases); leaving danglers erodes it. In-filter.

## Notes
- The four references were located via recon 2026-07-12 during INT-146 work -- captured here so
  execution needs no re-hunt.
- ORDER is the safety rail: repoint aliases BEFORE removing the package, so `v`/`svi` never point at
  a gone binary even momentarily.
- Small intent, low risk -- but real (4 references across 2 files + a deploy). Not a one-liner.

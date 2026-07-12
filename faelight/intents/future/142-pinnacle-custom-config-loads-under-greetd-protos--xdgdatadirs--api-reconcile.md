---
id: 142
date: 2026-07-11
type: future
title: "Pinnacle custom config loads under greetd (protos + XDG_DATA_DIRS + API reconcile)"
status: planned
tags: [pinnacle, greetd, compositor, 067, reproducibility]
---

## Problem
Pinnacle's custom Lua config (~/.config/pinnacle/lua/init.lua, built June 13) has NEVER
loaded under greetd -- it silently falls back to Pinnacle's DEFAULT config. That's why the
custom forest keybinds (Super+B=brave, Super+E=broot, Super+P=faelight-bar) don't work,
while Super+Return/Super+F do (those exist in the default config too). IMPORTANT: this means
the earlier 'Pinnacle works on metal' confirmation (INT-086 cancel, INT-056) was observing
the DEFAULT config, not the custom one.

## Root-cause chain (diagnosed 2026-07-11 via the pinnacle logs at ~/.local/state/pinnacle/)

### Layer 1 -- FIXED (gen 349, committed)
`lua` was not in greetd's SYSTEM PATH, so `run = ["lua", "lua/init.lua"]` in pinnacle.toml
failed with 'No such file or directory' -> config never started -> default fallback. The lua
interpreter was only in the per-user profile (which the interactive shell has but a
greetd-launched session does not). FIX: added `pkgs.lua5_4` to nix/modules/desktop/pinnacle.nix
systemPackages. Confirmed: after this, the log shows `Started config with ["lua", ...]` using
/run/current-system/sw/.../lua-5.4.7. (Detour lesson: an earlier attempt to set PATH via
pinnacle.toml [envs] BROKE it worse -- it overrode PATH to dirs lacking lua. Reverted. Do NOT
set a bare PATH in [envs]; it replaces rather than extends.)

### Layer 2 -- THE NEXT FIX (not yet done)
Config now STARTS but crashes at Pinnacle.setup() (init.lua:15) with:
  protobuf.lua:51: could not find protobuf definitions directory
The Lua (share/lua/5.4/pinnacle/grpc/protobuf.lua) searches `$XDG_DATA_HOME` (or
`$HOME/.local/share`) plus each `$XDG_DATA_DIRS` entry for `<dir>/pinnacle/protobuf`, then
runs `protoc` on the .proto files. Under greetd, $HOME/$XDG_DATA_HOME aren't set like the
shell, so it misses the (fragile, untracked) home symlink ~/.local/share/pinnacle/protobuf.

THE FIX: the client-api package `lua5.4-pinnacle-client-api-0.2.3` SHIPS the protos at the
exact needed structure: `<pkg>/share/pinnacle/protobuf/` (verified -- it has pinnacle/*/v1/
and google/protobuf/ subtrees). Add that package to the Pinnacle module's systemPackages so
its `share/` lands in the SYSTEM $XDG_DATA_DIRS that greetd has -> Lua finds the protos
reproducibly, no home symlink needed. `protoc` is ALREADY in the system path (confirmed at
/run/current-system/sw/bin/protoc), so no protoc fix needed.

BLOCKER hit tonight: naming the client-api package via the flake input. The pinnacle flake
input exposes packages under `inputs.pinnacle.packages.${system}` but the attribute name for
the client-api wasn't resolved (nix eval introspection was fiddly at the hour). NEXT SESSION:
find the attribute (try: `nix eval` on the input's packages attrNames from a repl, or read the
pinnacle flake.nix outputs, or check how inputs.pinnacle.packages.${system}.pinnacle pulls it
in -- the client-api may already be a dependency whose share just needs linking). Alternative
if the attribute is elusive: pathsToLink / a system XDG_DATA_DIRS entry pointing at the
client-api store output via the module.

### Layer 3 -- RISK (unknown until Layer 2 clears)
init.lua is from June 13 against an older API; installed client-api is 0.2.3. Once protos
load, the config MAY hit API-drift errors (Snowcap.integration.bind_overlay,
focus_border_with_titlebar, keybind signatures, Layout.builtin.* etc.). If so, reconcile the
config against the 0.2.3 Lua API. May be clean; may need edits. Unknown until tested.

## Reproducibility gap (fold in or spin out)
- The entire ~/.config/pinnacle/ (pinnacle.toml + lua/init.lua) is UNTRACKED -- lives in home,
  not home-manager/the flake. A fresh install would lack it. Should be tracked via
  home-manager dotfiles (same gap as the mango config).
- The ~/.local/share/pinnacle/protobuf symlink is a hand-created artifact pointing at a SOURCE
  store path that is now EMPTY/gone. Retire it once the XDG_DATA_DIRS fix (Layer 2) lands.

## Success criteria
- [ ] client-api protos available via system XDG_DATA_DIRS (Layer 2) -- config gets past the
      protobuf assert; log shows setup() completing, no 'Config crashed! Falling back'.
- [ ] Custom config actually loads under greetd: Super+B (brave), Super+E (broot), Super+P
      (faelight-bar) all fire from a real Pinnacle greetd session (demonstrated on metal).
- [ ] Any Layer-3 API-drift errors reconciled against pinnacle 0.2.3 (or confirmed none).
- [ ] Pinnacle config tracked in home-manager (reproducible); the stale ~/.local/share
      protobuf symlink retired.

## Context / notes
- Core goal (reach Claude from Pinnacle) ALREADY WORKS via typing `brave` in a terminal --
  brave is on PATH. This intent is about the custom config loading so the KEYBINDS work.
- The custom config already CONTAINS everything wanted (brave/broot/faelight-bar binds,
  autostart of terminal + faelight-notify, tags, layouts, media keys, borders). Nothing to
  build in the config -- it just needs to LOAD.
- Pinnacle logs: ~/.local/state/pinnacle/YYYY-MM-DD-HH.pinnacle.log (grep for 'config',
  'crashed', 'Unable to load', Lua stack traces).
- Diagnostic method that worked: read the pinnacle log stderr; it names the exact Lua file:line.

## Relates To
- INT-067 (faelight-bar under secondary compositor) -- 067's bar needs Pinnacle's config to
  actually load first; this unblocks the Pinnacle side of 067.
- INT-087 (Miracle) -- Miracle likely has the SAME class of issue (config env under greetd);
  fixes here inform Miracle.
- The goal: 'switch to any profile and still communicate' -- Pinnacle keybinds are the polish
  on top of the already-working typed-brave path.
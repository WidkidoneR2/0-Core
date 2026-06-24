---
id: 088
date: 2026-06-23
type: future
status: in-progress
title: "Nix Inspector: why did this value win? (option-resolution debugger)"
tags: [nix, debugging, tool, understanding, forest-native]
version: TBD
---
## Why
Biggest Nix pain point: WHY did an option end up this way? Set services.openssh.enable in
three files -- which wins, and why? Nix has the data (module system, priorities); the UX is
missing. "Understanding over convenience" made into a tool. (Christian's idea, 2026-06-23.)
## Vision
  inspect services.openssh.enable
  -> Value: true | Type: bool
     Defined in: hosts/framework16:108 ; modules/security/ssh.nix:12 (mkDefault)
     Winner: framework16:108 (explicit overrides mkDefault)
## How (research first)
- nixos-option already shows definitions + value; the tool is a forest-native, themed,
  readable layer over it (ratatui TUI or fsh builtin). Also: definitionsWithLocations,
  nix eval of the module system. Integration: `inspect <opt>` builtin / `core nix inspect`.
## Phases
P0 research (nixos-option, definitionsWithLocations). P1 MVP (value/type/definitions/winner).
P2 the "why" (mkDefault/mkForce/mkOverride priority). P3 forest UX (themed TUI/builtin).
## Related (captured, NOT this intent)
- Nix Time Machine (gen diff/browse): OVERLAPS INT-074 (gen browser) + INT-044 (done). Fold
  into 074.
- Nix Control Center (GUI -> generate Nix): TENSION with hand-authored philosophy. Capture as
  inspect/visualize, not auto-generate. Separate future.
## The Rule
"If you cannot say why the value won, you do not understand your system." 🌲

## Progress -- 2026-06-24 (Phase 0 research + Phase 1 MVP -- WORKING, not yet closed)
Phase 0 findings (the foundation):
- `nixos-option --flake <repo>#<hostname> <option>` is the engine. Cached calls ~0.6-1.3s
  (fast enough for interactive use). Gives Value / Default / Type / Description /
  Declared-by (where the OPTION is defined, upstream) / Defined-by (where the VALUE is set
  -- what won).
- Freeform submodule leaves (nix.settings.*, etc.) THROW "inside submodule option while
  traversing" -- handled by falling back to the parent attrset query + an info note.
- "Defined by" returns /nix/store/HASH-source/<path> -- translated to repo-relative paths.
- Multi-source options DO exist: nix.settings shows 4 Defined-by sources (our config + 3
  upstream nix modules) -- the real "who shaped this value" case.
Phase 1 MVP (BUILT + DEPLOYED, gen 224):
- New engine domain engine/src/domains/nix/mod.rs (~156 lines): inspect(option) wraps
  nixos-option, parses output, translates store-paths -> repo-paths, submodule-parent
  fallback, flags "value equals default -> redundant". Themed forest output.
- Wired as `core nix inspect <option>` (parser.rs clap Commands::Nix + NixCommands::Inspect;
  commands.rs internal Command::Nix + NixCommand; cli/mod mapping; dispatcher arm;
  domains/mod registration). Built as a CORE capability so the future friday-daemon (INT-039)
  can consume option resolution programmatically -- not just an fsh builtin.
- `inspect` alias -> `core nix inspect` added to config.fsh (live after next deploy).
VERIFIED LIVE (gen 224):
  core nix inspect services.openssh.enable -> Value true, Default false, Defined by
    hosts/framework16/configuration.nix (translated), Declared by upstream sshd.nix.
  core nix inspect networking.firewall.enable -> "value equals default -- redundant" flag fired.
  core nix inspect nix.settings.cores -> submodule fallback to parent, 4 Defined-by sources.
LESSON (Friday should learn): a NEW engine file must be `git add`-ed before nixos-rebuild --
  the flake source only includes git-tracked files, so an untracked nix/mod.rs gave E0583
  ("file not found for module") in the nix build even though `cargo check`/working-tree build
  passed. Stage new files before the nix build.
STILL OPEN (why this stays in-progress):
- Phase 2 "the why": explain priority/override (mkDefault/mkForce/mkOverride) -- nixos-option
  shows values but not WHY one won when priorities differ. Needs deeper module-system query.
- Phase 3 forest UX: themed TUI or richer presentation (current is clean prose output).
- Obsolete-option-name flagging (nixos-option -r surfaces renames as traces -- could detect).

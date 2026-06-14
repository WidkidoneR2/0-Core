---
id: 061
date: 2026-06-14
type: future
title: \"Canonical 0-Core repository structure on NixOS\"
status: planned
tags: [faelight]
version: TBD
---

## Why
NixOS makes the repo bigger and multi-faceted: a flake, profiles, host configs,
system + user modules, derivations -- plus the unchanged Faelight project
(engine, rust-tools, intents, runtime). Without a canonical structure the layout
drifts and stops expressing the 0-Core layer model. This intent fixes the target
so every future file has an obvious, principled home -- and so the repo still
*is* 0-Core, not just a tidy NixOS config.

## The 0-Core invariant (what must survive any layout)
- Layer separation is VISIBLE in the tree (substrate / core / declarative /
  policy / runtime / adapters).
- Declarative-over-imperative: the OS-level registry is expressed in Nix
  (flake/profiles/modules/hosts); no imperative drift.
- runtime/ stays gitignored and `rm -rf runtime/` stays safe.
- Single orchestrator (core) unchanged; engine authored, not generated.
- Understanding over convenience: forest modules are hand-authored, never a bare
  upstream `enable = true` we do not understand.

## Vision -- canonical target tree
  0-core/
  ├── flake.nix                  # manifest only -- inputs + outputs wiring
  ├── profiles/                  # LAYER 2 (OS registry, in Nix)
  │   ├── base.nix               #   every machine
  │   ├── desktop.nix            #   GUI machines
  │   ├── laptop.nix             #   Framework-specific
  │   ├── development.nix        #   dev tools
  │   └── security.nix           #   hardening toggles
  ├── hosts/
  │   └── framework16/
  │       ├── configuration.nix  #   imports profiles
  │       ├── hardware-config.nix
  │       └── disko.nix
  ├── modules/                   # LAYER 5 (adapters, native NixOS)
  │   ├── services/
  │   │   └── friday.nix         #   SYSTEM service (cross-session nervous system)
  │   ├── forest/
  │   │   └── faelight-tools.nix #   wires derivations (delegates to pkgs/)
  │   ├── security/
  │   │   ├── luks.nix
  │   │   ├── firewall.nix
  │   │   └── hardening.nix
  │   └── desktop/
  │       ├── mango.nix          #   daily driver (was niri.nix)
  │       ├── niri.nix           #   optional, only if a deliberate fallback
  │       └── greetd.nix         #   login -- ISOLATED module (lockout-class)
  ├── users/
  │   └── christian/             # USER scope (home-manager, per-session)
  │       ├── home.nix           #   imports user modules
  │       ├── fsh.nix            #   user shell (+ config.fsh source of truth)
  │       ├── alacritty.nix      #   user terminal
  │       ├── git.nix            #   user git config
  │       ├── faelight-bar.nix   #   per-session bar (USER service, not system)
  │       └── faelight-notify.nix#   per-session notify (USER service)
  ├── pkgs/faelight/             # custom derivations (build the rust-tools)
  ├── tests/                     # NixOS VM tests -- gate every rebuild
  │   ├── boot.nix               #   boots, no critical kernel errors
  │   ├── login.nix              #   greetd -> usable mango session (anti-lockout)
  │   └── friday-service.nix     #   friday.service starts + responds
  ├── labs/                      # was r-and-d/
  │   ├── experiments/
  │   └── graduated/
  │
  │  # ---- Faelight project (the eventual `faelight/` half) ----
  ├── engine/                    # LAYER 1 -- core orchestrator (unchanged)
  ├── rust-tools/                # specialist TUI tools (unchanged)
  ├── registry/                  # LAYER 2 (engine-side: zones, capabilities)
  ├── policy/                    # LAYER 3 -- constraints + health-check defs
  ├── intents/                   # intent ledger (unchanged)
  ├── docs/                      # human documentation
  └── runtime/                   # LAYER 4 -- gitignored, rm -rf safe
      └── state.db

## Key decisions baked in
1. desktop/ names reality: mango.nix is the daily driver; niri.nix only if kept
   as an explicit fallback. No stale niri-as-default.
2. greetd gets its OWN module. Login is lockout-class; isolating it makes it
   testable and keeps boot/login changes surgical.
3. Layers 2 + 3 stay VISIBLE on the Faelight side. The OS-level registry
   dissolves into Nix (profiles); the engine-level registry does NOT --
   registry/ (zones, capabilities Nix cannot model) and policy/ (constraints,
   health-check defs) remain explicit dirs, not vanished by omission.
4. System vs user is a deliberate authority boundary:
   - SYSTEM (modules/): friday.service, security, login, host.
   - USER (users/christian/): bar, notify, shell, terminal -- anything bound to
     a Wayland session.
5. Derivations live with / are exposed by the tools they build; modules/ only
   wires them. This pre-cuts the faelight / faelight-os seam.

## The seam -- monorepo now, split-ready
Dependency is one-directional: faelight-os depends on faelight, never the
reverse. Keep that boundary clean so the eventual split is a `filter-repo`, not
a rewrite. Do NOT physically split yet -- defer until a real trigger: an external
consumer of the tools (fsh / Friday), or a public project + private machine
config (conference release). Until then: root-level siblings grouped logically
by the divider above.

## Phases
Phase 0 -- Spec lock. This doc is the canonical structure.
  Gate: agreed + committed.
Phase 1 -- Non-risky renames + homes. r-and-d/ -> labs/. Create registry/ +
  policy/ explicit homes. docs/ listed.
  Gate: tree matches spec for all NON boot/login paths; rebuild clean.
Phase 2 -- desktop/ truth + greetd isolation. Split into mango.nix
  (+ optional niri.nix) + greetd.nix.
  Gate: greetd is its own module; NO behavioural change to login.
Phase 3 -- tests/ harness (BEFORE any boot/login move lands). Add boot.nix +
  login.nix + friday-service.nix to `nix flake check`.
  Gate: all three pass in a VM; login test asserts a reachable mango session.
Phase 4 -- system/user re-scoping. Move bar + notify to users/christian/ as user
  services; confirm friday stays a system service.
  Gate: services land on their chosen side; rebuild + tests green.
Phase 5 -- seam tidy (NOT the split). Group Faelight-side dirs under the divider;
  pkgs derivations reference tool source cleanly.
  Gate: faelight-os -> faelight dependency is one-directional, no reverse refs.

## Hard rule (lockout-class)
No change touching boot, login (greetd), disko, or the host config lands without
a passing VM test FIRST. The test goes red in CI instead of locking the laptop.
(INT-045 login gate + the 24h greetd lockout are the precedent.)

## Depends On
- INT-056 (Forest Recovery Protocol / TTY2 hardening) -- safety net for boot/login moves
- INT-045 (devShells / direnv) -- the build environment this structure assumes
- tests/ harness (Phase 3) gates Phases 4-5

## Supersedes / absorbs
- planned: r-and-d -> labs rename
- planned: tests/ with NixOS VM tests
- planned: mono-repo split (deferred here behind a trigger, not dropped)

## The Rule
"The structure is the philosophy made visible. If you cannot point at the layer
 in the tree, it is not 0-Core -- it is just files." 🌲

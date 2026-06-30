---
id: 061
date: 2026-06-14
type: future
title: "Canonical 0-Core repository structure on NixOS"
status: in-progress
tags: [faelight]
version: TBD
---
## Why
NixOS makes the repo bigger and multi-faceted: a flake, profiles, host configs,
system + user modules, derivations -- plus the Faelight project (engine,
rust-tools, intents, runtime). Without a canonical structure the layout drifts
and stops expressing the 0-Core layer model. This intent fixes the target so
every future file has an obvious, principled home -- and so the repo still *is*
0-Core, not just a tidy NixOS config.

## The 0-Core invariant (what must survive any layout)
- Layer separation is VISIBLE in the tree (substrate / core / declarative /
  policy / runtime / adapters).
- Declarative-over-imperative: the OS-level registry is expressed in Nix
  (flake/profiles/modules/hosts); no imperative drift.
- runtime/ stays gitignored and `rm -rf` of it stays safe.
- Single orchestrator (core) unchanged; engine authored, not generated.
- Understanding over convenience: forest modules are hand-authored, never a bare
  upstream `enable = true` we do not understand.
- Dependency is ONE-DIRECTIONAL: the OS half depends on the Faelight half, never
  the reverse. (This is the seam the whole structure protects.)

## Structural evolution (2026-06-30): two first-class domains
The original charter expressed the OS/Faelight seam as a COMMENT divider between
root-level siblings. Upgrade: promote the divider to real directories. Directories
are stronger than comments -- they encode the layer model AND the eventual repo
split into the filesystem itself, so `tree` teaches the architecture with zero
documentation. Even if we never split the repo, separation of concerns is obvious
from the first tree output.

  Platform (Faelight)   ->   faelight/
        ^
        |
  Operating System      ->   nix/
        ^
        |
  Host                  ->   nix/hosts/

## Vision -- canonical target tree (v2)
  0-core/
  |-- flake.nix                  # manifest only -- inputs + outputs wiring
  |
  |-- nix/                       # ===== OPERATING SYSTEM (consumes faelight) =====
  |   |-- profiles/              #   LAYER 2 (OS registry, in Nix)
  |   |   |-- base.nix           #     every machine
  |   |   |-- desktop.nix        #     GUI machines
  |   |   |-- laptop.nix         #     Framework-specific
  |   |   |-- development.nix    #     dev tools
  |   |   `-- security.nix       #     hardening toggles
  |   |-- hosts/
  |   |   `-- framework16/
  |   |       |-- configuration.nix
  |   |       |-- hardware-config.nix
  |   |       `-- disko.nix
  |   |-- modules/               #   LAYER 5 (adapters, native NixOS)
  |   |   |-- system/            #     NEW: boot, networking, locale (was junk-drawer risk)
  |   |   |   |-- boot.nix
  |   |   |   |-- networking.nix
  |   |   |   `-- locale.nix
  |   |   |-- desktop/
  |   |   |   |-- mango.nix       #     daily driver (reality, not niri)
  |   |   |   |-- niri.nix        #     optional, ONLY if a deliberate fallback
  |   |   |   `-- greetd.nix      #     login -- ISOLATED module (lockout-class)
  |   |   |-- security/
  |   |   |   |-- luks.nix
  |   |   |   |-- firewall.nix
  |   |   |   `-- hardening.nix
  |   |   `-- forest/             #     KEPT: faelight-adapter layer (load-bearing)
  |   |       |-- friday.nix      #     SYSTEM service (cross-session nervous system)
  |   |       `-- faelight-tools.nix #  wires derivations (delegates to faelight/packages/)
  |   |-- home/                  #   RENAMED from users/ -- these are Home Manager modules
  |   |   `-- christian/         #     (see OPEN DECISION below re: home/ vs users/)
  |   |       |-- home.nix
  |   |       |-- fsh.nix
  |   |       |-- alacritty.nix
  |   |       |-- git.nix
  |   |       |-- faelight-bar.nix    # per-session bar (USER service)
  |   |       `-- faelight-notify.nix # per-session notify (USER service)
  |   `-- tests/                 #   NixOS VM tests -- mirror the architecture
  |       |-- boot/
  |       |   `-- boots.nix
  |       |-- desktop/
  |       |   `-- login.nix      #     greetd -> usable mango session (anti-lockout)
  |       |-- forest/
  |       |   `-- friday.nix     #     friday.service starts + responds
  |       `-- security/
  |           `-- luks.nix
  |
  |-- faelight/                  # ===== PLATFORM (produced; OS consumes) =====
  |   |-- engine/                #   LAYER 1 -- core orchestrator (unchanged)
  |   |-- rust-tools/            #   specialist TUI tools (unchanged)
  |   |-- packages/              #   NEW: derivations live with the project that
  |   |                          #     produces them (was nix-side pkgs/faelight/).
  |   |                          #     nix/modules/forest/ merely WIRES these.
  |   |-- registry/              #   LAYER 2 (engine-side: zones, capabilities)
  |   |-- policy/                #   LAYER 3 -- constraints + health-check defs
  |   |-- intents/               #   intent ledger (unchanged)
  |   `-- runtime/               #   LAYER 4 -- gitignored, rm-rf safe
  |       `-- state.db           #     (relocation here is DEFERRED -- see below)
  |
  |-- docs/                      # human documentation
  `-- labs/                      # experimental -- kept at the bottom, off the
      |-- experiments/           #   critical path so it doesn't compete visually
      `-- graduated/             #   with canonical architecture

## Key decisions baked in
1. nix/ + faelight/ are the two first-class domains. The dependency arrow
   (nix depends on faelight, never reverse) is now STRUCTURAL, not a comment.
   The eventual repo split becomes a `git filter-repo` of one directory.
2. desktop/ names reality: mango.nix is the daily driver; niri.nix only if kept
   as an explicit fallback. No stale niri-as-default.
3. greetd gets its OWN module. Login is lockout-class; isolating it makes it
   testable and keeps boot/login changes surgical.
4. Layers 2 + 3 stay VISIBLE on the Faelight side. The OS-level registry
   dissolves into Nix (profiles); the engine-level registry does NOT --
   faelight/registry/ (zones, capabilities Nix cannot model) and faelight/policy/
   (constraints, health-check defs) remain explicit dirs.
5. System vs user is a deliberate AUTHORITY boundary:
   - SYSTEM (nix/modules/): friday.service, security, login, host, boot.
   - USER (nix/home/christian/): bar, notify, shell, terminal -- anything bound
     to a Wayland session.
6. Derivations live with / are exposed by the project that builds them
   (faelight/packages/); nix/modules/forest/ only WIRES them. This pre-cuts the
   faelight / faelight-os seam and reinforces the dependency direction.
7. modules/system/ added so boot/networking/locale have a home and `services`
   never becomes a junk drawer -- WITHOUT dissolving forest/ (which is the
   load-bearing faelight-adapter layer, kept deliberately).
8. tests/ mirror the source architecture (boot/ desktop/ forest/ security/) so
   the harness scales as tests grow.

## OPEN DECISIONS (conscious, not silently resolved)
- **home/ vs users/**: renaming users/ -> home/ is the Nix convention (these ARE
  Home Manager modules, not accounts) and reads naturally for multi-user. BUT the
  original charter chose `users/` to emphasise the system/user AUTHORITY boundary
  (decision #5), not the HM mechanism. Trade-off: `home/` = convention clarity;
  `users/` = authority-boundary emphasis. LEANING home/ (convention wins, and the
  authority boundary is already carried by the nix/modules vs nix/home split).
  Decide at Phase-1 execution, not silently.
- **runtime/ placement**: by OWNERSHIP, state.db is the engine's state ->
  faelight/runtime/ is correct (shown above). BUT runtime/state.db paths are
  HARDCODED in ~15+ rust-tools (db.rs and others). Relocating runtime is the SAME
  hardcoded-path-refactor class as meta/schema/config (see Deferred). So:
  faelight/runtime/ is the TARGET, but the move is FOLDED INTO the deferred
  runtime-path refactor pass, not done as a quick `git mv`.

## The seam -- monorepo now, split-ready
Dependency one-directional: nix/ (OS) depends on faelight/ (platform), never the
reverse. The two-domain layout makes this a structural guarantee. Do NOT physically
split yet -- defer until a real trigger: an external consumer of the tools (fsh /
Friday), or a public project + private machine config (conference release). Until
then: two sibling domains under one root.

## Phases (re-sequenced: low-risk -> lockout-class, VM-gated)
Phase 0 -- Spec lock. THIS doc (v2) is canonical. Gate: agreed + committed.
Phase 1 -- Non-risky homes + the nix/ + faelight/ top split for NON-gated paths.
  Create nix/ and faelight/ domains; move docs/, labs/, tests/, registry/, policy/,
  intents/, engine/, rust-tools/ into place. Resolve home/ vs users/. r-and-d ->
  labs (already done). Gate: tree matches v2 for all NON boot/login/runtime paths;
  `nix flake check` + rebuild clean. (No hardcoded-path files moved yet.)
Phase 2 -- desktop/ truth + greetd isolation. Split into mango.nix (+ optional
  niri.nix) + greetd.nix; add modules/system/. Gate: greetd its own module; NO
  behavioural change to login. LOCKOUT-CLASS -> VM test first.
Phase 3 -- tests/ harness (BEFORE any boot/login move lands). boot/ + desktop/login
  + forest/friday into `nix flake check`. Gate: all pass in a VM; login test asserts
  a reachable mango session. (Blocked by the metal-gated VM-login finding -- see
  Deferred; needs a working login-test path.)
Phase 4 -- system/user re-scoping. Move bar + notify to nix/home/christian/ as user
  services; confirm friday stays a system service. Gate: services on chosen side;
  rebuild + tests green.
Phase 5 -- packages move + seam tidy (NOT the split). pkgs/faelight/ -> 
  faelight/packages/; nix/modules/forest/ references them cleanly. Gate: nix -> 
  faelight dependency one-directional, no reverse refs.
Phase 6 -- DEFERRED hardcoded-path refactor (the careful one). Move meta/, schema/,
  config/, AND runtime/ to their faelight/ homes, fixing the ~15+ rust-tools that
  hardcode these paths, one tool at a time, understood not swept. Gate: every path
  usage read + updated; rebuild; NO runtime breakage; rm-rf runtime still safe.

## Hard rule (lockout-class) -- UNCHANGED
No change touching boot, login (greetd), disko, or the host config lands without a
passing VM test FIRST. The test goes red in CI instead of locking the laptop.
(INT-045 login gate + the 24h greetd lockout are the precedent.) The candy-tuigreet
metal ship (2026-06-29) followed exactly this: VM-proven, staged, rescue-armed.

## Depends On
- INT-056 (Forest Recovery Protocol / TTY2 hardening) -- safety net for boot/login moves
- INT-045 (devShells / direnv) -- the build environment this structure assumes
- tests/ harness (Phase 3) gates Phases 4-6

## Supersedes / absorbs
- planned: r-and-d -> labs rename (done)
- planned: tests/ with NixOS VM tests
- planned: mono-repo split (deferred behind a trigger, now structurally pre-cut)

## The Rule
"The structure is the philosophy made visible. If you cannot point at the layer in
 the tree, it is not 0-Core -- it is just files." The two-domain split makes the
 deepest philosophy -- platform produced, OS consumes -- visible in the first line
 of `tree`. 

## Progress (2026-06-28): Phase 1 cleanup done; remaining moves scoped as deferred
### Done
- Purged 178 stale timestamped .bak edit-backups repo-wide (commit 363b2b24).
- Confirmed homes already exist + match spec: registry/, policy/, labs/.
  Phase 1 home-creation substantially complete (pre-v2 layout).
### Deferred -- and WHY (recon findings, not avoidance)
- meta/, schema/, config/ (and now runtime/) are referenced by COMPILED RUST at
  RUNTIME, not just Nix imports:
    * schema/*.json -> read at runtime by engine doctor/bootstrap.
    * meta/VERSION  -> read by BOTH Nix (hosts configuration.nix:122) AND
      faelight-release at runtime.
    * config/       -> home-manager stow SOURCE (home/christian/fsh.nix) AND
      referenced by faelight tools.
    * runtime/state.db -> hardcoded in ~15+ rust-tools (db.rs et al).
  Moving any = cross-cutting refactor (Phase 6): move dir + fix Nix imports + fix
  hardcoded paths per-tool + rebuild + verify NO runtime breakage. Understood, not
  batch-swept.
- Phase 2 (greetd isolation) is LOCKOUT-CLASS, gated by Phase 3 (tests harness),
  which is itself gated by the metal-only VM-login finding (INT-054 2026-06-28:
  cage->mango seat handoff is a QEMU artifact). Phase 2 blocked until a working
  login-test path exists.
### State
061 stays in-progress (correctly partial). v2 spec adds the nix/+faelight/ two-domain
upgrade. Do NOT cicomplete until the gated/deferred phases land.

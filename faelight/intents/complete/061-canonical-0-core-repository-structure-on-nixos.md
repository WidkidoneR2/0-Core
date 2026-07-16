---
id: 061
date: 2026-06-14
type: future
title: "Canonical 0-Core repository structure on NixOS"
status: complete
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
  **RESOLVED (2026-07-02):** `nix/home/christian/` -- convention wins per the
  lean above; authority boundary carried by the nix/modules vs nix/home split.
  Rename users/ -> nix/home/. MOVE DEFERRED to the INT-054 VM-gated session: it's
  hosts-imported (framework16/configuration.nix:155 + hosts/vm/configuration.nix +
  hosts/vm/base.nix all `import ../../users/christian/home.nix`), so relocating it
  edits boot-critical host config -- same gate as modules/ + hosts/.
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
[SUPERSEDED 2026-07-16 -- they landed. See the final section at the bottom of this file.]

## Progress (2026-07-02): Phase 6 hardcoded-path refactor -- SWEEP COMPLETE
The cross-cutting refactor flagged above as "Understood, not batch-swept" is now
DONE. This was the blocker for relocating meta/, schema/, runtime/: those dirs are
read by compiled Rust at runtime, so moving them required first making every path
resolve through ONE authority. That authority now exists and every tool uses it.

### Single path authority (faelight-core/src/paths.rs) -- the complete map
Accessors: core_dir, runtime_dir, state_db, version_file, schema_dir,
core_root_string, target_dir, logs_dir, backups_dir, checkpoints_dir, events_dir,
reactions_dir, cache_dir, capabilities_log, health_cache, forecast_cache,
reactions_config. Moving any tracked dir is now a ONE-LINE change here instead of a
per-tool hunt across the codebase. Engine AppContext core_root also derives from
paths.rs (Arch format! retired).

### Swept (18 units, ~77 refs, zero warnings, full workspace builds clean, pushed)
faelight-shell (incl. primary db.rs open), db-browse, faelight-git,
faelight-sandbox, faelight-update, faelight-ade, faelight-compositor,
faelight-contextd, faelight-idle, faelight-link, faelight-wallpaper, friday-chat,
faelight-docs, faelight-release, engine (core, ~28 refs / 3 path categories),
faelight-daemon, fsh-test. Verified LIVE: fsh restart opens real db; sandbox audit
reads real trail; core doctor 6/6 healthy; fsh-test 82/82 stored via authority.
Deliberately kept literal: fsh-test `state_db_exists` fixture (tests shell path
behavior, must NOT couple to the abstraction it may validate). Foundation commit
ada8672d ... final fsh-test commit e209e8a9.

### What this DID and DID NOT do (honest scope)
DID: removed the hardcoded-path blocker; paths.rs is now the single source of truth.
DID NOT: move any directory. The tree is still in the CURRENT layout, NOT yet the
v2 nix/+faelight/ structure. The sweep is the ENABLER; the restructure is the act.

### REMAINING for 061 (the tree restructure -- lockout-adjacent)
- Phases 1-5 dir homes + nix/+faelight/ top split (per the re-sequenced plan above).
- Phase 6 dir MOVES: git mv meta/, schema/, runtime/ into v2 homes; update the
  ONE-LINE paths.rs roots + flake.nix imports; rebuild; verify no runtime breakage.
- Hard rule still applies: VM-proof before metal; nixos-rebuild test before switch;
  TTY rescue confirmed (Ctrl+Alt+F2 out / F1 back). Phase 2 still gated by the
  metal-only login-test finding (INT-054).

### Pending (non-blocking) + follow-up ideas to file as their own intents
- Deploy routed faelight-daemon: next `rebuild` restarts the live service onto the
  swept binary -- worth watching the restart.
- IDEA (file as intent): `verify-structure` command -- assert paths.rs accessors
  resolve to real existing paths, assert NO new hardcoded paths exist outside
  paths.rs (makes the sweep SELF-ENFORCING against future drift), and once
  restructured, assert the tree matches this v2 charter. "Structure knows itself."
- IDEA (file as intent): build-output severity classifier (HIGH=compile errors,
  MEDIUM=env/dep e.g. pkg-config, LOW=warnings) to make scary output readable.
- IDEA: check faelight-contextd vs faelight-context naming/duplication.

## Progress (2026-07-02b): restructure STARTED + sweep was INCOMPLETE (honest correction)
Correction to the "SWEEP COMPLETE" claim above: it was PREMATURE. The original
sweep caught state.db / VERSION / runtime-subdir paths, but MISSED an entire class
-- the Arch-era numbered paths (00-meta/, 01-registry/, 03-interfaces/). The
restructure recon surfaced this: moving policy/ led to auditing tool references,
which revealed the gap. "Demonstrated not declared" caught what a premature done
would have buried.

### Restructure -- STARTED (recoverable Faelight-half moves)
- Created faelight/ domain. Added paths::faelight_dir() helper (core_dir().join("faelight")).
- Moved policy/ -> faelight/policy/ (repointed rules_dir() through faelight_dir()).
  Proof-of-pattern: git mv + one accessor repoint, ZERO tool source changes (the
  sweep's payoff). Verified: sandbox policy-list works, faelight-core builds.

### Second-wave sweep fixes (the missed numbered-path class -- NOW eliminated)
- faelight-sandbox: policy loader read dead 01-registry/sandbox-policies.toml ->
  routed to registry_dir(). Was BROKEN since migration; policy-list now lists 5.
- faelight-shell / faelight-release / teach: 7 stale 01-registry/tools.toml,
  00-meta/CHANGELOG.md, 01-registry/shell-patterns.toml refs -> tools_registry(),
  changelog_file(), registry_dir(). faelight-docs stale zshrc ref noted (stow class).
- faelight-link: REMOVED (dead weight). Rust GNU-Stow reimpl superseded by
  home-manager (Nix-native config symlinking; ~/.config/* -> nix store confirmed).
  Broken since migration (dead 03-interfaces/stow paths). Removed tool dir +
  registry entry + engine deploy-list entry + dead paths.rs accessor + doc refs.

### VERIFIED sweep-complete (this time by exhaustive grep, not claim)
Full-repo audit `grep -rEn "0-core/[0-9][0-9]-[a-z]" rust-tools/ engine/` -> EMPTY.
Both path classes (state.db/VERSION/runtime AND Arch-era numbered) grepped to zero.
Full workspace builds clean (33 tools now, was 34). Health 100% (tool count honest).

### RESTRUCTURE still remaining (the bulk -- unchanged from above)
[ALL DONE as of 2026-07-16. This list is kept as the record of what was outstanding.]
- Move remaining Faelight dirs -> faelight/: registry, meta, schema, runtime,
  intents, engine, rust-tools (data dirs = accessor repoint + git mv; code dirs
  also need Cargo.toml members path updates).
- Then nix/ half: profiles, modules, hosts -> LOCKOUT-CLASS, VM-proof required,
  Phase 2 still gated by INT-054 login-test finding.
- labs/ -> stays (or the docs/labs top-level, per v2 tree).

## FINAL (2026-07-16): 061 IS DONE. Every phase. Measured, not claimed.

This file has been wrong in BOTH directions, which is why it needed auditing rather than
trusting. It UNDER-reported (claiming the tree was "still in the CURRENT layout" long after
faelight/ and nix/ were real, and calling the Phase 3 harness blocked when it exists and
passes) and it OVER-reported (calling Phase 1 "substantially complete" when nix/profiles/ --
the charter's LAYER 2 -- had never been created at all).

### Phase-by-phase, as measured on 2026-07-16

Phase 1 -- domains + homes. DONE. faelight/ holds engine, rust-tools, packages, registry,
  policy, intents, runtime, meta, schema. nix/ holds modules, hosts, home, tests. home/ vs
  users/ resolved to nix/home/christian/ and the move happened.
  THE GAP WAS profiles/. It did not exist. Built 2026-07-16 (commit eaba44f2):
  nix/profiles/base.nix, holding FOUR settings and only four, because only four were
  measured duplicated byte-for-byte across hosts (experimental-features,
  auto-optimise-store, max-jobs, cores -- framework16:31/35/36/37 = vm/base.nix:39/43/44/45).
  DELIBERATELY NOT BUILT: desktop.nix / laptop.nix / development.nix / security.nix. The
  charter names them. Each would have exactly ONE consumer today -- one laptop, one VM, one
  rescue image. A profile with one consumer is ceremony, not structure. They get built when
  a second machine needs them, which is when they start being true. The charter was written
  for a fleet that does not exist yet.

Phase 2 -- greetd isolation. DONE ON METAL, commit 328b2e4c. LOCKOUT-CLASS, and it was
  gated exactly as the hard rule demands: nix flake check first, then dep.
  BUILT BECAUSE THE DRIFT WAS MEASURED, not because the charter said so.
  hosts/vm/login-mirror.nix:4 claimed "Exact replica of hosts/framework16's login (line
  108)". It was not: its tuigreet --theme said button=lightmagenta where metal said
  button=white. The VM built to MIRROR metal's login was testing a different greeter than
  metal runs. Nobody chose that -- it is what two hand-maintained copies of one string do.
  (It also said "line 108"; the block had moved to 122.)
  nix/modules/desktop/greetd.nix now owns the service, the tuigreet command and SafeShell.
  Metal's button=white won. PROVEN BY EVAL: framework16 and faelight-vm produce the
  byte-identical tuigreet command from the real flake. SafeShell verified present on both
  framework16 and faelight-vm-regreet.
  AND THE METAL DEPLOY WAS A PROVABLE NO-OP: the running greetd.toml store path
  (260hc277bwfsh...) was UNCHANGED after dep. Same derivation -> same store path -> greetd
  never restarted. The login layer was untouched while its definition collapsed from three
  drifting copies into one module.

Phase 3 -- tests harness. DONE, and the file above was wrong to call it blocked.
  nix/tests/framework16.nix exists, is wired into flake.nix checks, imports the REAL
  hosts/framework16/configuration.nix, and passes. It was never blocked by the INT-054
  login-test finding -- it asserts greetd.service is configured, not that a compositor
  gets a seat, which is the part the VM genuinely cannot do.
  BUT IT WAS NOT ACTUALLY GUARDING ANYTHING, and that was the real find: `nix flake check`
  died on the rustfmt hook before ever reaching it (see INT-119). Repaired 2026-07-16,
  commit d9b9b4d7 -- the hook had never been installed into .git/hooks, so ~30 commits that
  day alone landed unchecked, INCLUDING the lockout-class lanzaboote change (INT-161).
  Also cleaned: nix/tests/ had FOUR files and checks had TWO entries. boot.nix booted an
  EMPTY machine (`nodes.machine = _: {}`) and asserted that echo echoes -- it could not fail
  for any reason connected to this repo. friday-service.nix imported
  ../modules/services/friday.nix, WHICH DOES NOT EXIST. Both deleted (fb7d0bee); archived/
  deleted too (48f73dda -- its script hardcoded an Arch-era ~/.cargo path that does not
  exist on NixOS, so it could not run even if called; git is the archive). nix/tests/ is now
  one file that is actually run.

Phase 4 -- system/user re-scoping. ALREADY DONE. nix/home/christian/ holds
  faelight-bar.nix and faelight-notify.nix as systemd.user.services, imported by home.nix.
  Decision #5's authority boundary is satisfied. Nobody had recorded it.
  NOTE on the charter's modules/forest/friday.nix "SYSTEM service": there is no
  friday.service on this machine (`systemctl status friday` -> Unit could not be found) and
  no friday reference in framework16's config. Friday today is an ENGINE invoked by tools,
  not a daemon. The persistent daemon is INT-039 (friday-daemon), still [planned]. The
  charter wrote the destination as though it were the present -- the same error this file
  made everywhere else. modules/forest/ gets built when 039 gives it something to hold.

Phase 5 -- packages. DONE. faelight/packages/ exists; nix/modules/forest wiring is not
  needed while there is nothing in forest/ to wire.

Phase 6 -- hardcoded-path sweep + moves. DONE, and verified by exhaustive grep rather than
  claim (see the 2026-07-02b section above). paths.rs is the single authority.

### modules/system/ and modules/security/ -- deliberately NOT built
The charter wants nix/modules/system/{boot,networking,locale}.nix and
nix/modules/security/{luks,firewall,hardening}.nix. Same reasoning as the unbuilt profiles:
those settings live in framework16's configuration.nix and have exactly one consumer. The
charter's decision #7 justified system/ so "services never becomes a junk drawer" -- but
nix/modules/services/ holds one file (atticd.nix), so there is no junk drawer to prevent
yet. Splitting a 194-line host config into six single-consumer modules would make the tree
look more architectural while making nothing more true. THE CHARTER'S OWN RULE cuts this
way: "If you cannot point at the layer in the tree, it is not 0-Core -- it is just files."
Layers you can point at: nix/ vs faelight/ (the dependency seam), profiles/ (LAYER 2),
modules/desktop/ (four compositor+login modules, all multi-consumer), hosts/, home/, tests/.
That is the philosophy made visible. The rest is filing.

### The honest scorecard
DONE: the two-domain split, paths.rs authority, faelight/ moves, nix/home, greetd isolation
  on metal, the VM harness (now actually running), profiles/ LAYER 2.
NOT DONE, ON PURPOSE, EACH WITH A REASON: four profiles with one consumer each; modules/
  system+security+forest with one consumer each or nothing to hold; the tests/ subdirectory
  mirror (one test file does not need four directories).
The tree now IS the philosophy. Where it is flat, it is flat because flat is true.

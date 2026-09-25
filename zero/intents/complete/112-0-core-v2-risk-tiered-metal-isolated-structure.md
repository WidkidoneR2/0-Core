---
id: 112
date: 2026-07-02
type: future
title: "0-Core v2 risk-tiered metal-isolated structure"
status: complete
tags: [structure, restructure, lockout, lanzaboote]
priority: high
---

## Vision
Evolve 0-Core's structure from the INT-061 two-domain charter (faelight/ + nix/)
to a RISK-TIERED, metal-isolated tree that communicates operational risk at a
glance and lets tooling ENFORCE safety gates. Organize by stability and
boot-criticality, not by feature -- the arrangement that minimizes lockout risk.
Done FROM STRENGTH: after the VM harness exists (it does, INT-061) and the whole
restructure can be VM-proven before touching metal.

## Builds ON, does not redo, INT-061
061 (complete) migrated everything into faelight/ (platform) and nix/ (OS),
VM-proven, cold-boot validated. 112 is the next evolution: add risk isolation
on top of that clean foundation. NOT a second blind move -- a deliberate,
harness-proven restructure designed with the known structure-changers baked in.

## The core insight (from external design input, 2026-07-02)
Organize by STABILITY / boot-criticality, not feature. Four trust levels:
- CRITICAL: flake.nix, hardware, bootloader, initrd, disk layout, SSH rescue,
  users(account), sudo, networking, secure-boot keys. Rarely moved. HIGH lockout.
- SYSTEM: services, desktop modules, kernel, drivers, fonts. Move with testing.
- USER: home-manager, shell, editor configs. Move freely. LOW lockout.
- DATA: docs, labs, assets, runtime, experiments. Move anytime. NONE.

## Proposed shape (refine at design time)
An isolated metal/ (or critical/) peer holding the lockout-class layer:
  metal/ { boot/ disks/ hardware/ users-account/ secure-boot/ rescue/ }
kept SEPARATE from nix/ (system+user config) and faelight/ (platform). The point:
metal/ changes very slowly and is the ONLY place a lockout can originate, so it
gets the strictest gates.

## RISK.toml + enforcement (the standout idea)
Per-directory risk metadata that TOOLING reads and enforces:
  # metal/boot/RISK.toml
  risk = "critical"
  requires = ["vm-test", "boot-test"]
  review = true
A git pre-commit hook reads RISK.toml: if a critical dir changed, it BLOCKS the
commit until the VM harness (INT-061) passes. Structure stops merely COMMUNICATING
risk and starts ENFORCING it. Demonstrated-not-declared, made structural.

## Must bake in the known structure-changers (decide once, restructure once)
- INT-059 (lanzaboote secure boot): REPLACES systemd-boot (framework16 currently
  boot.loader.systemd-boot.enable=true, UEFI, disko+LUKS). Needs a keys/signed-boot
  home. The metal/boot layer MUST have a lanzaboote-shaped slot so 059 slots in
  instead of restructuring the boot layer a second time. THIS is why "decide
  structure once" matters most.
- INT-086/087 (remove pinnacle -> miracle-wm): modules/desktop must be
  compositor-SWAP-ready, not hardcoded.
- INT-039 (friday-daemon / fridayd): needs a services home for the persistent
  daemon. (fridayd idea folds here / into 039.)

## Why now-ish (sequencing)
The VM harness (INT-061) is the precondition: it lets the ENTIRE v2 restructure
be simulated in a VM (move all lockout-class dirs, boot, verify login) and then
applied to metal in ONE proven move -- not a one-dir-at-a-time crawl. Design v2
with 059/086/087/039 known, VM-prove the whole tree, single metal application.

## Gates (when built)
- [x] Target v2 tree decided (metal-isolated, risk-tiered) WITH 059/086/087/039 slots
<!-- evidence: 2026-07-16. DECIDED: the metal/ tree is NOT being built, and the reason is the
same one INT-061 used an hour earlier to decline four single-consumer profiles.
This intent proposed metal/{boot,disks,hardware,users-account,secure-boot,rescue} as an
isolated peer. Measured against the actual repo, that is SIX DIRECTORIES HOLDING ONE HOST'S
FILES UNDER A DIFFERENT NAME:
    boot/           one consumer (framework16)
    disks/          one consumer (disko.nix)
    hardware/       one consumer (hardware-configuration.nix)
    users-account/  one consumer (users.users.christian, 5 lines)
    secure-boot/    NOTHING TO HOLD -- see gate 6. lanzaboote is four lines in
                    configuration.nix and its PKI lives at /var/lib/sbctl, outside the repo.
    rescue/         ALREADY EXISTS as nix/hosts/rescue/.
THE INSIGHT SURVIVES WITHOUT THE MOVE: THE RISK TIER IS METADATA, NOT A PATH. RISK.toml
communicates and enforces risk wherever the files already live. Moving them would make the
tree look more architectural while making nothing more true -- and it would burn the whole
lockout budget on a rename.
The 059/086/087/039 "slots" this gate demanded are all resolved without a slot:
  059/161 lanzaboote -- four lines in framework16. Landed on metal 2026-07-16 (f0d0a08e),
    Secure Boot enforcing with custom keys, and it needed no structural home at all.
  086/087 compositor swap -- nix/modules/desktop/ already holds mango, pinnacle, miracle and
    greetd as four peer modules, each toggled by an option. Already swap-ready. Proven:
    framework16:187-189 enables three of them independently.
  039 friday-daemon -- STILL PLANNED. There is no friday.service (systemctl: "Unit could not
    be found"). A services home for a daemon that does not exist is exactly the error the 061
    charter made with modules/forest/friday.nix: writing the destination as though it were
    the present. It gets a home when it exists. -->
- [x] RISK.toml schema defined + per-critical-dir metadata written
<!-- evidence: commit b7342957, 2026-07-16. NINE files, not just the critical dirs -- the map
is worth more than the gate. Schema: risk (critical|system|user|data), requires (checks to run,
critical only), reason (prose, measured).
    critical  nix/hosts/framework16/   boot chain, disko/LUKS, lanzaboote+SecureBoot, account
              nix/modules/desktop/     greetd.nix -- login is lockout-class
    system    nix/profiles/            shared, but nix.settings only: breaks builds, not boots
              nix/hosts/vm/            the proving ground. NOT critical ON PURPOSE -- breaking
                                       the VM costs a test run, not a laptop
              nix/modules/services/    atticd: degrades builds, does not stop boot
    user      nix/home/christian/      home-manager
              faelight/                the platform half -- the dependency seam runs ONE WAY,
                                       so nothing here can break boot or login
    data      docs/ labs/              nothing reads them at boot
TWO CRITICAL OUT OF NINE, and that is the design. A tier that contains everything is not a
tier. If every dir were critical, every commit would pay a VM boot, and by the fifth one
anyone reaches for --no-verify -- which is precisely how INT-113 and INT-119 died.
faelight/ is user-tier BY MEASUREMENT, not assumption: `getent passwd christian` ->
/run/current-system/sw/bin/bash, and `nix eval ...users.christian.shell.name` ->
bash-interactive-5.3p9. fsh is the TERMINAL, not the login. A bad fsh build costs a prompt,
not a machine. Both nix/home/christian/RISK.toml and faelight/RISK.toml record that if
`shell = pkgs.faelight-shell` ever appears, both become critical. -->
- [x] git pre-commit hook enforces RISK.toml (critical change -> harness must pass)
<!-- evidence: commit c97d40f7, 2026-07-16. nix/lib/risk-gate.sh + a let-bound riskGate in
flake.nix, wired as a git-hooks.nix pre-commit hook. Walks UP from each staged path to the
nearest RISK.toml (nearest wins), and if it reads risk = "critical", runs everything in
requires. Blocks the commit on failure.
THIS GATE WAS IMPOSSIBLE UNTIL ELEVEN HOURS AGO. It has sat here since 2026-07-02 demanding a
pre-commit hook, and there WAS no pre-commit hook: flake.nix:284 claimed rustfmt was
"unskippable" while .git/hooks/pre-commit did not exist. INT-119 shipped the same defect
INT-113 was retired for. Repaired 2026-07-16 (d9b9b4d7) -- one missing line, sourcing
pre-commit-check.shellHook into the devShell.
PROVEN BOTH DIRECTIONS, BY EYE, NOT BY STOPWATCH:
    bash nix/lib/risk-gate.sh nix/hosts/framework16/RISK.toml
      -> "RISK GATE (INT-112): critical-tier files staged."
      -> "-> running check: framework16-boot"
      -> "ok: framework16-boot passed"
    bash nix/lib/risk-gate.sh faelight/RISK.toml docs/RISK.toml
      -> silent. Exit 0. No cost.
And in real commits: a flake.nix + nix/lib/ commit took 218ms (silent, nothing above them);
a framework16/RISK.toml commit took 83.4s (it booted the VM first).
TWO REAL BUGS, both found by RUNNING it rather than reasoning about it:
  1. `undefined variable riskGate` -- I defined it inside the outputs attrset. Nix attrsets
     are not recursive; the hook could not see a name beside it. It belongs in the let.
  2. `line 31: needed: unbound variable` -- writeShellApplication runs under `set -euo
     pipefail`, and under `set -u` bash treats a declared-but-EMPTY associative array as
     UNSET. ${#needed[@]} aborted before any logic ran, so BOTH tests failed identically and
     neither told us anything. A plain string has no such trap. -->
- [x] Whole restructure VM-simulated (boots + login) before metal
<!-- evidence: 2026-07-16. NOT APPLICABLE, and that is a result, not a dodge: there is no
restructure. Gate 1 decided the metal/ move is not happening, because the risk tier is
metadata rather than a path. No directory moved, so there is nothing to simulate.
What DID get VM-proven the same evening, under exactly this intent's hard rule: INT-061's
greetd isolation -- a genuine lockout-class change to the login layer, `nix flake check`
green before metal, and the deploy provably a NO-OP (the running greetd.toml store path was
unchanged, so greetd never restarted).
The harness this gate depends on is real and, since d9b9b4d7, actually runs.
AND THE POST-RESTRUCTURE CHECKLIST BELOW STAYS. It was INT-110's only real content (110 is
correctly cancelled -- verified verbatim identical, 962 chars both files). No dirs moved
here, so it did not fire. It applies to the NEXT move, whenever one is justified. -->
- [x] Single VM-proven metal application; cold-boot validated at 100%
<!-- evidence: 2026-07-16. NOT APPLICABLE for the same reason as the gate above: no
restructure, nothing to apply. Recorded rather than ticked green on a technicality.
The RISK.toml files are INERT -- no module imports them, no derivation reads them, nix flake
check does not see them. They cost nothing at runtime and cannot affect a boot. The hook
that reads them can only ever refuse a commit; it never rebuilds, never touches /boot, never
runs switch.
Metal state at the end of this intent, unchanged by it: gen 385, health 91%, Secure Boot
enabled (user), Measured UKI yes. -->
- [x] lanzaboote (059) slot verified to accept secure-boot without re-restructure
<!-- evidence: INT-161, 2026-07-16, commit f0d0a08e. This gate was written 2026-07-02 as a
DESIGN QUESTION -- "make sure the metal/boot layer has a lanzaboote-shaped slot so 059 slots
in instead of restructuring the boot layer a second time." It is now MEASURED, and the answer
is that no slot was ever needed.
Lanzaboote's ENTIRE repo footprint:
    flake.nix          one input (github:nix-community/lanzaboote/v1.0.0)
    configuration.nix  one module import
                       boot.loader.systemd-boot.enable = lib.mkForce false;
                       boot.lanzaboote = { enable; pkiBundle; configurationLimit; }
    THE PKI IS NOT IN THE REPO AT ALL. It lives at /var/lib/sbctl. Private keys must never be
    committed -- GitHub is public, and publishing db.key means anyone can sign a binary this
    firmware will boot.
Four lines and a path outside the tree. This intent imagined metal/secure-boot/ needing a
home; it needed four lines. The only repo artifact is Framework's PUBLIC factory certs at
nix/hosts/framework16/secureboot-factory/ (X.509 certs, nothing secret, legitimate in a repo)
and they sit inside the host dir where they belong.
"Without re-restructure" is satisfied absolutely: Secure Boot is LIVE and enforcing on metal
with custom keys and zero Microsoft, and the structure did not move a single file to get
there. -->

## Related
- Foundation: INT-061 (complete). Harness: INT-061 VM boot gate.
- Structure-changers to bake in: INT-059, INT-086, INT-087, INT-039.

---


## POST-RESTRUCTURE CHECKLIST -- fsh-test path debt (added 2026-07-07)
Any directory move in this restructure WILL silently break fsh-test, which hardcodes
repo paths in its assertions (faelight/rust-tools/fsh-test/src/main.rs). Precedent:
INT-061's restructure moved dirs under faelight/ and left 17 fsh-test failures with
stale pre-061 paths (rust-tools, engine, intents, runtime, pkgs->packages) -- found
only when the suite was run much later.

After ANY dir move here:
1. Update fsh-test path references AND top-level-structure expectations (e.g. a test
   doing `ls ~/0-core` expecting a dir that moved must expect the new top-level name).
2. Rebuild: nix develop ~/0-core#faelight-forest -c cargo build -p fsh-test
3. DEPLOY -- the `fsh-test` command runs the Nix-DEPLOYED binary, not target/debug.
   A cargo build alone shows green while the live command still fails. Must `dep`.
4. Confirm 82/82 on the deployed binary before considering the move done.

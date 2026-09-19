---
id: 253
date: 2026-09-19
type: future
title: "the doctor still tells you to run nixos-rebuild -- 85 NixOS leftovers across 21 files, and the ones that give advice are the urgent half"
status: in-progress
tags: [NixOS, deadcode, novashell, nova]
---

## Vision

Nothing on this machine advises a command that cannot run here, and no branch tests for a system
that was wiped. What remains of NixOS is what should remain: the record of what it taught.

## The Problem

Measured 2026-09-19: **85 occurrences across 21 live files** (completed intents excluded). They
are NOT one thing, and treating them as one is how a cleanup breaks something.

### ⚠️ GROUP 1 -- WRONG ADVICE THE USER ACTUALLY READS. The urgent half.

```text
    checks.rs:222    "Add the package to configuration.nix, or: nix profile install nixpkgs#<pkg>"
    checks.rs:811    "Enable firewall: networking.firewall.enable = true in configuration.nix"
    checks.rs:968    "Run: update  (or: sudo nixos-rebuild switch)"
    (friday/mod.rs   RECLASSIFIED 2026-09-19 -- see below. It is a TRANSLATION TABLE, group 3.)
    toolgen.rs:375   GENERATES documentation saying "deploy # sudo nixos-rebuild switch --flake"
```

⭐ A HEALTH CHECK THAT TELLS YOU TO RUN `nixos-rebuild` ON ARCH IS WORSE THAN A CHECK THAT SAYS
NOTHING. It is a red line with a recovery step that cannot be taken, which is the INT-199 shape
inverted: the finding is real, the fix is fiction. `toolgen.rs` is the worst of them because it
MANUFACTURES the wrong advice into generated docs on every run.

### GROUP 2 -- DEAD BRANCHES THAT CAN NEVER FIRE

```text
    faelight-update/pip_checker.rs   if Path::new("/etc/NIXOS").exists()   x2 -- never true here
    engine/domains/nix/mod.rs        an entire NIX DOMAIN
    novashell/commands/mod.rs  (27)  nix-store -q, nixos-rebuild list-generations --json
    novashell/prompt.rs         (4)  flake.nix detection in the prompt
    novashell/completion.rs     (3)  completes flake.nix, parses nixpkgs.lib.nixosSystem
    novashell/platform.rs            id.starts_with("/nix/store/")
    novashell/config.rs         (2)  home-manager symlink reasoning
    engine/bootstrap, doctor/mod    "deployed by home-manager"
```

Harmless at runtime, and confusing to read. ⚠️ BUT `domains/nix/` IS NOT A DELETION, IT IS A
DECISION -- the same whole-domain question as the 57-domain split, and it belongs with that work
rather than with a string sweep.

### ⭐ GROUP 3 -- HISTORICAL COMMENTS THAT MUST STAY

```text
    zero-gate/main.rs      "whose every entry is an absolute /nix/store path"
    ship/main.rs           "THE ACT THAT NIXOS USED TO PERFORM. dep was nixos-rebuild"
    entropy.rs             "INT-116: NixOS-native -- a package's identity is its /nix/store path"
    nsh-test/main.rs       the old NSH_BIN example path
    generate-platform-census.py   names nix-store, nixos-rebuild, /nix/store
```

THESE ARE THE RECORD OF WHAT WAS FIXED AND WHY. Deleting them destroys the reasoning that earned
the current code. ⭐ AND THE CENSUS GENERATOR IS THE OPPOSITE OF A LEFTOVER: it TESTS FOR THE
ABSENCE of these assumptions. Its own comment says "Naming a tool is not assuming a capability."
It is the guard, not the thing guarded.

FRIDAY'S TRANSLATION TABLE BELONGS HERE TOO, RECLASSIFIED 2026-09-19 AFTER READING IT. The first
census called it group 1 on the strength of a grep. It is `TRANSLATIONS` in sync_knowledge_meta:
pairs whose PRIMARY is the Arch command and whose second element is the NixOS equivalent, so
Friday can answer what the Nix version of a pacman command was. Deleting the NixOS half would
remove the ability to relate the two systems, which is knowledge rather than residue.

## The Solution

```text
    1  GROUP 1, BEFORE THE 3b FLIP      advice strings say what works on THIS machine
    2  GROUP 2 dead branches            after the flip, once its week of observation is done
    3  domains/nix/                     its own decision, with the domain-split work
    4  GROUP 3                          NOT TOUCHED, and that is recorded as correct
```

⭐ WHY GROUP 1 GOES FIRST, AND IT IS NOT URGENCY: THE FLIP WEEK IS AN OBSERVATION WINDOW. INT-247
Layer 3b flips on 2026-09-24 and the days around it are spent reading `core doctor` for damage.
A doctor that advises `nixos-rebuild` while you are reading it for flip damage is NOISE IN THE
SIGNAL YOU ARE TRYING TO READ. Clean advice makes the flip easier to judge. That is the whole
argument -- the leftovers do not block the flip and never did.

## Success Criteria

- [ ] THE CENSUS IS REGENERATED AND CLASSIFIED into the three groups above before any edit. The
      85 is from 2026-09-19 and every number in this file will have drifted.
- [ ] ⭐ EVERY RECOVERY STRING AND EVERY PIECE OF ADVICE NAMES A COMMAND THAT RUNS HERE.
      **Proven by running it:** each replacement is executed on this machine and its output
      recorded in the intent. An advice string that was never run is a guess with better grammar.
- [ ] `toolgen.rs` no longer GENERATES NixOS advice. Proven by regenerating the docs and grepping
      the output, not by reading the template.
- [ ] Group 2 branches are removed and the code still compiles and passes `nsh-test`, with the
      count of `/etc/NIXOS` tests in live code at ZERO.
- [ ] `domains/nix/` is NOT deleted by this intent. Its fate is recorded as an open decision with
      the domain-split work, and the reason is stated.
- [ ] GROUP 3 IS UNTOUCHED AND THE LIST IS WRITTEN DOWN, so a later sweep does not "finish the
      job" by deleting the history. The census generator especially: it is the guard.
- [ ] Both doors run afterwards -- `nsh -c`, a PTY session, `core doctor`, `history` -- and the
      doctor's advice is read end to end by a human who confirms every fix is runnable.

## Not in scope

Renaming anything. INT-252's directory move. Any of INT-247's layers. Deleting `domains/nix/`.

## Relationship

Found while porting the doctor for INT-222 -- three of the five worst strings are in the very
checks being migrated. Sequenced against INT-247 Layer 3b for signal clarity, not dependency.

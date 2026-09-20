---
id: 255
date: 2026-09-20
type: future
title: "NovaShell thinks in Arch"
status: in-progress
tags: [nsh, novashell, arch]
---

## Vision

NovaShell asks questions this machine can answer. Not a shell with the Nix parts removed -- a
shell that thinks in Arch, where every command that reports on the system reports on THIS one.

## The Problem

⭐ THE LEFTOVERS ARE NOT ALL THE SAME THING, AND THAT IS WHY A SWEEP WOULD BE WRONG. INT-253
found this three times over: a translation table read as bad advice, a domain read as dead code,
and a pip guard read as a dead branch when its TEST was wrong and its CONCERN was live. Deleting
that one would have left the tool attempting a pip upgrade Arch blocks.

So each item here gets read, and the ruling is recorded per item.

### The commands -- LIVE, and pointed at a system that left

```text
    packages / pkgs       nix-store -q --references /run/current-system/sw
    generations / gens    nixos-rebuild list-generations --json
    store                 store_cmd: roots, referrers, closure queries
    nix_query_lines       commands/mod.rs:8275, the helper all of them use
```

⚠️ THESE ARE NOT DEAD CODE. They run, they fail to find nix-store, and they SAY SO -- INT-227
built that diagnostic on purpose: "this machine has no nix-store", with a help line and a code.
They degrade honestly, which is why they are still here.

★ BUT ARCH ANSWERS THREE OF THESE QUESTIONS. `packages` is `pacman -Q`. `generations` is snapper,
which INT-129 measured: limine-snapper-sync already puts snapshots in the BOOT MENU, and
`snapper list` is already a formatted table that needs root. `store` has no equivalent and
probably retires.

THE RULING IS PER COMMAND, and "it degrades honestly" is not a reason to keep a command that
could instead be useful.

### The prompt, completion and identity

```text
    prompt.rs      (4)  looks for flake.nix to decide what to show
    completion.rs  (3)  offers flake.nix, parses nixpkgs.lib.nixosSystem
    platform.rs         a package is identified by its /nix/store path
    config.rs      (2)  home-manager symlink reasoning
```

The prompt and completion are the ones a person SEES every day, which makes them the urgent half
here the way the advice strings were in INT-253.

### The docs

Stale for Nix, and stale for other reasons. Both get fixed, and the second is not a bonus --
a doc that is wrong about anything teaches the reader to check the code instead, which is how
documentation stops being read at all.

## The rulings, 2026-09-20 -- EACH COMMAND RUN BEFORE IT WAS JUDGED

```text
    packages    REPOINT    fails with a good diagnostic: "this machine has no nix-store".
                           pacman -Q answers exactly the question it asks. It is used.
    generations RETIRE     CRASHES. "No such file or directory" -- it spawns nixos-rebuild
                           and does not guard the spawn, so it is not honest degradation,
                           it is an unhandled error. And INT-129 already measured the
                           replacement and rejected it: limine-snapper-sync puts snapshots
                           in the BOOT MENU, which works when the system will not boot and
                           a TUI cannot, and snapper list is already a formatted table that
                           needs root. A TUI here would prompt for a password to show what
                           one command shows.
    store       RETIRE     THE WORST OF THE THREE. It prints a full help menu -- why,
                           reclaim, big -- as if it works, and only fails once a subcommand
                           runs. A tool that advertises capability it cannot deliver is
                           worse than one that errors. No Arch equivalent: closure queries
                           are a content-addressed-store idea and pacman has no counterpart.
```

★ THE STANDARD: WHEN YOU MOVE INTO A NEW HOUSE YOU CLEAN IT BEFORE YOU MOVE IN. Christian,
2026-09-20, ruling that this finishes before INT-247 Layer 3b flips.

### platform.rs KEEPS ITS NIX BRANCH -- ruled 2026-09-20

The census flagged one line: `id.starts_with("/nix/store/")`. It is a TEST, and it early-returns
unless `is_nix_deployed()`, so it no-ops here.

⭐ AND THE CODE IT GUARDS IS CORRECT PORTABLE CODE, not a NixOS assumption. Its own comment makes
the argument: the question is not "is the distro NixOS" but "does the deploy indirection exist
here". A makeWrapper-wrapped binary reports the WRAPPER, not the artifact whose hash
distinguishes one deploy from the next, so on such a system the store path is the identity.
Elsewhere there is no store and no wrapper, `current_exe()` IS the artifact, and that is the
branch this machine takes.

★ THE THIRD CORRECT GUARD THIS INTENT HAS FOUND, after the pip PEP 668 check and the
degrade-honestly diagnostics. Removing it would make the code LESS correct on a machine that
has the indirection, in exchange for deleting the word nix.

### THE MENU ALIAS LIVED IN THREE PLACES -- found 2026-09-20

Removing `faelight-logout` took three edits, in three files, with nothing linking them:

```text
    faelight/registry/aliases.toml        the DECLARATION -- documentation of the alias
    ~/.config/faelight-shell/config.nsh   the DEFINITION -- what actually creates it
    ~/.local/bin/faelight-logout          the TARGET -- a binary dated 26 August
```

⚠️ AND REMOVING TWO OF THE THREE LEFT THE SYSTEM WORSE THAN BEFORE. The registry entry and the
binary went first, so the shell still defined `menu` pointing at a command that no longer
existed. `faelight-deadwood --strict` caught it at the pre-push gate:

```text
    [MED ] alias menu -> 'faelight-logout' (target 'faelight-logout' not found)
```

⭐ THE GATE DID ITS JOB AND THAT IS THE ONLY REASON IT WAS CAUGHT. Nothing links the three
locations, so nothing could have told me the removal was partial. The registry is not the
source of truth for aliases -- config.nsh is -- and the registry describes it without being
consulted by it.

★ THE SAME SHAPE AS DECISION 149 (nine readers of tools.toml) AND INT-256 (three of
twenty-six tools calling restore_sigpipe): not carelessness, THE ABSENCE OF A PLACE WHERE THE
QUESTION GETS ASKED. Here the question is "what else knows about this alias?" and nothing does.

### Still to do, measured 2026-09-20

```text
    triage.rs              13KB classifying nixos-rebuild output. Its ONLY invoker was the
                           deploy script deleted today, so it is now unreachable by any path.
    platform.rs            one line: a package is identified by its /nix/store prefix
    faelight-logout        NOT IN tools.toml. The binary on PATH is dated 26 Aug -- the
                           migration day -- so it is a leftover from the last NixOS deploy,
                           not something ship manages. Christian confirms it was removed.
                           The package source, the binary and the `menu` alias in
                           aliases.toml all go.
    the docs               stale for Nix and stale for other reasons
```

⚠️ AND AN ALIAS POINTING AT A TOOL THAT DOES NOT EXIST IS A DEFECT NOTHING CHECKS. The doctor
has alias_coverage, which asks whether every TOOL has an alias. Nothing asks whether every
ALIAS has a tool. That is the inverse, and `menu` is proof it happens.

## Success Criteria

- [ ] THE CENSUS IS REGENERATED FIRST. Every NixOS reference in novashell/, classified per item:
      repoint / retire / keep-and-explain. No item is actioned before it is classified.
- [ ] ⭐ EVERY REPOINTED COMMAND IS RUN HERE AND ITS OUTPUT RECORDED IN THIS FILE. The same gate
      caught two lies in twenty recoveries on 2026-09-20 -- a command that was never run is a
      guess with better grammar.
- [ ] `packages` either answers from pacman or is retired, and the choice is stated with a reason.
- [ ] `generations` is ruled on against what INT-129 already measured about snapper, rather than
      re-deriving it.
- [ ] `store` is ruled on. "No Arch equivalent" is an acceptable answer IF it is written down.
- [ ] The prompt shows nothing that depends on a file this system does not use.
- [ ] Completion offers nothing that cannot exist here. **Proven by typing it**, not by reading
      the list.
- [ ] `nix_query_lines` and its callers are gone, or the survivors are named and justified.
- [ ] THE DOCS ARE CORRECT ABOUT WHAT THE SHELL DOES TODAY -- checked against the binary, not
      against the source, and stale-for-other-reasons counts as stale.
- [ ] nsh-test still green, and the suite gains a case for anything repointed.
- [ ] Nothing in INT-253's group 3 is deleted: the historical comments and the census generator
      stay, and the generator especially, because it is the guard that tests for the absence of
      these assumptions.

## Not in scope

The doctor -- INT-222 finished it. `engine/domains/nix/` -- recorded in INT-253 as a decision
belonging with the domain split. Renaming anything: that is INT-252.

## Relationship

Split out of INT-253 on 2026-09-20. That intent removed the ADVICE and the dead branches; this
one changes what the shell KNOWS. Filed separately because "remove leftovers" and "think in Arch"
are different jobs, and the second one makes decisions the first has no business making.

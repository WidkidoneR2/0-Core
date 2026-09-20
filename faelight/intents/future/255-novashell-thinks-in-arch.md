---
id: 255
date: 2026-09-20
type: future
title: "NovaShell thinks in Arch"
status: planned
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

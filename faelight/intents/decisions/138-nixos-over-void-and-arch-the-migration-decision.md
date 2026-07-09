---
id: 138
date: 2026-07-09
type: decisions
title: "NixOS over Void and Arch: the migration decision"
status: complete
tags: [nixos, arch, void, migration, friday, foundational]
---

## Recorded late, on purpose
The migration to NixOS is the largest architectural change in the forest's history, and it
had NO decision record. The lineage in decisions/ runs 035-sway-migration, 043-paru-migration,
065-source-first-build, 091-stylix, 102-version-bumping-on-nix -- the Arch era -- and then
Nix is simply THERE, unexplained. Written down 2026-07-09 from Christian's own account, so
the reasoning survives the people who held it.

## Leaving Arch
- **AUR trust.** The AUR is unvetted user-submitted build scripts. That is a supply-chain
  trust problem, not a convenience problem, and it does not get better by being careful.
- **Ceiling.** Arch was felt to be lacking opportunity -- a competent base, but not a
  platform with room to build something larger on top of.

## Why not Void
Void was seriously considered. The blocker was **community size**: a small community means
thin documentation, few packages, and long odds of support when something breaks at 3am.
Christian wanted a community that was more supportive and more populous. NixOS won on
ecosystem gravity, not on technical purity.

## Why NixOS -- the real reason
Ecosystem and security mattered. But the deciding argument was **Friday**:

> "I was looking at what would be best for Friday to learn from and to be able to grow,
> being built in both Rust and Nix, and I can open the door to other languages."

The OS was chosen for what the AI could learn from it. A declarative system is *legible*:
the whole machine's state is data, versioned, diffable, inspectable, reproducible. An
imperative package manager leaves no such trace. NixOS gives Friday a system it can read,
reason about, and eventually reason *with*. Rust and Nix as Friday's native languages, with
the door deliberately left open to more.

That is a decision about the AI's substrate, not the human's convenience -- and it is why
the forest's "understanding over convenience" principle has an OS underneath it that agrees.

## What was traded away
- Arch's AUR breadth and immediacy (accepted: trust was worth more than convenience).
- Void's runit simplicity and its small-but-real appeal (accepted: community size decided it).
- A steep learning curve, and a build system that must be understood, not merely used.
  Understanding was the point.

## Consequences visible today
- Every system change is a generation: rollback-able, diffable, cold-boot verifiable.
- The forest's tooling reads Nix as data (nix domain, faelight-nix, nix-tree, deadnix).
- Friday's knowledge layer distinguishes native (forest/Nix) from foreign (pacman/Arch)
  facts and translates between them -- INT-128. That capability only makes sense because
  of this decision.

## Relationship
Precedes and enables: INT-340 (NixOS migration execution), INT-061 (two-domain tree),
INT-128 (native/foreign knowledge), the whole nix/ half of 0-core.
Supersedes the Arch-era assumptions in decisions/043 (paru) and 065 (source-first build).

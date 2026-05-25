---
id: 340
title: "Faelight Forest NixOS -- clean foundation, fresh start, new architecture"
status: planned
date: 2026-05-25
tags: [nixos, migration, foundation, fresh-start, architecture, nix, flake, friday, rust]
---

## The Decision

On 2026-05-25, the decision was made to migrate Faelight Forest to NixOS.

This is not a migration intent. This is a rebirth intent.

The reasoning:
- Armijn Hemel (open source compliance expert, conference sponsor) specifically
  recommended NixOS and noted that core-protect fits NixOS philosophy perfectly
- NixOS's immutable store + atomic upgrades + rollback aligns with what the
  forest has been building toward manually
- Rust and NixOS have excellent synergy -- naersk/crane make Rust builds
  fully reproducible
- The security model is genuinely better -- immutable by default
- One flake.nix can describe the entire forest environment -- the Faelight
  Forest Omakub vision becomes real
- Nobody in the world has built what we are building: a Friday-powered,
  forest-native NixOS environment with nightly builds and R&D pipelines
- This is something new to learn -- growth is part of the forest philosophy

The Arch forest proved it was possible to build a 96%+ Rust system
from nothing. NixOS is where that proof becomes a product.

## The Mindset

This is a FRESH START. Not a port. Not a migration script. A deliberate
rebuild where every tool earns its place again.

Rules for the NixOS forest:
1. No Jarvis. Not a single reference. Friday only, from day one.
2. Fresh intent numbers starting from 001. Clean ledger. No ghost gates.
3. Every tool is a Nix derivation -- no mystery packages, no AUR, no
   "I think this is installed somewhere."
4. The shell (fsh) comes first. The compositor comes second. Everything
   else serves those two.
5. Friday is built into the foundation -- not added later.
6. core-protect becomes a NixOS module. Immutability is the OS, not a script.
7. The intent system, state.db, and deploy pipeline are rebuilt for NixOS
   conventions from the start.
8. Nightly builds are a first-class concept -- the forest can produce
   nightly/stable/experimental channels.

## What Comes With Us (Earned Its Place)

Tools that have proven their value and come to NixOS:
- fsh (faelight-shell) -- the soul of the forest
- faelight-compositor -- proven on real AMD hardware
- faelight-bar -- high marks from demo feedback
- faelight-git -- now with Friday integration
- faelight-deploy -- 8.5/10 feedback
- Friday intelligence layer -- event bus, reasoning engine, one-mind answer
- core-protect -- becomes a NixOS module
- faelight-term -- needs work but the foundation is right

Tools that do NOT come:
- Anything with Jarvis in the name or code
- audit-stale tools that exist only because nothing integrates the data
- Ghost intents with unverified gates

## What Gets Rebuilt Better

### Intent System v2
- Fresh intent numbers starting 001
- Gates enforced from day one (INT-332 logic built in)
- No directory mismatch possible -- NixOS structure enforces it
- Deferral requires human approval -- baked into the tool

### Friday v3 (Built for NixOS)
- No legacy Jarvis tables
- Event bus is the foundation, not an addition
- state.db is a proper NixOS service with backup/restore
- Reasoning engine runs as a systemd user service
- Friday speaks unprompted -- confidence-gated, not on every command

### fsh Semantic Architecture (INT-326)
- Three-layer execution from the start
- Human-readable verbs as canonical names
- UNIX as fallback, not primary
- Structured data pipeline built in from v1 on NixOS

### faelight-compositor v3 (Pinnacle-informed)
- Smithay foundation -- proven
- Pinnacle patterns studied (INT-337) before building
- Layer shell for bar and notifications from day one
- Forest Candy visual identity from day one
- Replaces Niri permanently

### R&D Pipeline
- VM-based sandbox (INT-328) for experimenting
- Nightly/stable/experimental channels
- Hypothesis-test-gate-graduate pipeline
- Other developers can run the forest by installing one flake

## The NixOS Learning Path

Christian has never used NixOS. The learning path:
1. Install NixOS in the VM (INT-328 -- already planned)
2. Learn Nix language basics -- derivations, flakes, home-manager
3. Port fsh to a Nix derivation -- prove the toolchain works
4. Port faelight-compositor -- the hardest dependency chain
5. Build the full flake.nix describing the forest environment
6. Run the forest on NixOS in the VM for 30 days
7. If VM proves it: migrate the Framework laptop
8. If laptop is stable: the Arch forest becomes the reference implementation,
   NixOS becomes the future

## The Conference Story (USENIX HotOS, WCRE, MSR, ASE)

The story for the conferences:
"One person, one year, from zero Linux knowledge to building a
fully reproducible, Friday-powered, NixOS-native development environment
in Rust. The forest thinks alongside you. The shell is the mind.
The compositor is the face. Friday is the nervous system.
And it ships as a single Nix flake anyone can install."

That is a story worth telling. That is what Armijn sees.

## What Stays on Arch (For Now)

The current Arch forest does not get abandoned. It continues as:
- The daily driver while NixOS is being learned
- The reference implementation
- The source of the tools being ported
- The place where new ideas are prototyped before moving to NixOS

## Gates

⬜ NixOS installed in VM (INT-328 prerequisite)
⬜ Nix language fundamentals learned -- can write a basic derivation
⬜ fsh compiles as a Nix derivation -- flake.nix proven for shell
⬜ faelight-compositor compiles as a Nix derivation
⬜ home-manager configured with forest tools
⬜ state.db runs as a NixOS user service
⬜ Friday event bus runs as a NixOS user service
⬜ Fresh intent ledger started at 001 -- NixOS forest has clean slate
⬜ core-protect implemented as a NixOS module
⬜ Forest Candy visual identity applied in NixOS compositor
⬜ fsh v4 semantic architecture running as daily driver on NixOS VM
⬜ 30-day VM trial -- forest stable on NixOS
⬜ Framework laptop migrated to NixOS
⬜ Single flake.nix describes the entire forest environment
⬜ Nightly build channel working -- forest can self-update
⬜ Conference demo: boot NixOS, run forest, show Friday reasoning in real time

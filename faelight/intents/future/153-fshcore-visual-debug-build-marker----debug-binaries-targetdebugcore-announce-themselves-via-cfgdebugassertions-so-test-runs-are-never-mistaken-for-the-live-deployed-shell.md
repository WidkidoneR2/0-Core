---
id: 153
date: 2026-07-12
type: future
title: "fsh/core: visual DEBUG-BUILD marker -- debug binaries (./target/debug/core) announce themselves via cfg!(debug_assertions) so test runs are never mistaken for the live deployed shell"
status: planned
tags: [core, debug, dx, safety, testing]
---

## Vision
When you run a debug binary (`./target/debug/core ...`) it visibly announces itself, so a test
run is never mistaken for the live, deployed shell. The forest always tells you which reality
you are standing in.

## The Problem
Throughout INT-148/151 (and every testing session) we ran `./target/debug/core doctor run` dozens
of times to test unreleased builds -- with NOTHING visually distinguishing the debug binary's output
from the real deployed `d`. A debug health check and a live health check look identical. This is a
latent footgun: easy to read a debug-binary result as authoritative, or to forget you are in a
throwaway test context. (Cousin risk surfaced live: the PATH="" test that clobbered the session --
clear "you are in a test context" signals reduce that whole class of mistake.)

Two distinct cases:
  1. Running a debug BINARY (`./target/debug/core`) -- still in real fsh, just an unreleased build.
     THIS IS THE GAP. No marker today.
  2. Dropping into a clean SHELL (`bash --noprofile --norc`) -- already handled: fsh prints
     "Stepping out of the forest... You are entering bash" and the prompt changes. Case 2 is fine;
     this intent is about Case 1.

## The Mechanism (settled)
Rust cfg!(debug_assertions) -- compile-time true in debug builds, false in release. The deployed
binary is a release build, so the marker branch is compiled OUT entirely (zero runtime cost, and
impossible to show accidentally). The debug binary intrinsically knows what it is and announces it.
The binary self-identifies; there is no runtime flag to forget or spoof.

## OPEN DESIGN QUESTIONS (decide at cistart -- do NOT assume)
1. Scope: which output shows the marker? Every command (max safety, but could pollute output that
   scripts parse via pipes)? Only interactive/human-facing output (doctor run, dashboards)? Only
   when stdout is a TTY (so piped/scripted use stays clean)? Leaning: TTY-gated banner so machine
   consumers are unaffected -- confirm.
2. Form. A one-line top banner (distinct color, e.g. a wrench glyph + "[debug build -- not
   deployed]")? A persistent prompt-level marker? Both? Should feel consistent with Case 2's
   "stepping out of the forest" aesthetic.
3. Does fsh itself need this too, or only core? fsh has its own debug/release builds
   (target/debug/faelight-shell). If you can run a debug fsh, it should announce itself the same way.
   Decide: core only, or core + fsh.

## Success Criteria
- [ ] design questions RESOLVED (scope, form, core-only-vs-core+fsh)
- [ ] debug build shows the marker -- demonstrated: ./target/debug/core doctor run displays it
- [ ] release build does NOT -- demonstrated: build release, run it, confirm NO marker (elegant
      proof: same source, two builds, only debug shows it -- cfg!(debug_assertions) compiled out)
- [ ] piped/scripted output unaffected (per Q1 -- e.g. TTY-gated so `core ... | grep` sees no banner)
- [ ] consistent with the Case 2 "stepping out of the forest" clean-shell marker (shared aesthetic)
- [ ] built, deployed; live d (release) is clean, debug binary flagged

## Relationship
Origin: user question during the INT-151 session -- "could we visually tell debug-testing from the
real shell?" A DX + safety improvement in the same "honest self-observation" family as the doctor
Status::Unknown work (INT-148): the system should always be truthful about which reality you are in.
Filter: reduces a real testing footgun at zero cost to the deployed binary. In-filter.

## Notes
- cfg!(debug_assertions) is the whole trick -- no runtime version check, no env var, no config.
  The compiler decides, and release literally cannot show it.
- Case 2 (clean-shell "stepping out of the forest") is the aesthetic precedent to match.

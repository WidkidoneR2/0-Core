---
id: 221
date: 2026-08-13
type: future
title: "fsh pick calls sk (skim) which is not installed, so every fuzzy selection fails with "sk not found" while fzf is installed on the same system"
status: in-progress
tags: [fsh, pick, fuzzy, dependency, int-134]
---

## Vision
A built feature reaches for a backend the system actually has.

## The Problem
MEASURED 2026-08-14 while ruling INT-134 gate 5:

    pick file   ->   x sk not found

`pick` is real and its subcommands are `pick intent`, `pick intent --active`, `pick history` and
`pick file [--core]`. It pipes candidates through an external fuzzy selector. That selector is `sk`
(skim), and **skim is not installed on this system**.

⚠️ AND A WORKING ALTERNATIVE IS ALREADY PRESENT: `fzf` IS installed. So the shell ships a feature
that cannot run, next to the tool that would let it.

Every subcommand is affected, not just files -- they all funnel through the same selector.

## HOW IT WAS FOUND, and why that matters
Nobody reported this. It surfaced because INT-134 gate 5 required a RULING on "fuzzy command
completion", and ruling required running the thing rather than reading about it. The roadmap had
carried a note calling `pick` intents-only, written from a partial subcommand list -- that note was
also wrong, and the same probe corrected both.

## The Solution
⚠️ THERE IS A REAL CHOICE HERE AND IT SHOULD BE MADE DELIBERATELY, not patched over:
  - DECLARE `sk` as a dependency in the Nix config, so the tool the code names is the tool present
  - USE WHICHEVER IS AVAILABLE -- probe for `sk`, fall back to `fzf` -- which tolerates both systems
    but puts two selectors in one code path
  - SWITCH TO `fzf` outright, since it is already installed and is the more widely available tool

⚠️ AND WHATEVER IS CHOSEN, THE FAILURE MESSAGE SHOULD SAY MORE THAN "sk not found". A user reading
that cannot tell whether they typed something wrong, whether the feature exists, or whether a
dependency is missing. Same class as INT-215: an accurate message that leaves the reader guessing.

## Evidence (measured 2026-08-14)
- `pick file` reports "sk not found". `pick` with no argument prints its full usage, so the builtin
  itself is fine -- the failure is at the selector.
- `shutil.which` reports sk absent and fzf PRESENT on this system.
- The subcommands are pick intent, pick intent --active, pick history, pick file [--core].

## Success Criteria
- [ ] G1 RED FIRST: the failure is reproduced by a case, not only by hand -- `pick` with a selector
      absent, asserting the message names a missing dependency rather than an unknown command
- [ ] G2: THE CHOICE IS MADE AND RECORDED with what it gives up -- declare sk as a dependency, probe
      for either, or switch to fzf. A decision, not a default
- [ ] G3: PROVEN: the chosen selector runs on this system and a fuzzy selection completes end to end
- [ ] G4: the failure message names the missing dependency and what to do about it, so a reader can
      tell a missing tool from a typo. Same class as INT-215
- [ ] G5: every subcommand is checked, not only `pick file` -- they share one selector path, so a fix
      at that path must be shown to serve all four
- [ ] G6: the fsh-test suite stays green, and a case covers the selector-absent path so this cannot
      regress silently
- [ ] G7: each gate carries evidence per INT-158

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->

---
id: 183
date: 2026-07-20
type: fix
title: "Friday fires false high-confidence hints at unrelated input (99% on irrelevant matches)"
status: planned
tags: [fsh, friday, confidence, noise]
---

## Vision
Friday's suggestions are trustworthy: when Friday speaks with high confidence, it is because the
suggestion is actually relevant -- not because a keyword matched. A hint the human learns to
ignore is worse than no hint.

## The Problem
OBSERVED repeatedly 2026-07-20 (a whole session of sightings, and INT-132 territory): Friday
fires 99%-confidence hints at input they do not apply to. Concrete examples from this session:
- `core intent cancel 180` (a missing --reason FLAG error) -> Friday: "Enum needs
  #[derive(Debug, Clone, Subcommand)]..." (a Rust clap hint, 99%) -- totally unrelated to a shell
  usage error.
- Repeated failed commands -> Friday surfaces high-confidence hints keyed off surface tokens, not
  actual relevance.
The pattern: confidence appears keyed to KEYWORD/PATTERN MATCH, not to whether the suggestion
fits the situation. 99% displayed next to an irrelevant hint trains the human to distrust ALL of
Friday's confidence numbers -- the noise poisons the signal.

## The Solution
Recon FIRST (this is Friday's core -- do not guess): find where Friday computes the confidence it
displays, and what that number is actually measuring. Likely findings to check:
- Is "confidence" the pattern's stored confidence (how sure Friday is the PATTERN is good),
  displayed as if it were RELEVANCE (how sure this pattern applies HERE)? Those are different, and
  conflating them is the classic bug.
- Does the match gate on context (command domain, error type) or only on token overlap?
The fix direction (to be confirmed by recon, not assumed): separate PATTERN confidence from
MATCH relevance, and gate display on relevance -- a rock-solid pattern that does not apply here
should not fire, regardless of its stored confidence. This may be scoped with INT-132.

## Success Criteria
- [ ] Recon: locate where Friday computes and displays hint confidence, and what the number
      measures (pattern-confidence vs match-relevance). Documented before any change.
- [ ] The two false-positives from 2026-07-20 (clap-hint on the cancel --reason error; hints on
      unrelated failed commands) are REPRODUCED, so the fix has a target.
- [ ] Relevance gating added: a hint fires only when it actually applies, not on token match
      alone. The reproduced false-positives no longer fire.
- [ ] Real hints still fire (no over-correction that silences Friday). A known-good hint still
      appears -- proven.
- [ ] Relationship to INT-132 named (this may be part of it or a precursor).
- [ ] Each gate carries evidence per INT-158.

## The Rule
"A confidence number the human learns to ignore is a lie with a percentage on it. Friday should
be sure it FITS before it is sure at all." 🌲

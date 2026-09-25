---
id: 175
date: 2026-07-18
type: future
title: "Finish the event bus."
status: cancelled
tags: [bus, fsh, faelight-shell]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->

## Gate Check
🚫 175 -- cancelled: faelight-daemon (the event bus this refers to) is Arch-era prototype code, untouched since the NixOS migration -- it was the prototype for friday-daemon (INT-039). 'Finish the event bus' had a false premise: there is no NixOS event bus to finish, only a dead Arch relic. Overtaken by the migration (cf INT-159 precedent). If a live event bus is wanted on NixOS later, it belongs with friday-daemon (039), built native -- with faelight-daemon as salvage/reference, not the thing to finish. Deferred to that decision. -- approved by: christian 2026-07-20

---
id: 210
date: 2026-08-08
type: arch
title: "inventory and classify the remaining quote-state scanners in expandrs -- three machines with three stated purposes, and whether each is lexical interpretation or a consumer of lexical facts decides whether it can be consolidated"
status: planned
tags: [architecture, rust, design]
---

## Vision
Know what each remaining quote-state scanner IS before deciding whether it can move. The answer may
be that some of them should not.

## The Problem
INT-209 removed strip_comments, a 55-line machine that tracked heredoc state and walked characters
with its own quote pair. It was one owner of several. expand.rs still holds at least three more, and
they are NOT obviously the same problem:

  ~240-258  a validity checker, with in_s/in_d AND a second in_s2/in_d2 pair for bracket depth
  ~346-356  needle scanning -- find a substring that is not inside quotes
  ~433-439  expansion gating, whose own comment records that double quotes still permit command
            substitution so only in_single gates it

Three stated purposes. Treating them as one refactor because they share two boolean names is how a
controlled architectural change becomes an uncontrolled rewrite.

## The Solution
Classify first, consolidate second -- and only what the classification says can move.

The question for each: is this LEXICAL INTERPRETATION, which the canonical scanner should own, or is
it a CONSUMER OF LEXICAL FACTS serving a narrower semantic operation? A consumer asking "is this
offset inside quotes?" is not a second lexer; it is a caller that would be better served by the
scanner exposing that fact. A machine that decides what a quote MEANS is a second lexer and must go.

⚠️ THE EXPANSION-GATING ONE IS THE INTERESTING CASE. Its rule -- double quotes allow substitution,
single quotes do not -- is semantics, not lexing. The scanner already records QuoteContext per
segment, so this may be a consumer that should read that fact rather than rediscover it. That would
make it a consumer migration rather than a deletion.

## Explicitly out of scope
Moving anything. This intent produces a classification anyone can act on. If it starts growing
implementation, it has failed -- the same fence INT-198 set, for the same reason.

## Success Criteria
- [ ] Each of the three machines is classified: lexical interpretation, or consumer of lexical facts
- [ ] For each, the evidence is its stated PURPOSE and its inputs, not its variable names
- [ ] Any that is a consumer names the fact it needs and whether the scanner already records it
      -- QuoteContext exists per segment today
- [ ] Any that is a second lexer gets a lane and a rough order, or an explicit deferral with reason
- [ ] A grep-able statement of how many quote-state machines remain outside the scanner, so
      INT-209's sole-owner gate has a countable target rather than a belief
- [ ] Nothing moved. The close condition is a decision, not a diff
- [ ] Each gate carries evidence per INT-158
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

---
id: 132
date: 2026-07-08
type: future
title: "Friday teaching path: bidirectional knowledge feedback loop"
status: planned
tags: [friday, teaching, knowledge, feedback, translation, native-foreign]
---

## Vision
A teaching path where Christian and Friday improve each other's knowledge over time.
Christian corrects and extends Friday's facts; Friday surfaces the right native
knowledge and translations back to him; outcomes close the loop so it sharpens with
use. Built directly on INT-128's native/foreign/translation data layer.

## The Problem
INT-128 gave Friday's knowledge facts native/foreign/translation metadata (system,
kind, translates_to in friday_knowledge_meta), self-healing via sync_knowledge_meta.
But nothing CONSUMES it yet. The labels exist; no behaviour reads them. Friday cannot
yet teach native-as-"this-system", cannot translate a foreign command from a data row,
and there is no path for Christian to teach knowledge back into the forest.

## The Solution
A bidirectional feedback loop with three flows:

1. Christian -> Friday (teach): add / correct / confirm a fact or translation. INT-128's
   sync_knowledge_meta auto-labels it native/foreign. Fix a wrong translation, teach a
   new native fact, confirm a good one.

2. Friday -> Christian (translate / surface): type a foreign command (e.g. `pacman -Syu`)
   -> Friday recognises it via the kind='translation' row -> teaches the native way from
   translates_to. Native facts surfaced as "this system"; foreign ones translated, not
   parroted.

3. The loop closes (outcome): tell Friday whether its answer helped -> confidence
   adjusts -> Friday learns which knowledge actually serves the work.

## First piece
A shared primitive: translate_foreign(term) -> Option<native>, reading friday_knowledge_meta.
BOTH the existing query-answer path (friday/mod.rs ~498) AND a future `friday translate`
command call it. Build the primitive once; multiple consumers. (This is the "between A
and B" seam: not bolted only into the existing path, not a whole new command surface --
a reusable core that both use.)

## Honesty -- what Friday is
Asymmetric. Friday observes, records, and surfaces -- it is not a co-discovering peer.
The loop is real (Christian improves its knowledge; it improves his recall and teaches
him the native system), but it is a tool learning from corrections and serving them back,
consistent with "understanding over convenience". No pretense of two equal minds. The
value is a knowledge partner that gets measurably better with use, not a second mind.

## Success Criteria (gates -- refine at cistart)
- [ ] translate_foreign() primitive reads friday_knowledge_meta, returns the native way for a foreign term
- [ ] Friday teaches only native (nixos) facts as "this system"; foreign facts not surfaced as native   <!-- rehomed from INT-128 gate 4 -->
- [ ] Friday recognises a foreign command and translates it via a DATA row, not a hardcoded string        <!-- rehomed from INT-128 gate 5 -->
- [ ] a new fact/translation added via db write is live with NO core rebuild -- demonstrated               <!-- rehomed from INT-128 gate 6, the "learnable" proof -->
- [ ] Christian -> Friday teach/correct flow writes a fact, auto-labeled by INT-128's sync
- [ ] outcome yes/no adjusts confidence -- the loop closes and is observable

## Relationship
- Builds on: INT-128 (complete) -- the native/foreign/translation data layer this consumes.
- Rehomes the behavioural gates 4-6 that INT-128 scoped out at its data-layer close (2026-07-08).
- Relates to: INT-118 (friday engine resumption), INT-039 (friday daemon), INT-041 (shell context).

## Notes
Filed 2026-07-08 directly (not via `core intent new` -- that command errors because
faelight/intents/templates/ does not exist, likely lost in the INT-061 tree move; worth
a hygiene-pass fix later, out of scope here). Origin: while closing INT-128, Christian
asked for "a teaching path where we both learn from one another." This charter is the
honest, buildable form of that vision.

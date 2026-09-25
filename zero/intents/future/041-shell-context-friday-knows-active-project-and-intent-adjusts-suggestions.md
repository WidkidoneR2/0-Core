---
id: 041
date: 2026-06-08
type: feature
title: "shell-context: Friday knows active project and intent, adjusts suggestions"
status: planned
depends_on: [039]
tags: [friday, shell-context, suggestions, intent-aware, prediction, rust, nixos]
version: TBD
---

## Vision
Friday's suggestions today are context-free: it knows things, but not where you are or what you
are trying to do. shell-context makes Friday situated -- it knows the active project, the focused
intent, the git state, and your recent moves, and uses that to make every suggestion, every
surfaced fact, every prediction fit THIS moment. A shell that understands intent: when you cistart
073, Friday knows you are on generation-control and surfaces the relevant commands, risks, and past
decisions; when you hit an error in 0-core, it suggests the fix that fits this project, not a generic
one; when you switch projects, the vocabulary and knowledge follow. Context is the difference between
an assistant that answers and one that anticipates.

## Why Now
Friday already emits suggestions and tracks whether they were useful ("50% useful, trust building")
-- but without context that signal is capped: generic suggestions can only be so relevant, and
irrelevant ones erode trust. The forest already KNOWS the context (cistart sets a focused intent, the
prompt knows the project and git state, fsh sees recent commands); Friday just is not consuming it
yet. Once INT-039 gives Friday an always-on place to hold context and INT-071 restores its commit
learning, situating that intelligence is the natural next multiplier.

## What
Friday becomes context-aware along these signals:
- Active project / directory (0-core vs elsewhere; which crate).
- Active intent -- the cistart focus, including that intent's gates and blockers.
- Git state -- branch, dirty/clean, recent commits.
- Recent activity -- last commands and errors in the session.
And uses them to:
- Rank and filter suggestions by context (situated beats generic -- and we MEASURE that).
- Be intent-aware: surface the focused intent's next gate, or what it is blocked on.
- Scope knowledge by project: facts/patterns retrieved by where you are.
- Situate prediction: the next-action guess is conditioned on context, not global priors.

## Approach
Define a crisp context object (project, active intent, branch, recent events) held by the
friday-daemon (INT-039) and updated continuously from signals the forest already emits -- cistart
focus, the prompt's project/git detection, fsh's command stream. Friday's existing suggestion and
knowledge paths gain a context filter/ranker in front of them. The honesty mechanism is built in:
Friday already measures suggestion usefulness, so context-ranking must demonstrably beat the generic
baseline on that metric -- if situated suggestions are not more useful, they are just more noise, and
we do not ship noise. The intent-aware hook reads the focused charter's gates (the ledger is
file-based and right there) so Friday can name the next gate or the blocker.

## Phases
Phase 0 -- context model: define the context object (signals + sources + how the daemon holds it).
  Record here. Depends on INT-039 for the always-on holder.
Phase 1 -- context capture: daemon tracks active project + active intent + git state continuously.
Phase 2 -- context-ranked suggestions: filter/rank by context; usefulness measured vs the generic
  baseline -- improvement shown honestly, or the ranker is reworked.
Phase 3 -- intent-aware assistance: Friday surfaces the focused intent's next gate / blocker.
Phase 4 -- project-scoped knowledge: facts/patterns tagged and retrieved by project context.

## Gates
- [ ] Phase 0: context model defined and recorded (signals, sources, daemon-held)
- [ ] daemon tracks active project + active intent + git state continuously
- [ ] suggestions ranked by context; usefulness beats the generic baseline (measured, honest)
- [ ] intent-aware: Friday surfaces the focused intent's next gate / blocker
- [ ] project-scoped knowledge retrieval working

## Notes
- Depends on INT-039 (daemon = the always-on context holder); build 039 first.
- Benefits from INT-071 (commit learning) and INT-034 (intent/commit data) for material.
- Distinct from INT-071 (parity): this is a NEW capability (situated intelligence), not a recovery.
- Anti-noise rule (non-negotiable): situated suggestions must measurably beat generic, or they are
  clutter. The existing usefulness/trust metric is the gate, not vibes.
- How far this could go: context is the precondition for genuine anticipation -- the v11 Prediction
  and v12 Strategy pillars become real once conditioned on what you are actually doing. shell-context
  is where Friday stops reacting and starts reading the room.

## The Rule
"Knowing things is memory. Knowing what you are doing is attention. Friday needs both." 🌲

## Dependencies

**INT-039** -- friday-daemon is the always-on context holder this intent reads from.
Stated in the body since filing: build 039 first. Shell context has nothing to attach to
until a persistent Friday process exists.

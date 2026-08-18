---
id: 223
date: 2026-08-17
type: arch
title: "core imports intelligence and the layering invariant cannot be tested"
status: planned
priority: high
depends_on: []
tags: [architecture, rust, design]
---

## Vision

Remove every Intelligence component and the system still builds, boots, and is usable.

That sentence is the whole intent. It is currently false, and it is the only statement of the
architecture that can be tested rather than argued.

⚠️ THIS IS NOT ABOUT MAKING INTELLIGENCE SMALLER. Friday is real work and roughly 24 of ~55 engine
domains belong to it legitimately. The problem is direction, not size.

## The Problem

The stated architectural invariant is **Core must never depend on Intelligence**. Decision 143
records it. Every architecture document written so far assumes it.

⚠️ Measured 2026-08-17: it is not true. Five Core domains import Intelligence directly, so deleting
the Intelligence domains today would fail to compile. The invariant has never been tested because
nothing has ever been able to test it.

## Evidence

`grep -rln` for `domains::{friday, knowledge, planning, predict, prioritize, strategy}` across
`faelight/engine/src` returns seven files.

**Two are legitimate and stay:**

- `app/dispatcher.rs` -- the composition root. It routes to every domain by design; that is what a
  composition root is for.
- `domains/friday_arch/mod.rs` -- itself Intelligence.

**Five are upward dependencies from Core:**

| File | Layer it belongs to |
| --- | --- |
| `domains/deploy/mod.rs` | State |
| `domains/doctor/mod.rs` | Trust / diagnostics |
| `domains/intent/mod.rs` | Graph + State |
| `domains/snapshot/mod.rs` | State |
| `domains/status.rs` | Experience / reporting |

★ AND THE COUPLING LOOKS SHALLOW, WHICH IS WHY THIS IS TRACTABLE. Two known call sites:

- `doctor/mod.rs:563` -- `if let Some(celebration) = crate::domains::friday::check_milestones(ctx)`.
  A celebration message.
- `cistart` prints "🎯 Intent is now focused" from `domains/intent`.

Both are **presentation**: hints, celebrations, focus messaging. Neither is core function. If the
remaining sites are the same shape, this is an inversion rather than a rewrite.

⏭ NOT YET MEASURED: whether any of the five calls Intelligence for a value it actually needs, rather
than for something it prints. That distinction decides the size of the work and is the first gate.

## The Solution

**Core emits events. Intelligence subscribes.**

`domains/events` already exists for this. Instead of `doctor` calling `friday::check_milestones`,
doctor emits a health-checked event and Friday reacts to it. The dependency arrow reverses without
either side losing the behaviour.

⚠️ THE PREREQUISITE, AND THE REASON THIS INTENT HAS `depends_on: []`: INT-175 "finish the event
bus" is unfinished, and INT-214 measured that **no commit has ever created `events.source_tool` or
`events.correlation_id`** -- a database built from source cannot record an event. Inverting onto an
event bus that does not carry provenance would move the problem rather than solve it.

★ This is also the first populated `depends_on` edge in the ledger, which is the mechanism decision
142 exists to establish. The feature proves itself on its own prerequisite.

## Success Criteria

### Establish the truth

- [ ] Each of the five importing files is classified **presentation** or **functional**, with the
      call site quoted. A functional dependency is a different problem from a printed hint and must
      not be counted as one.
- [ ] `dispatcher.rs` and `friday_arch` are recorded in this intent as **legitimate**, with the
      reason, so a future reader does not re-litigate them.
- [ ] The baseline is captured by **watching it fail**: remove the Intelligence domain declarations,
      run `cargo check --workspace`, and record the exact errors. That error list is the definition
      of done, in reverse.

### Decide before moving

- [ ] INT-175's real state is measured, not assumed. If the event bus cannot carry provenance, that
      is settled first or this intent is deferred with a reason.
- [ ] For each presentation-class site, the replacement is decided and written here: which event,
      emitted where, consumed by what.
- [ ] For any functional-class site, the decision is recorded explicitly -- invert it, move the
      needed logic down out of Intelligence, or accept the dependency with a stated reason.
      ⚠️ Accepting one is allowed. Accepting one silently is not.

### Prove it

- [ ] Each site is inverted, and the behaviour it produced still happens. A celebration that stops
      appearing is a regression, not a success.
- [ ] ★ **THE GATE: with the Intelligence domains removed, `cargo check --workspace` succeeds.**
      Currently false. Proven by running it, not by reasoning about it.
- [ ] `faelight-deadwood` gains a mechanical check that flags an import from a Core domain into an
      Intelligence domain. **Proven by watching it fail first:** reintroduce one, watch it be
      flagged, remove it, watch the flag clear.
- [ ] `AGENTS.md` gains the rule in its Dependency Rules section, stated as enforceable rather than
      aspirational, with a pointer to the deadwood check that enforces it.

## Prior art -- do not duplicate

- **decisions/143** -- the layer tree, and where this invariant is recorded
- **decisions/142** -- ledger shape; this intent is its first `depends_on` edge
- **INT-175** -- finish the event bus (future/). The prerequisite.
- **INT-214** -- no commit ever created `events.source_tool` or `events.correlation_id`
- **INT-167** -- DevBox; the instrumentation half of the same event story
- **INT-192** -- tools that cannot express an undetermined outcome
- **INT-222** -- doctor integrity. ⚠️ Overlaps at `doctor/mod.rs`. 222 owns whether checks are
  honest; this owns which direction doctor's imports point. **Do not let either edit the other's
  ground without saying so.**

## Non-goals

- Making Intelligence smaller. It is large because Friday is real.
- Moving any directory. Decision 143 already ruled that the tree is not the directory layout.
- Rewriting the doctor, the deploy path, or the intent tooling. This changes which way arrows point,
  nothing else.
- Removing the celebrations, hints, or focus messaging. They stay; they arrive by a different route.

## Risk

`system`. Nothing here is lockout-class -- it is engine code, not boot, login, or disk.

⚠️ The realistic failure mode is a silent behavioural regression: an inverted call site that
compiles, passes, and quietly stops producing the hint it used to. Every inversion needs the
behaviour demonstrated, not the build.

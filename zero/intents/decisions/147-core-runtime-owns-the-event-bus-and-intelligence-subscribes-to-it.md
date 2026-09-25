---
id: 147
date: 2026-08-17
type: decision
title: "core runtime owns the event bus and intelligence subscribes to it"
status: decided
tags: [decision]
---

## Context

INT-223 records that five Core domains import Intelligence, so the invariant *Core must never depend
on Intelligence* is currently false. Its fix is inversion: **Core emits events, Intelligence
subscribes.**

That fix has a prerequisite nobody had ruled on -- **who owns the event bus.**

⚠️ INT-175, "finish the event bus", is CANCELLED, and its cancellation reason matters:
`faelight-daemon` is Arch-era prototype code untouched since the NixOS migration, so *"finish the
event bus" had a false premise -- there is no NixOS event bus to finish, only a dead Arch relic.*
It then said that if a live bus is wanted later, **it belongs with friday-daemon (INT-039)**.

⚠️ That successor choice collides with decision 145. **friday-daemon is Intelligence.** If the bus
lives there, Core emitting an event *is* a dependency on Intelligence -- precisely what INT-223
exists to remove.

★ The cancellation was written 2026-07-20, four weeks before decision 145 existed. This is not a
contradiction to argue about; it is a decision that was never made.

## Decision

**Core Runtime owns the event model and the event bus. Intelligence subscribes.**

```
                    Core Runtime
                    domains/events
                    event definitions + bus
                          |
                 publishes | subscribes
                          |
        +-----------------+-----------------+
        |                                   |
   Core consumers                    friday-daemon (INT-039)
                                         Intelligence
```

Not:

```
Core -----depends----> friday-daemon
```

Core publishes facts about itself -- a command completed, the system entered a state, an activation
finished -- and **does not need to know who cares.** The bus is a decoupling mechanism, which is the
opposite of a dependency.

★ **The newer invariant wins over the older cancellation.** 175's reasoning about *what* was dead
was correct and stands; its guess about *where the successor belongs* predates the ownership model
and is superseded here.

### Three things, not one

⚠️ `domains/events` currently blurs three separate concerns, and an earlier reading of this
architecture treated the 43,000+ persisted rows as evidence that a bus exists. **It is evidence that
a LOG exists.**

| Concern | What it is |
| --- | --- |
| **Event definitions** | what events exist, and their schemas |
| **Event bus** | how events are published and delivered to subscribers |
| **Event persistence** | the historical log, 43,000+ rows today |

⚠️ **Do not let the historical event store become the architectural authority for the runtime bus.**

The contract:

```
Core Runtime      EventBus.publish(event)
EventBus          dispatch(event)
Subscriber        on_event(event)
```

**Core knows only the first line.**

### The test, and what kind of test it is today

Two questions, stronger than reading dependency declarations:

1. Can Core build with zero Intelligence code?
2. Can friday-daemon be added afterward and observe Core events **without modifying Core**?

⚠️ **RULED: today this is a BUILD-TIME property, not a runtime one.** Intelligence is roughly 24
domains inside the single `engine` crate; there is no "install without friday-daemon". So the
immediate form of test 1 is INT-223's existing gate -- **exclude the relevant module declarations
and run `cargo check --workspace`.**

★ A future crate split would turn that into a genuine runtime and dependency boundary. That split is
not decided here.

## Sequencing -- and this is part of the decision

⚠️ **THE RULING DOES NOT AUTHORISE THE WORK.**

Building a minimal event bus is new code in Core, and it belongs to **P5** of `ROADMAP.md`. Starting
it now would skip P0 (make the ledger able to answer), P1 (the safety floor and the unverified boot
recovery), P2 (docs and desktop truth), P3 (the module shape) and P4 (the tree) -- on the day the
roadmap was committed.

★ **Treating a resolved decision as permission to start work is the thread-proliferation failure
already named in this project's own history.** The architecture is settled either way; what is at
stake is whether the phase order survives contact with a good idea.

**Decided now. Implemented at P5.**

## Consequences

- INT-223 has an achievable fix rather than a blocked one, and no invented dependency edge.
- INT-039 (friday-daemon) becomes a **subscriber**, and can be built or deferred without holding up
  Core.
- `faelight-daemon` remains what 175 called it: **salvage and reference, not the thing to finish.**
- The first real event-bus milestone is defined and testable: Core builds, Core emits, the bus
  delivers, no Intelligence dependency exists, friday-daemon can subscribe afterward.
- P5 gains a concrete first task -- **define the minimal event-bus contract** -- rather than an
  open-ended one.

## Deliberately not decided here

- Whether Intelligence becomes a separate crate. That would upgrade the test from build-time to
  runtime, and it is its own decision.
- The event schema, transport, or delivery guarantees.
- Whether INT-039 is built at all, or when.
- INT-214's place. It concerns `events.source_tool` and `events.correlation_id` never being created
  from source, which breaks a database built from scratch rather than the running machine. It is a
  **DevBox prerequisite**, and INT-213's G6 already names `167 -> 214`.

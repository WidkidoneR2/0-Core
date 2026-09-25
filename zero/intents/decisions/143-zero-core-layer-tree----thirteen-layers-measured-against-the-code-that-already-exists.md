---
id: 143
date: 2026-08-17
type: decision
title: "zero core layer tree -- thirteen layers measured against the code that already exists"
status: decided
tags: [decision]
---

## Context

Three architecture documents have proposed a layer tree for Zero Core. Each added layers; none
subtracted. Eleven became thirteen. Zero directories were created, and no layer had ever been
tested against the code that exists.

On 2026-08-17 the inventory was read and every existing component assigned to a layer. The
question was not "is this tree elegant" but "does anything live here."

Inventory: 34 crates in `faelight/rust-tools/`, roughly 55 domains in `engine/src/domains/`,
32 modules in `faelight-shell/src/`, and `nix/` at home / hosts / lib / modules / profiles / tests.

## What the table showed

**Populated, with real inhabitants today:**

- **State** -- domains/{snapshot, checkpoint, deploy, update, release}, faelight-update,
  faelight-release, gen-diff, runtime/state.db
- **Graph** -- domains/deps, domains/genealogy, domains/intent, domains/decisions, domains/events
- **Runtime** -- domains/{events, daemon, engines, sandbox}, faelight-daemon, faelight-insightd,
  faelight-vm, fsh jobs/exec/pty_exec
- **Storage** -- domains/{db, journal, nix}, state.db, atticd
- **Trust** -- engine/capabilities, domains/{capabilities, security, audit, integrity, sandbox},
  faelight-vault, faelight-sandbox, fsh safety_guard, policy/
- **Shell** -- faelight-shell, fsh-test, faelight-term
- **Experience** -- fsh prompt/completion and six TUIs, faelight-{compositor, notify, lock, login,
  wallpaper, clipboard, idle, wsd, zone, glog, docs}
- **Data** -- faelight-fm, db-browse, domains/db, teach, fsh output
- **Foundation** -- faelight-core, engine/errors, fsh value.rs (thin, but real)

**Not populated:**

- **Model** -- three weak candidates (schema/, policy/, meta/), all of which read as Foundation or
  State
- **Network** -- domains/fetch and atticd distribution, and nothing else

## Decision

### 1. Model is not a layer yet

⚠️ A layer with one honest inhabitant is the same shape as a profile with one consumer. The rule
already exists, written in `nix/profiles/base.nix`:

> A profile with one consumer is ceremony, not structure. They get built when a second machine
> needs them -- which is when they start being true.

Model gets built when a second thing needs it. Schema validation lives in Foundation until then.

### 2. Network is not a layer yet

Same reasoning. `domains/fetch` and a binary cache are not a network layer. It becomes one when
Core actually distributes something.

### 3. Trust is cross-cutting, not a link in the chain

Storage needs it, Runtime needs it, Network needs it, Shell needs it. The band diagram that places
Trust at position 8 in a linear gravity order is wrong, and the first architecture document had it
right the first time.

### 4. Intent belongs to Graph and State, not to Intelligence

The ledger is identity, status, and relationships -- Foundation, State, Graph. Intelligence *reads*
it and contributes Planning.

⚠️ Filing Intent under Intelligence would make the ledger require the AI layer to exist, which
contradicts the invariant below.

### 5. Objects becomes Data, and Processes leaves it

`Foundation/Object` beside `Objects/Files` forces an explanation every time. Data is about content
representations. Processes belong unambiguously to Runtime.

### 6. Numbering means architectural depth, never priority

Three structures, three different kinds of order:

```
TREE    = architecture      how the system is constructed      01 -> 13, stable
GRAPH   = relationships     what depends on what               edges, not order
INTENT  = decisions         what to do next and why            dynamic, re-derived
```

The tree's order is structural. The ledger's order is computed from readiness and priority. Neither
borrows the other's numbering. This is the same conclusion the ledger reached independently in
decision 142.

### 7. The tree is not the directory layout

Architecture is about conceptual boundaries; directories are about maintaining code. No directory
moves follow from this decision. Any future move needs its own intent with a path audit attached --
the thing INT-061 skipped, whose wreckage was still being found in the docs six weeks later.

## The invariant, and the measurement that contradicts it

The stated rule is: **Core must never depend on Intelligence.** Remove every AI component and the
system still works.

⚠️ **Measured 2026-08-17: this is not true today.**

A grep for `domains::{friday, knowledge, planning, predict, prioritize, strategy}` across
`engine/src` returns seven files. Two are legitimate -- `app/dispatcher.rs` is the composition root
and routes to every domain by design, and `friday_arch` is itself Intelligence.

**Five are real upward dependencies:**

- `domains/deploy/mod.rs`
- `domains/doctor/mod.rs`
- `domains/intent/mod.rs`
- `domains/snapshot/mod.rs`
- `domains/status.rs`

State, Graph, and the health check all reach up into Intelligence. Concretely: **delete the
Intelligence domains today and the engine does not compile.**

★ But the coupling looks shallow, and therefore fixable. `doctor/mod.rs:563` calls
`domains::friday::check_milestones(ctx)` to print a celebration. `cistart` prints "Intent is now
focused." This is presentation -- hints, celebrations, focus messaging -- not core function.

The fix direction is inversion: **Core emits events; Intelligence subscribes.** That is what
`domains/events` exists for.

## Also recorded

- **Intelligence is the largest layer**, roughly 24 of ~55 engine domains. The tree describes it as
  a thin removable cap. The code does not agree, and that gap is worth knowing.
- **Version control has no layer.** faelight-git, faelight-glog, domains/git, fsh git_tui -- four
  components, no home. Either a missing layer or an admission that VCS is Ecosystem.
- **Developer tooling is correctly homeless.** faelight-deadwood, teach, fsh-test, intent-guard,
  db-browse, gen-diff belong to Ecosystem, outside Core.
- **`capabilities` exists twice** -- `engine/src/capabilities/` and
  `engine/src/domains/capabilities/`. Two owners, unresolved.
- **`faelight-bar` has no crate** in `rust-tools/`, while `nix/home/christian/faelight-bar.nix`
  exists and `faelight-bar-gtk` is in systemPackages. A strong candidate for why no bar runs.
- **`faelight-clipboard`, `faelight-idle` and `faelight-wallpaper` exist as crates.** Adopting
  cliphist, swayidle or wpaperd would displace his own tools, not fill gaps.

## Deliberately not decided here

- Directory structure. Not this decision, and not soon.
- Whether VCS gets a layer. Recorded as open.
- Whether Intelligence should be smaller. It is large because Friday is real work, not because
  something went wrong.
- Renumbering, prefixes, or the Faelight to Zero migration. Separate.

## Consequences

- The tree drops from thirteen layers to eleven, and both cuts are justified by measurement rather
  than taste.
- The invariant becomes a testable gate rather than a slogan: **remove the Intelligence domains and
  the system still builds.** Currently false. That sentence is worth more than any diagram.
- New code has a home, because every remaining layer holds something real.

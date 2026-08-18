---
id: 213
date: 2026-08-09
type: future
title: "the intent dependency graph is unfed, so core intent blocked reports "nothing blocked" falsely across 239 of 241 intents"
status: in-progress
tags: [ledger, intent, deps]
---

## Vision
The ledger knows its own order. When an intent cannot start until another is finished, the ledger
says so before the work begins -- not after a session is spent building on a foundation that was
not there yet.

## The Problem
The tooling is built and the data is empty. `core intent deps / blocked / next / graph` all consume
`depends_on`, and it works: adding one line to INT-161 made `blocked` correctly report it waiting on
INT-160. But across 241 intents the relationship keys are populated on TWO. So `core intent blocked`
answers "no blocked intents -- all dependencies satisfied", and that answer is FALSE rather than
empty. A command that reports confidently on an empty graph is worse than one that reports nothing,
because it is trusted.

The cost is the pattern this intent exists to end: starting an intent whose prerequisite is not
done, discovering it mid-session, and going back over code from a previous pass. That is not a
planning inconvenience -- it is rework that breaks work already proven.

## The Solution
Feed the graph. This is a DATA problem, not a code problem -- nothing needs building for
`depends_on` to work today.

But three code defects must be fixed FIRST, because each turns a wrong edge into a silent one, and
a graph that fails silently is worse than no graph:
  - A `depends_on` naming an id that does not exist renders as "INT-NNN (not found)" and blocks its
    dependent forever. Nothing validates the reference.
  - `next_intent` skips every blocked intent with a bare `continue` (mod.rs:2781-2783) and says
    nothing. A mis-typed edge makes an intent disappear from recommendations with no explanation.
  - Only `status == "complete"` clears a dependency (`complete_ids`, mod.rs:2702). A CANCELLED
    dependency therefore blocks its dependents permanently, and eight cancelled intents exist.

Then populate deliberately, with the discipline that keeps the graph trustworthy: `depends_on` means
"cannot start until", never "related to". Over-declaring turns today's false negatives into false
positives, and a `blocked` list full of soft associations is one nobody reads.

## Evidence (measured 2026-08-09)
- 241 intents; `depends_on` populated on 1, `blocks` on 1, `relates` on 0.
- The consumers exist and are correct: `deps` (mod.rs:1696), `blocked` (2698), `next_intent` (2762),
  `deps_critical_path` (2553), `graph` (2923).
- `blocked` currently prints "No blocked intents -- all dependencies satisfied" against an empty
  graph.
- CYCLE SAFETY IS ALREADY PRESENT and should not be re-solved: `deps_critical_path` inserts into a
  `visited` set before following (2576-2595), so a cycle cannot hang. But two intents each naming the
  other would both be silently skipped by `next_intent` forever -- a silent failure, not a hang.
- `blocks:` is NOT wholly unread: `deps` displays it (1716). It is dead to the SCORING and BLOCKING
  logic only, which reads other intents' `depends_on` (2789) and never `self.blocks`.
- `priority:` is documentation, not signal -- `next_intent` scores on unblocks + tags only and never
  reads it, though `core intent deps` instructs the user to set one. 72 intents carry one.

## Non-goals
- A new dependency mechanism. `depends_on` works; this feeds it.
- Declaring an edge for every pairing. Soft associations belong in `relates`.
- Retro-filing dependencies on complete intents. The graph exists to order FUTURE work.

## Success Criteria
- [ ] G1 RED FIRST, captured before any edge is written: `core intent blocked` reports no blocked
      intents while at least one real ordering is known to exist. Record the output verbatim -- it is
      the false answer this intent removes
- [ ] G2: a `depends_on` naming a nonexistent intent id is a VALIDATION ERROR from
      `core intent validate`, not a silent permanent block. Proven by seeding a bad reference,
      watching validate name it, and removing it
- [ ] G3: a dependency CYCLE is a validation error. Proven by seeding a two-intent cycle and watching
      validate reject it. ⚠️ Do NOT re-implement cycle protection in the walkers -- 2576's visited set
      already prevents the hang; this catches the SILENT case, where both intents vanish from
      recommendations
- [ ] G4: what satisfies a dependency is DECIDED AND IMPLEMENTED. Today only `complete` clears, so a
      cancelled dependency blocks forever. State the rule for cancelled and for superseded, in
      writing, before changing the code
- [ ] G5: `next_intent` no longer skips silently -- a blocked intent is either named with its blocker
      or counted, so an intent can never disappear from recommendations without explanation
- [ ] G6: THE FIRST REAL EDGE, chosen because it is already proven rather than assumed: INT-167's P0
      cannot proceed until INT-214, since the events table cannot be created from source. Encode it
      and show `deps 167`, `blocked` and `next` all reflecting it
- [ ] G7: a deliberate pass over the planned and in-progress intents, declaring only edges that mean
      "cannot start until". Each edge names its reason in the depending intent, so a future reader can
      check it rather than trust it
- [ ] G8: the discipline is written into the intent and into CONVENTIONS: `depends_on` = cannot start
      until; `relates` = worth reading together. A false positive costs more than a false negative
      here, because a noisy blocked list stops being read
- [ ] G9: `blocks:` is resolved -- either made real for the blocking logic or documented as
      display-only, so nobody populates it expecting it to gate anything
- [ ] G10: `priority:` is recorded or fixed. `core intent deps` tells the user to set a field that
      changes no recommendation
- [ ] G11: `core intent blocked` gives a TRUE answer, demonstrated: it either names real blockers or
      reports none against a graph that has edges in it
- [ ] G12: each gate carries evidence per INT-158

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->

## Dependency Satisfaction Contract (G4 -- decided 2026-08-17)

**The rule, stated before the code changes, as this gate requires.**

| Dependency status | Satisfies? | Effect on the dependent |
| --- | --- | --- |
| `complete` | ✅ yes | unblocked, no flag |
| `cancelled` | ✅ yes | **unblocked, but FLAGGED as questionable** |
| `planned` | ❌ no | blocked |
| `in-progress` | ❌ no | blocked |
| `deferred` | ❌ no | blocked -- paused is not abandoned |
| `superseded` | -- | out of scope; the status does not exist yet |

### Why cancelled satisfies

⚠️ Today only `complete` clears, so **a cancelled dependency blocks its dependent forever**, and
eight cancelled intents exist. That is plainly wrong: when B is cancelled, A is no longer waiting
for anything.

★ But silently clearing it is also wrong. **Cancellation removes the blocking condition without
retroactively making the dependency assumption true.**

**Proven live, 2026-08-17.** INT-223 declared `depends_on: [175]`. INT-175 is cancelled -- and
cancelled *because its premise was false*: `faelight-daemon` is Arch-era prototype code and there
was never a NixOS event bus to finish. So 223 encoded the assumption *the event bus will be
finished*, and that assumption became **false, not satisfied**. What 223 actually needed was a
decision, and it got one (decisions/147).

Had the flag existed, it would have surfaced that faster than `core intent blocked` did.

### The flag

A dependent whose dependency was cancelled is reported as:

> depends on a cancelled intent -- the assumption behind this edge may no longer hold

⚠️ This is **INT-192 applied to edges**: a state that is neither clean nor blocked must be
expressible, or it gets reported as one of the two. Neither "blocked forever" nor "silently fine"
is the truth here.

### Superseded

Doc A proposed a `superseded` status; nothing implements it. **Deciding its semantics blind would be
guessing.** If it is introduced later, it satisfies **only through the completed replacement** --
a chain (`A depends_on B`, `B superseded_by C`, satisfied only when `C` is complete), not a flag.

### Implementation note

⚠️ **The rule has FIVE owners, not one.** `complete_ids` is rebuilt independently in `start` (802),
`blocked` (2702), `next_intent` (2766), `brief` (2859) and `graph` (2927) -- all user-facing
commands, all treating only `complete` as satisfying.

★ The fix is **one shared helper and five call sites**, following INT-070 (*"three copies of this
logic existed before"*) and INT-135 (*"now calls the ONE validator"*).

📍 `deps` (1696) and `deps_critical_path` (2553) do **not** build `complete_ids` -- they determine
satisfaction some other way. Check both before claiming the helper covers everything.

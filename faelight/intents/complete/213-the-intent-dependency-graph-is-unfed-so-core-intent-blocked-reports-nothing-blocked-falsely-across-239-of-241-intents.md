---
id: 213
date: 2026-08-09
type: future
title: "the intent dependency graph is unfed, so core intent blocked reports "nothing blocked" falsely across 239 of 241 intents"
status: complete
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
- [x] G1 RED FIRST, captured before any edge is written: `core intent blocked` reports no blocked
      intents while at least one real ordering is known to exist. Record the output verbatim -- it is
      the false answer this intent removes
<!-- evidence: 2026-08-17, DISCHARGED BY EXPLANATION (INT-027:58 precedent). The false answer
     this gate asked to capture was NO LONGER REPRODUCIBLE: two edges existed by the time it ran
     (161 depends_on 160, 212 depends_on 211), so blocked answered truthfully. Captured instead,
     verbatim: 2 intents blocked -- INT-212 waiting on INT-211 (planned), and INT-223 waiting on
     INT-175 (CANCELLED). That second line is the cancelled-dependency trap firing live, which is
     better evidence than the empty-graph answer would have been. -->
- [x] G2: a `depends_on` naming a nonexistent intent id is a VALIDATION ERROR from
      `core intent validate`, not a silent permanent block. Proven by seeding a bad reference,
      watching validate name it, and removing it
<!-- evidence: commit e1d4efa8, gen 501. Watched it fail: seeded INT-224 depends_on INT-999,
     validate reported one issue naming the intent, the reference and the file. Removed it and
     validate returned All 261 intents valid. -->
- [x] G3: a dependency CYCLE is a validation error. Proven by seeding a two-intent cycle and watching
      validate reject it. ⚠️ Do NOT re-implement cycle protection in the walkers -- 2576's visited set
      already prevents the hang; this catches the SILENT case, where both intents vanish from
      recommendations
<!-- evidence: commit e1d4efa8, gen 501. Watched it fail: pointed 224 and 225 at each other,
     validate reported a cycle for BOTH ends. Reverted, validate returned clean. Confirms the
     visited set in deps_critical_path was never the issue -- the silent case was. -->
- [x] G4: what satisfies a dependency is DECIDED AND IMPLEMENTED. Today only `complete` clears, so a
      cancelled dependency blocks forever. State the rule for cancelled and for superseded, in
      writing, before changing the code
<!-- evidence: contract written BEFORE the code in commit d9d50316; helper added unused in
     05435aa8; five call sites converted by eefa3bbb, verified on gen 500. grep -n complete_ids on
     intent/mod.rs now returns NOTHING (exit 1). start, blocked, next_intent, brief and graph all
     route through dep_state. brief keeps counting completed intents directly, with a comment,
     because that is a statistic and not a dependency question. -->
- [x] G5: `next_intent` no longer skips silently -- a blocked intent is either named with its blocker
      or counted, so an intent can never disappear from recommendations without explanation
<!-- evidence: commit 19646fc7, gen 499. next_intent now prints Not recommended, and why:
     INT-212 waiting on INT-211. Before, the bare continue at mod.rs:2781-2783 made it vanish with
     no message. Recommendation itself unchanged, which is the control. -->
- [x] G6: THE FIRST REAL EDGE, chosen because it is already proven rather than assumed: INT-167's P0
      cannot proceed until INT-214, since the events table cannot be created from source. Encode it
      and show `deps 167`, `blocked` and `next` all reflecting it
<!-- evidence: commit fb9df845, gen 503. INT-167 depends_on 214, with the reason written into
     167 under a Dependencies heading. validate clean, deps 167 renders the edge, blocked lists it.
     Chosen because it was already proven rather than assumed. -->
- [x] G7: a deliberate pass over the planned and in-progress intents, declaring only edges that mean
      "cannot start until". Each edge names its reason in the depending intent, so a future reader can
      check it rather than trust it
<!-- evidence: commit c2f43db1. Method was measurement: grep found only five prose-stated
     dependencies across 57 planned intents. Two were genuinely blocking and are encoded with their
     reasons -- 041 depends_on 039, 155 depends_on 142. 157 to 027 was found and deliberately NOT
     encoded, because 027 is complete and an edge is a claim that work is impossible. The count is
     itself the finding: the real ordering is milestone-gated, not intent-gated. -->
- [x] G8: the discipline is written into the intent and into CONVENTIONS: `depends_on` = cannot start
      until; `relates` = worth reading together. A false positive costs more than a false negative
      here, because a noisy blocked list stops being read
<!-- evidence: commit edb575f3. docs/CONVENTIONS.md now has three sections; the third is
     Dependency edges, written in the house shape. Its tell: if you cannot name what would break by
     starting anyway, it is not a dependency. Records both failure directions. -->
- [x] G9: `blocks:` is resolved -- either made real for the blocking logic or documented as
      display-only, so nobody populates it expecting it to gate anything
<!-- evidence: commit fc3a3a8f, gen 503. RESOLVED AS DERIVED, per law 3. deps computes the set
     from every intent whose depends_on contains this id; the frontmatter hint no longer advertises
     the field. deps 214 shows 167 and deps 039 shows 041, both with nothing declared in their own
     frontmatter. deps 160 still shows 161 because 161 carries the reciprocal depends_on -- the one
     case where both directions were stored, they agreed, and the derived path reproduces it. -->
- [x] G10: `priority:` is recorded or fixed. `core intent deps` tells the user to set a field that
      changes no recommendation
<!-- evidence: commit 34369e4a, gen 502. FIXED, not merely recorded. Intent gained a priority
     member, parse_intent reads it lowercased, next_intent scores it, and the tag bonuses were
     removed because a security tag added 20 on top of a declared priority. Before: INT-223 and
     INT-225 at priority high scored 10/100 and did not appear at all. After: both at 50/100 citing
     declared priority: high. Also surfaced that seven intents were high against a cap of three;
     four were demoted in commit 38698c20. -->
- [x] G11: `core intent blocked` gives a TRUE answer, demonstrated: it either names real blockers or
      reports none against a graph that has edges in it
<!-- evidence: gen 503. core intent blocked reports four, every named blocker real: 041 waiting
     on 039, 155 on 142, 167 on 214, 212 on 211. validate clean at 262 intents. The graph has edges
     in it and the answer is true. -->
- [x] G12: each gate carries evidence per INT-158
<!-- evidence: this pass. Every gate above carries a commit hash, a generation number, or a
     demonstrated before-and-after. G1 carries prose because the thing it asked to capture had
     stopped existing -- stated rather than papered over. -->

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

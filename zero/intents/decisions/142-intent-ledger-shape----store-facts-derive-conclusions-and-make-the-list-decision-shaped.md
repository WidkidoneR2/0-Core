---
id: 142
date: 2026-08-17
type: decision
title: "intent ledger shape -- store facts, derive conclusions, and make the list decision-shaped"
status: decided
tags: [decision]
---

## Context

The ledger holds 241 intents. Choosing what to work on next is currently guesswork, and three
separate measurements from 2026-08-17 explain why.

**The ranker already exists and has no fuel.** `next_intent` (`intent/mod.rs:2762`) filters to
planned intents, skips anything with unmet `depends_on`, scores `unblocks * 15` capped at 30 plus
10 for being unblocked, sorts descending, and prints a score out of 100 with a reasons list. It is
a self-explaining recommender. But INT-213 measured `depends_on` as unfed across 239 of 241
intents, so it scores an empty graph.

**Priority already exists, undeclared, encoded in tags.** The same function adds 20 for a tag
containing `security` or `critical`, 15 for `intelligence` or `friday`, and 10 for `v2`. Nobody
declared this. Tagging an intent silently changes its rank forever.

**The list view shows none of what drives the choice.** A typical line is a paragraph-long title,
a `Status:`, a `Date:` whose meaning INT-211 records as inconsistent across the ledger, and eight
tags. No priority, no readiness, no blockers, no indication of what an intent unblocks.

## Decision

### 1. Store facts. Derive conclusions.

`depends_on` is a fact. `blocked` is a conclusion. "Work on this next" is a higher conclusion.

**Only facts live in the file.** Anything that changes when a *different* file changes must be
computed at display time, never stored.

Concretely, this block is `core intent show <id>` output, NOT file content:

```
ID          222
IMPORTANCE  high
READY?      no
BLOCKED BY  213
UNBLOCKS    0
```

⚠️ The moment INT-213 completes, a stored "BLOCKED BY 213" inside another intent becomes a lie and
nothing edits it. That is the two-owners defect, relocated into the ledger.

`WHY` and `WHAT MUST BE TRUE` are the exception. They are genuinely file content and already exist
in the template as `## The Problem` and `## Success Criteria`.

### 2. Frontmatter: two new fields, nothing renamed

```yaml
id: 222
date: 2026-08-17
type: arch
title: "short -- 80 characters or fewer"
status: in-progress
priority: high
depends_on: []
tags: [architecture, rust, design]
```

**`priority: high | medium | low`.** At most **three** may be high at any time. Promoting a fourth
requires demoting one.

**`depends_on: []` is written at creation, always, even when empty.** An explicitly empty list is a
decision. A missing field is how 239 of 241 ended up unfed.

**Nothing is renamed.** `id:` stays numeric, `date:` stays `date:`. `parse_intent` reads both and
240 files use them. INT-158 is forward-only; renaming for tidiness is a breaking change for no gain.

**No `updated:` field** unless the tooling writes it. A hand-maintained timestamp rots, and INT-211
already records that `date` means four different things -- a second date field makes that worse.

### 3. Explicit priority replaces tag scoring

Once `priority:` exists, the tag bonuses in `next_intent` are removed. Otherwise a `security` tag
adds 20 on top of a declared priority and the same intent is counted twice.

### 4. Titles are 80 characters or fewer, enforced at creation

`core intent new` **rejects** an over-long title rather than warning about it. The long explanatory
clause moves into `## The Problem`.

⚠️ Enforced, not requested. Every optional-and-hopeful field in this ledger is empty.

### 5. The list is sorted by decision value, never by ID

Ready above blocked. Score, not sequence.

```
READY
★ 213  feed the intent dependency graph        high   unblocks 4
  167  devbox debugging platform               high   unblocks 2

BLOCKED
  201  devbox investigation tui                med    needs 194, 198
```

Tags leave the list view entirely. They belong in `show`.

### 6. Demotion requires a reason

Moving an intent out of `in-progress/` records why. Without it the ledger cannot distinguish
deliberately paused from quietly abandoned.

⚠️ `defer_intent` does not do this -- it defers a **gate**, appending a `⏸` line under
`## Gate Check` to satisfy INT-332. The demote verb does not exist yet.

## Deliberately not decided here

- **Renumbering.** Zero Core may restart at 001 with an era prefix. Set aside; not this decision.
- **`cicomplete`'s version prompt** (major / minor / patch / skip). Unchanged.
- **Directory names.** `future/ in-progress/ complete/ cancelled/` stay. Renaming them to
  `planned/ active/` would break `next_id`'s LIFECYCLE list, cistart, cicomplete, the README, the
  doctor's intent validator, and 240 paths, for nothing.
- **`## History` blocks.** A hand-written lifecycle log is a second owner of facts cistart and
  cicomplete already record. If history is wanted, derive the lifecycle and hand-write only
  DECISION entries -- the one thing tooling cannot infer.
- **Retrofitting.** INT-158 is forward-only. The 241 existing intents keep their current shape.

## Consequences

- `next_intent` becomes useful without being rewritten. The work is feeding it, not building it.
- Priority becomes visible and capped instead of hidden in tags and unbounded.
- The list answers "what should I do next" instead of "what exists."
- Two fields and one length rule is the entire schema change.

## Implementation

**Extends INT-213** (the dependency graph is unfed). It does not take a new number -- 213 already
owns this ground, alongside INT-211 (document shape) and INT-212 (cicomplete vs gates).

⚠️ Not a fourth in-progress thread. 222 is already active; the max-three rule applies to this
decision as much as to any other.

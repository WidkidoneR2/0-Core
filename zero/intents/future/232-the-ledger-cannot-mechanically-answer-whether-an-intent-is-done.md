---
id: 232
title: "the ledger cannot mechanically answer whether an intent is done"
status: planned
type: architecture
priority: medium
date: 2026-08-23
tags: [ledger, schema, tooling, hygiene]
---

## Vision
An intent is a TYPED DOCUMENT with orthogonal kind and state, canonical machine-readable relations,
and ONE mechanically evaluable completion gate.

## ✅ RECON FIRST, and it invalidated the proposal it came from
A redesign was sketched -- add `problem`/`goal`/`non-goals` sections, add `blocked/` and
`superseded/` directories, add a relations block, add lifecycle dates. Measuring the ledger showed
most of it ALREADY EXISTS, under several names, which makes the problem NORMALISATION rather than
SCHEMA EXPANSION.

**269 intents. EIGHT directories, not four:**

    complete 165 · future 53 · decisions 27 · incidents 12
    cancelled 8 · philosophy 2 · experiments 1 · in-progress 1

**Frontmatter keys, measured:**

    id/date/title/status 268 · type 266 · tags 266 · priority 81 · version 23
    depends_on 8 · severity 7 · cancelled_date 3 · decided/verdict/duration/affected 2 · blocks 1

**Section headings, measured:**

    vision 146 · success criteria 118 · why 99 · the rule 87 · the problem 84
    the solution 66 · gates 55 · approach 52 · notes 43 · relationship 40
    phases 31 · gate 26 · gate check 25 · depends on 20

## The Problem -- THREE, and they are different
**① KIND AND STATE ARE CONFLATED IN ONE TREE.** `decisions/`, `incidents/`, `philosophy/` and
`experiments/` are DOCUMENT KINDS. `future/`, `in-progress/`, `complete/`, `cancelled/` are
LIFECYCLE STATES. They share one directory namespace, so a decision cannot be in-progress and an
incident cannot be complete without picking which axis to express.
⚠️ **AND THE ORIGINAL PROPOSAL WOULD HAVE MADE THIS WORSE** -- adding `blocked/` and `superseded/`
deepens a conflation rather than resolving it.

**② RELATIONS HAVE TWO OWNERS.** `depends_on` (8) and `blocks` (1) live in frontmatter, where a tool
can read them. A prose `## Relationship` section (40) and `## Depends On` (20) carry the same fact
where nothing can. ⭐ Two authorities over one fact is the shape this ledger keeps removing --
two alias expanders, three selector call sites, five observability instruments.

**③⭐ THE GATE CONCEPT IS FRAGMENTED FOUR WAYS, and this is the centre of the intent:**

    success criteria 118  ·  gates 55  ·  gate 26  ·  gate check 25

★ **SO THE LEDGER CANNOT MECHANICALLY ANSWER ITS MOST IMPORTANT QUESTION: is this intent done?**
`cicomplete` cannot verify gates, `next_intent` cannot rank by remaining work, and nothing can report
an intent marked complete with unticked gates -- which is precisely the failure the audit of "123
intents marked done that were not" already found once.

## THE INVARIANTS
**① KIND AND LIFECYCLE STATE ARE ORTHOGONAL.** No directory conflates them, and none is created that
does.

**② RELATIONS HAVE ONE OWNER.** Frontmatter is the machine-readable authority. Prose may EXPLAIN a
relationship; it does not DECLARE one.

**③ ONE CANONICAL COMPLETION GATE.** One term, one meaning, consumed by tooling. Existing variants
are MIGRATION TARGETS, not permanently accepted spellings.

⚠️ **AND THE CANONICAL NAME IS NOT CHOSEN BY COUNTING.** `success criteria` is the most common at
118, and that is NOT the argument. The right name follows from what the tooling needs the field to
MEAN -- a list of conditions, each independently verifiable, each carrying evidence per INT-158. The
intent establishes the requirement; the implementation reconciles the vocabulary.

## ⚠️ REUSE WHAT EXISTS -- DO NOT PROLIFERATE SYNONYMS
`## The Problem` (84), `## Why` (99) and `## Vision` (146) already carry problem and goal. They stay.
⭐ The failure mode this intent is guarding against is ITS OWN: a normalisation pass that introduces
a fifth word for something that already has four.

## Success Criteria

- [ ] G1 THE TWO AXES ARE RULED: which of the ten directories are kinds (decisions, incidents, philosophy, experiments, findings) and which are lifecycle states (future, planned, in-progress, complete, cancelled), how a document expresses both, and that a document status agrees with its directory. Red today: planned/ holds a file with status in-progress, and decisions/ holds eleven complete and three planned. A ruling, not a migration
- [ ] G2 `date:` is measured against the git first-add date of every intent, the number that disagree is recorded, and the ruling (the date the intent was filed, never rewritten) stands or changes with that number in hand
- [ ] G3 each gate carries evidence per INT-158

## Merged into this intent (INT-294 rulings, 2026-10-22)

- INT-211 is merged here and cancelled. Its two questions are ruled: the gate has one shape (INT-294 R1, recorded in INT-212), and `date:` means the date filed; start and complete are derived from cistart, cicomplete and git (decision 142).
- The old G2 (one canonical gate term) is closed by INT-294 R1. The old G3 and G4 (an unmet-gates query and a complete-with-unticked report) moved to INT-212 G9.
- The old G5 (relations from frontmatter only) was already ruled by decision 142 sections 1 and 2: frontmatter holds facts, blocked is derived.
- The old G6 and G7 stand as rules, not gates: no new synonym for a concept that has a name, and migration is forward-only per INT-158.

## Non-goals
- ⚠️ TURNING THE LEDGER INTO A TASK TRACKER. No epics, sprints, story points, owners or estimates.
  These are files that record WHY a decision was made; that is a decision ledger, not Jira, and the
  simplicity is the value.
- Lifecycle EVENT history as a required section. A per-intent event log is an interesting idea and a
  separate one; git already records when a file moved between directories.
- Rewriting 269 intents. Forward-only.
- INT-231's dangling-citation check. Adjacent, already filed, different invariant.

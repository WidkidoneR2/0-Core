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
- [ ] G1 THE TWO AXES ARE NAMED AND RULED ON: which of the eight directories are kinds, which are
      states, and how a document expresses both. ⚠️ A ruling, not a migration -- moving 269 files is
      a separate decision
- [ ] G2 ONE CANONICAL GATE TERM IS CHOSEN, with the reason recorded, and the reason is what TOOLING
      NEEDS IT TO MEAN rather than which spelling is commonest
- [ ] G3 A TOOL CAN ANSWER "does this intent have unmet gates?" for any intent, mechanically.
      Demonstrated on a real intent, not asserted
- [ ] G4 AN INTENT MARKED COMPLETE WITH UNTICKED GATES IS REPORTABLE. ★ That is the check the
      "123 intents marked done" audit needed and did not have
- [ ] G5 RELATIONS ARE READ FROM FRONTMATTER ONLY. Prose relationship sections are not parsed, and
      the ones that exist are either migrated or left as commentary with their authority moved
- [ ] G6 NO NEW SYNONYM IS INTRODUCED for a concept that already has a name
- [ ] G7 THE MIGRATION IS INCREMENTAL AND FORWARD-ONLY per INT-158: new intents use the canonical
      form; 269 existing files are NOT rewritten in one pass
- [ ] G8 each gate carries evidence per INT-158

## Non-goals
- ⚠️ TURNING THE LEDGER INTO A TASK TRACKER. No epics, sprints, story points, owners or estimates.
  These are files that record WHY a decision was made; that is a decision ledger, not Jira, and the
  simplicity is the value.
- Lifecycle EVENT history as a required section. A per-intent event log is an interesting idea and a
  separate one; git already records when a file moved between directories.
- Rewriting 269 intents. Forward-only.
- INT-231's dangling-citation check. Adjacent, already filed, different invariant.

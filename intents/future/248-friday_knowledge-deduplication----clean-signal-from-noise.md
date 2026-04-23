---
id: 248
title: "friday_knowledge deduplication -- clean signal from noise"
status: planned
date: 2026-04-23
type: fix
tags: [fix, friday, knowledge, deduplication, data-hygiene, v11.9.0]
version: 11.9.0
---
Friday's knowledge base has grown to 615 total facts but only 167 unique.
448 duplicates -- 72.8% noise.
This intent restores signal integrity by deduplicating existing rows,
adding a UNIQUE constraint, and making all friday_knowledge writes idempotent.
WHY NOW
INT-234 gate 6 (forward-chaining inference) cannot ship on polluted data.
Chaining on duplicated facts produces false confidence -- the same fact
counted twice is not two pieces of evidence, it is one piece counted twice.
A release titled "The Mind Awakens" that reasons from noise is not honest.
The duplication also skews:
- core friday status fact count (615 reported, 167 real)
- session summary top-3 by confidence (could select 3 copies of one fact)
- any future Friday feature that reads friday_knowledge
This is the last moment to fix it before v11.9.0.
ROOT CAUSE
seed_knowledge() in engine/src/domains/friday/mod.rs runs on every
core friday status call. Over 203 hours of observation the same facts
have been re-seeded without a uniqueness constraint.
Some call sites use INSERT OR IGNORE, some use INSERT OR REPLACE,
some use plain INSERT -- inconsistent semantics across the codebase.
APPROACH
Four parts, in order:
1. Audit existing duplicates per domain to confirm scope.
2. Migrate friday_knowledge to a table with UNIQUE(domain, fact) constraint.
   Create new table, copy deduplicated rows, drop old, rename.
   Preserve earliest created_at and highest confidence per (domain, fact).
3. Normalize all INSERT sites in core to INSERT OR REPLACE.
   Later seeds with higher confidence or fresher data win.
4. Validate that COUNT(*) = COUNT(DISTINCT (domain, fact)) after migration.
IMPLEMENTATION GATES
⬜ Audit: count duplicates by domain, confirm total scope (615 -> 167)
⬜ Migration: create friday_knowledge_new with UNIQUE(domain, fact) constraint
⬜ Migration: copy deduplicated rows (earliest created_at, highest confidence)
⬜ Migration: drop old table, rename new to friday_knowledge
⬜ Normalize: all INSERT sites in engine use INSERT OR REPLACE
⬜ Validation: COUNT(*) = COUNT(DISTINCT (domain, fact)) is true
DEMONSTRATION GATES
⬜ Run core friday status 5 times consecutively, row count unchanged
⬜ friday status, friday ask, friday suggest still work without regression
BLOCKS
INT-234 gate 6 (forward-chaining inference) -- cannot ship on duplicated data.
"Signal without noise. Memory without repetition." 🌲

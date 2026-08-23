---
id: 231
title: "code cites intents that do not exist, and nothing notices"
status: planned
type: fix
priority: medium
date: 2026-08-23
tags: [ledger, deadwood, citations, hygiene]
---

## Vision
An `INT-NNN` in a comment resolves to a real intent, or something says it does not.

## The Problem
MEASURED 2026-08-23 across `faelight/rust-tools/` and `faelight/engine/`:

    206 distinct INT-NNN cited in source
    229 intents filed
     61 CITED BUT NOT FILED

⚠️ AND THEY ARE NOT ONE POPULATION. Treating them as "61 missing intents" would produce the wrong
fix, so the census separates three conditions:

**~59 FORWARD-INVENTED (233 and above).** The highest filed intent is 230, and the ledger is nearly
contiguous below it -- only two gaps in 230 numbers. So these cite a numbering space THE LEDGER HAD
NOT REACHED. They were not lost; they were invented, written into code by hand or by a generator.
⭐ Git supports this: no commit ever adds an intent file for INT-233, and the citations arrive via
the Phase 1 tree move and a changelog-regeneration pass whose own message mentions files "full of
INT-numbers".

**ONE HISTORICAL GAP (INT-180).** A real hole BELOW the highest filed number, which makes it the
only genuine candidate for a lost intent. ⚠️ It is an INVESTIGATION ITEM, not a bug to fix.

**ONE PLACEHOLDER (INT-000).** Reserved by convention, not a defect.

★ THE CONTIGUOUS-NUMBERING OBSERVATION IS RECONNAISSANCE, NOT THE INVARIANT. A check that asked
"is the number <= the highest filed?" would wrongly PASS INT-180, which is precisely the one worth
looking at.

## THE INVARIANT
**Every intent citation must resolve to an existing intent in the ledger, unless the reference is an
explicitly classified placeholder.**

The question the check asks is `does INT-NNN exist?` -- not whether its number looks plausible.

## ⚠️ THE CHECK EXPOSES; IT DOES NOT MATERIALISE
⭐ THE 59 FORWARD CITATIONS ARE NOT RETROACTIVELY TURNED INTO INTENTS TO SATISFY THE CHECKER. That
would invert the ledger's purpose: an intent records a decision someone made, and manufacturing one
because code mentions a number produces a file with no decision in it. The check REPORTS them with
source locations; what happens to each is a separate judgement -- remove the citation, renumber it
to the intent that actually owns the work, or file a real intent because the work is real.

## Where it lives
`faelight-deadwood` already detects dead aliases and structural orphans. ⭐ A dangling intent
citation is the same family: a reference whose target does not exist. It belongs there rather than
in a new tool.

## Success Criteria
- [ ] G1 THE CHECK ASKS THE RIGHT QUESTION: does the cited intent EXIST, not whether its number is
      below some maximum. Proven by INT-180 being reported -- it would pass a range check
- [ ] G2 THE THREE CONDITIONS ARE REPORTED SEPARATELY, not as one count: forward-invented,
      historical gap, allowed placeholder. A single number would hide the finding
- [ ] G3 EVERY REPORT CARRIES ITS SOURCE LOCATION, so a reader can judge each citation rather than
      trust a total
- [ ] G4 `INT-000` IS AN EXPLICIT ALLOWED FORM, recorded as such rather than special-cased silently
- [ ] G5 RED FIRST: the check reports the 61 that exist today. ⚠️ It goes green by the citations
      being RESOLVED, never by the checker being loosened
- [ ] G6 NO INTENT IS FILED MERELY BECAUSE CODE CITES ITS NUMBER
- [ ] G7 INT-180 IS INVESTIGATED AND ITS STATUS RECORDED: lost, never filed, or renumbered. One
      line of finding, whichever it is
- [ ] G8 each gate carries evidence per INT-158

## Non-goals
- Fixing all 61 citations. The CHECK is this intent; the resolutions are judgement calls, and some
  may point at real work that deserves a real intent.
- Renumbering the ledger. The gaps at 47 and 180 stay gaps; history is not tidied.
- ⚠️ THE LEDGER FORMAT REDESIGN. Lifecycle dates, problem/goal/non-goals, relations, decision
  events, outcome sections and the `superseded`/`blocked` states are a SEPARATE and much larger
  question with its own existing notes. This intent is one check.

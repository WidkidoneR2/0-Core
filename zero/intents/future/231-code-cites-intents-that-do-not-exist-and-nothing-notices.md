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
- [x] G1 THE CHECK ASKS THE RIGHT QUESTION: does the cited intent EXIST
<!-- `check_dangling_intent_citations` builds the set of filed numbers from the ledger's filenames
     and asks `filed.contains(&n)`. INT-180 IS REPORTED, which is the proof: it sits below the
     highest filed number and a range check would have passed it silently. -->
- [x] G2 THE THREE CONDITIONS ARE REPORTED SEPARATELY
<!-- Live output: `INT-180 [gap]` at HIGH confidence, `INT-233 [forward]` and the rest at MEDIUM.
     ⭐ AND THE SEPARATION EARNED ITSELF IMMEDIATELY -- one `[gap]` line among fifty-six `[forward]`
     ones is visible; buried in a total of 57 it would not have been. -->
- [x] G3 EVERY REPORT CARRIES ITS SOURCE LOCATION
<!-- Each finding lists file:line for every site, e.g. `INT-233 [forward] cited 5 time(s):
     mod.rs:6447, mod.rs:9013, mod.rs:10079, exec.rs:494, main.rs:2216`.
     ★ THAT IS WHAT MADE G7 POSSIBLE: reading INT-180's three locations recovered what the intent
     was about. A total would have recovered nothing. -->
- [x] G4 `INT-000` IS AN EXPLICIT ALLOWED FORM
<!-- Skipped by name in the loop with the reason on the line, not filtered upstream where a reader
     would not see it. -->
- [ ] G5 RED FIRST: 57 reported on the first run -- STILL RED, and correctly so
<!-- ⚠️ 57, NOT THE 61 THE INTENT PREDICTED. The intent's number came from a shell pipeline counting
     distinct matches across a wider net; the check walks `.rs` files under rust-tools and engine
     only. THE CHECK'S NUMBER IS THE ACCURATE ONE, and the discrepancy is recorded rather than
     quietly adopted -- a census that changes its own headline without saying so is the thing this
     ledger keeps catching.
     ⚠️⚠️ THIS GATE IS NOT MET AND THE TICK WAS WRONG. "Red first" is the METHOD; the gate is that
     the citations are RESOLVED. Fifty-seven are still reported, so INT-231 is NOT complete -- the
     CHECK is built, the cleanup is separate work, and marking this done would be exactly the
     "123 intents marked done that were not" failure the audit already found once. -->
- [x] G6 NO INTENT IS FILED MERELY BECAUSE CODE CITES ITS NUMBER
<!-- Fifty-seven numbers are cited and zero intents were created. INT-180's SUBJECT is recovered
     into this intent's evidence, which is not the same as manufacturing a ledger entry for it: a
     file with no decision in it would satisfy the checker and record nothing. -->
- [x] G7 INT-180 INVESTIGATED -- AND RECOVERED FROM THE CODE
<!-- ⭐⭐ THE CHECK ANSWERED THIS ON ITS FIRST RUN. INT-180 is cited THREE times in the engine
     (the deadwood doc comment is a fourth, and correctly counted -- a checker exempting itself from
     its own rule would be the defect):
       checkpoint/mod.rs:115   // INT-180: sway removed
       checkpoint/mod.rs:505   // INT-180: sway removed
       lock/mod.rs:12          "swaylock"  // Niri via ext-session-lock (INT-180)
     ★ SO INT-180 WAS THE SWAY REMOVAL -- the compositor migration to Niri, with session locking
     moving to ext-session-lock. Real, completed work whose ledger entry is gone while the code
     kept the decision. STATUS: LOST, and its subject is recovered.
     📍 AND A LIVE DISCREPANCY FOUND ALONGSIDE IT: lock/mod.rs:12 comments Niri but invokes
     `swaylock`. Possibly correct -- swaylock runs under other compositors -- but the comment and
     the code name different eras. Recorded, not chased. -->
- [x] G8 each gate carries evidence per INT-158
<!-- this block. -->

## Non-goals
- Fixing all 61 citations. The CHECK is this intent; the resolutions are judgement calls, and some
  may point at real work that deserves a real intent.
- Renumbering the ledger. The gaps at 47 and 180 stay gaps; history is not tidied.
- ⚠️ THE LEDGER FORMAT REDESIGN. Lifecycle dates, problem/goal/non-goals, relations, decision
  events, outcome sections and the `superseded`/`blocked` states are a SEPARATE and much larger
  question with its own existing notes. This intent is one check.

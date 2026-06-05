---
id: 250
title: "Release tool intelligence layer -- faelight-release and faelight-docs translate, not just account"
status: complete
superseded_by: INT-264
date: 2026-04-24
type: infrastructure
tags: [release, docs, translation, public-voice, faelight-release, faelight-docs]
version: unplanned
---
## Status
This intent has been merged into INT-264 (faelight-synthesis).
INT-264 fully defines the solution this intent identified.
All gates and context absorbed into INT-264.
---
faelight-release and faelight-docs currently do accounting. They count commits,
list intents by number, generate sections, and produce technically-correct
changelogs. But technically correct is not the same as useful.
When I previewed v11.9.0 tonight, the changelog exposed raw INT-NNN numbers
directly to the public. It undercounted fix work because many fixes lived
inside larger intents and got flattened into single-line commit summaries.
The narrative generator picked the biggest intents to summarize and missed
the philosophical weight of small commits that mattered more than their
size (FSH-PHILOSOPHY.md as an example).
The gap is translation. These tools speak in internal vocabulary -- intent
numbers, commit hashes, gate counts. A changelog is a public artifact. It
needs to speak in the language of what changed for the reader, not what
changed in my ledger.
1. **Changelog exposes internal INT-NNN numbers.** Strangers reading the
   changelog see "INT-180 complete", "INT-233 fixes", "INT-241 work" with
   no translation. These are private accounting references. Public
   changelogs need human-readable summaries: "Sway removed, full Niri
   commitment" not "INT-180 complete".
2. **Fix count undercounts actual fix work.** INT-241 (integrity engine
   audit) contained many fixes. INT-233 (fsh v8) contained many fixes.
   Both shipped as "one intent" in the preview, hiding the real volume
   of fix work. The count "2 fixes" in v11.9.0 preview is technically
   correct (only 2 commits used the `fix:` prefix since the last tag)
   but deeply misleading about what the release actually repaired.
3. **Narrative generation misses philosophical weight.** Today's three
   commits -- FSH-PHILOSOPHY.md, extract-patterns honest counts, fsearch
   path-scoping -- carry more philosophical weight for the project than
   some of the larger intents they ship alongside. The auto-generated
   narrative weights by commit count and intent size, not by meaning.
4. **Release themes are generic.** "The Shell Remembers" / "Roots and
   Signals" / "The Living Toolkit" are all defensible themes, but they
   read like templates. A release theme should emerge from the actual
   texture of what changed, not from a pool of reusable phrases.
This intent covers faelight-release AND faelight-docs because they work
as a pair. faelight-release produces the changelog and manifest.
faelight-docs syncs the README and welcome page. Together they are the
public voice of the forest. Separating them creates coordination debt.
This intent IS:
- A rewrite of the changelog narrative layer -- translation, not listing
- A synthesis system that reads intent titles/descriptions and produces
  human summaries, not raw INT-NNN references
- A weighted fix counter that reads commit messages inside referenced
  intents, not just top-level `fix:` commits
- A theme suggestor that reads the actual commits of the release window,
  not a phrase pool
- Coordination between release and docs so the welcome page and
  changelog tell the same story
This intent is NOT:
- A full rewrite of faelight-release or faelight-docs
- A replacement for the existing release flow (manifest, git tags,
  rollback should keep working)
- An LLM-based changelog (stays local, reads forest state)
To be filled in when I return to this intent. Placeholder:
⬜ Audit current changelog output for every INT-NNN leak
⬜ Design translation layer: intent title -> human summary
⬜ Fix counter reads commit messages within referenced intent range
⬜ Narrative generator weights by philosophical signals, not commit count
⬜ Theme suggestor uses real release window, not template pool
⬜ faelight-docs and faelight-release share synthesis output
⬜ Welcome page and CHANGELOG tell the same story for a given release
⬜ Public-facing output contains no raw INT-NNN references without
   human-readable context
Surfaced during v11.9.0 preview session 2026-04-24. Three commits that
day were philosophically significant but buried in the release preview
beneath 10 days of older work. The release preview was technically
correct and spiritually wrong.
🌲

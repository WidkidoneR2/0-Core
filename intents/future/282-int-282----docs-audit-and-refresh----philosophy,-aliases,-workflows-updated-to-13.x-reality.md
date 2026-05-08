---
id: 282
title: "Docs Audit and Refresh -- Philosophy, Aliases, Workflows updated to 13.x reality"
status: in-progress
date: 2026-05-07
tags: [docs, audit, philosophy, aliases, workflows, cleanup, maintenance]
---
The forest has changed enormously since many of these docs were written.
The docs have not kept up.
v8.4.0 / Sway / January 27 -- that is what PHILOSOPHY.md still says.
The forest is at 13.1.0 / Niri / May 2026 now.
Every doc that is wrong is a liability.
Every doc that is right is an asset.
This intent sorts one from the other.
---
DOCS TO ARCHIVE (move to docs/archive/ -- not delete)
archaeology-0-core audit docs -- any remaining references
FSH-V9-ARCHITECTURE-AUDIT-A.md -- fsh v0.7.0 audit, April 24, obsolete
SHELL-LAYER-AUDIT.md -- INT-162 Phase 1, March 31, file referenced no longer exists
These were point-in-time audits. Their value is historical, not operational.
Move to docs/archive/. Keep in git history. Remove from active docs.
---
DOCS TO MERGE AND REMOVE
core-commands.md -- duplicate of COMMAND-GUIDE.md
  Check for any unique content not in COMMAND-GUIDE.md.
  Merge unique content. Remove core-commands.md.
  One command reference, not two.
---
DOCS TO UPDATE
PHILOSOPHY.md (currently v8.4.0, 2026-01-27, references Sway):
  Update version to 13.x
  Replace Sway references with Niri
  Add Friday as a pillar -- the forest now has an intelligence layer
  Add fsh as daily driver -- login shell since 2026-04-03
  Preserve the four core principles -- they have not changed
  Add the new principles that have emerged: forest speaks human first,
  the ledger knows itself, trust is earned not assumed
ALIASES.md (likely based on ~300 aliases, now 368+):
  Update alias count
  Reflect fsh vocabulary additions (human words: delete/find/rename/make/launch/replace)
  Remove any aliases for retired tools
  Add new forest commands: compare, pick, cheat, core intent blocked/next/brief/graph
WORKFLOWS.md (likely missing current workflow):
  Update cistart/cicomplete to show dependency enforcement and retrospective
  Update deploy workflow with current deploy intelligence
  Add fg done workflow
  Add core intent next as standard session-start
  Add Super+Ctrl+Escape for lock
  Remove any Sway/swaylock references
---
DOCS TO REVIEW (keep or archive based on content)
TESTING.md -- is there an active test suite? If not, archive.
MANUAL_INSTALLATION.md -- still accurate? Verify.
SCRIPTING-STORY.md -- what is this? Read and decide.
AUTOSTART-MAP.md -- verify against current Niri autostart config.
THEORY_OF_OPERATION.md -- may still be valuable. Review.
THEMING.md -- forest colors still the same. Probably still accurate.
KEYBINDINGS.md -- superseded by cheat TUI (core cheat). Archive.
FAELIGHT-SHELL-GUIDE.md -- may be superseded by cheat. Review.
FAELIGHT-SHELL.md -- duplicate of GUIDE? Review.
FSH-PHILOSOPHY.md -- merge into PHILOSOPHY.md? Review.
SHELL-GRAMMAR.md -- is this still accurate for fsh v2.1.0?
RELEASE.md -- is this the release process? Verify against current workflow.
POLICIES.md -- review for accuracy.
---
GATES
[ ] docs/archive/ directory created
[ ] FSH-V9-ARCHITECTURE-AUDIT-A.md moved to archive
[ ] SHELL-LAYER-AUDIT.md moved to archive
[ ] core-commands.md unique content checked, merged if needed, removed
[ ] PHILOSOPHY.md updated to 13.x / Niri / Friday / fsh
[ ] ALIASES.md updated for 368+ aliases and current vocabulary
[ ] WORKFLOWS.md updated for current cistart/cicomplete/deploy workflow
[ ] All remaining docs reviewed -- each either updated or archived
[ ] No doc references Sway, swaylock, or pre-Niri tooling
[ ] No doc references retired tools (archaeology-0-core, etc.)
[ ] d shows 100% health after all changes
"A doc that is wrong is worse than no doc.
It teaches the wrong thing with confidence.
The forest documents what is true.
Nothing else." 🌲

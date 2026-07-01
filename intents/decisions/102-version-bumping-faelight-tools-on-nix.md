---
id: 102
date: 2026-07-01
type: decisions
title: "Version-bumping Faelight Tools on Nix"
status: planned
tags: [tools, bump, version, Nix, Faelight]
---

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

---

## Why
Tool versions drift stale because the only bump path is faelight-release, a full
FOREST-RELEASE ceremony (publish TUI, changelog, generation triad, rollback
points). That is far too heavy for "one tool got a patch." Result: faelight-shell
sat at 2.5.0 through BOTH INT-100 (parser fix) and INT-101 (schema fix) -- two
real shipped fixes, no version movement. There is no lightweight per-tool bump.

## The core conceptual split (the root of the problem)
NixOS conflated two DIFFERENT axes:
- TOOL VERSION = "this artifact's code changed" (faelight-shell 2.5.0 -> 2.5.1)
- FOREST GENERATION = "the whole-system state at gen N" (faelight-release's domain)
These are orthogonal. A tool patch should not require a forest release; a forest
release snapshots whatever tool versions exist at that moment. Today they're welded
together through faelight-release, so per-tool versioning has no home.

## Design space (decide before building)
PHILOSOPHY 1 -- per-tool semver, DECOUPLED from forest releases [LEANING]
  Each tool owns its version; bumps when ITS code changes, independent of forest
  generations. faelight-release stays a separate axis that snapshots the forest.
  The bump-versions registry already wants to be this -- the lightweight per-tool
  bump ACTION is what's missing.
PHILOSOPHY 2 -- versions bump ONLY at forest release
  Simpler but coarse; tool versions don't reflect individual fixes. This is the
  current de-facto state and the thing causing the frustration.
PHILOSOPHY 3 -- auto-bump on change via cicomplete
  cicomplete already SUGGESTS bumps ("faelight-shell patch or minor"). Close the
  loop: cicomplete detects which tools an intent's commits touched and bumps them.

## The hard sub-problem: WHO decides patch vs minor vs major?
This is a SEMANTIC judgment a tool can't fully automate. cicomplete suggested
"patch OR minor" for faelight-shell precisely because it can't tell a bug fix from
a feature. Options:
- (a) Human declares the level at cicomplete time (one prompt: patch/minor/major).
- (b) Convention via intent `type:` field -- intents ALREADY have type
  (feature/infrastructure/polish/future/decisions...). Map type -> semver:
  bugfix/polish -> patch, feature -> minor, breaking -> major. The deciding
  METADATA MAY ALREADY EXIST in the ledger. This is the promising lead.

## Integration question
Should cicomplete close the loop end-to-end: on intent completion, detect touched
tools (from the intent's commits), read the intent's type, bump each touched tool
by the mapped semver level, and record it? That makes versioning a natural
byproduct of the intent lifecycle instead of a forgotten manual chore.

## Where versions live (recon needed)
- bump-versions (display alias) reads a registry -- WHERE? (state.db? a versions
  file? faelight-release's triad?) Find the source of truth before designing writes.
- faelight-release commands: publish/plan/preview/status/history/query/gc-check/
  rollback/diff -- understand how it currently reads+writes versions.

## Gates (when built)
- [ ] Source-of-truth for tool versions located + documented
- [ ] A lightweight per-tool bump exists (NOT the full release ceremony)
- [ ] patch/minor/major decision mechanism chosen (human prompt or type-mapping)
- [ ] faelight-shell bumped to reflect INT-100/101 (retroactive first use)
- [ ] Forest-release vs tool-version axes cleanly separated (documented)

## Deferred
This intent CAPTURES the design space; a follow-up (or this one's later phase)
DECIDES the philosophy + builds. Do not build until the philosophy is chosen.

## Notes
Surfaced 2026-07-01 trying to bump faelight-shell after closing INT-100/101 --
found bump-versions is display-only and faelight-release is a full ceremony, with
no lightweight middle. Christian: "we need to figure out a systematic way to where
tool versions could be bumped up -- but deciding how is another thing."

## ============================================================
## DECIDED ARCHITECTURE (2026-07-01) -- supersedes the open design space above
## ============================================================
Christian designed the resolution; refined together. The design-space section
above is retained as rationale/history. The decisions below are the plan.

### Decision 1: two orthogonal axes, forest release SNAPSHOTS (never bumps)
Nix already separates these; the forest mirrors it:
- TOOL VERSION -- semver per tool (faelight-shell 2.5.1), bumped when ITS code
  changes. Independent of system generations.
- FOREST RELEASE -- a SNAPSHOT that RECORDS whatever tool versions currently
  exist at a generation. It changes NOTHING. Analogous to flake.lock pinning
  inputs. e.g. forest-342.json { generation: 342, shell: "2.5.1", db: "1.8.0",
  tui: "0.14.2" }.
faelight-release stops being the only bump path; it becomes a pure recorder.
A tool patch NEVER requires a forest release.

### Decision 2: version lives in ONE place Nix already reads (single source of truth)
CONSTRAINT (learned from the 104 shell_snapshots dual-schema bug -- do NOT recreate
that at the version layer): a tool's version must have EXACTLY ONE source of truth,
and it must be the place Nix already reads (the derivation/flake attribute, e.g.
packages.faelight-shell.version). `bump-version faelight-shell patch` edits THAT.
- A separate per-tool version.toml is allowed ONLY IF the flake IMPORTS from it
  (so Nix reads it -> it's the single source). Never both a .toml AND a flake attr
  that can drift. Decide at build time which physical location; the rule is
  one-source-Nix-reads-it.

### Decision 3: stored semver for RELEASES, git-describe for the DEV suffix
Reconcile "store the version" vs "derive from git" -- they compose as base+suffix,
not either/or:
- The tool's BASE version is STORED (2.5.1) -- authoritative, human-set at bump.
- Between releases, dev builds may APPEND git info: 2.5.1-14-g4b2a18d (git describe).
- Only releases receive permanent stored semver. Nightly/dev = stored-base + git.
So stored is authoritative; git-describe decorates dev builds. No conflict.

### Decision 4: suggest, never auto-decide (human owns semver)
cicomplete/CI DETECTS changed tools and SUGGESTS, but the human picks the level:
  Intent INT-NNN touched: faelight-shell, faelight-db
  Suggested: faelight-shell patch, faelight-db minor   Accept? [y/edit]
The human can override every level. The tool never finalizes semver alone.

### Decision 5: REJECTED -- do NOT map intent `type:` to semver level
(Corrects the "promising lead" floated in the design space above.) Intent type
answers WHY a change was made; semver answers API COMPATIBILITY. Different axes.
- type: feature can be hidden / experimental / internal / breaking / reverted.
- type: infrastructure can require a MAJOR bump if behavior changes.
Mapping type->semver produces confidently-WRONG versions that LOOK principled.
Worse than no automation. The human decides compatibility; type does not encode it.

### THE HARD PART (the real engineering; everything else is data-recording)
"Detect which tools changed" needs a concrete mechanism. Options to evaluate:
- git-diff each tool's source dir since its last version bump/tag, OR
- track touched paths per intent commit (map commit -> tool by path prefix).
Detection is the core build-work. Suggestion + recording + the snapshot file are
straightforward once detection is solid.

### Revised pipeline (Christian's)
  intent complete -> detect changed tools -> suggest semver bump -> user confirms
  -> update tool version (single source Nix reads) -> commit.
  LATER: forest release -> snapshot current versions (forest-N.json). Release
  NEVER bumps; it only records.

### Revised gates (supersede the earlier gate list)
- [ ] Single source of truth for each tool's version, in a place Nix reads
- [ ] `bump-version <tool> <patch|minor|major>` edits ONLY that source
- [ ] "detect changed tools" mechanism chosen + working (the hard part)
- [ ] cicomplete/CI SUGGESTS bumps on intent completion; human confirms/overrides
- [ ] intent-type is NOT used to auto-pick semver level (explicitly avoided)
- [ ] forest release RECORDS a version snapshot (forest-N.json) and bumps nothing
- [ ] dev builds may carry git-describe suffix over the stored base version
- [ ] faelight-shell bumped to 2.5.1 as the first real use (retroactive: INT-100/101)

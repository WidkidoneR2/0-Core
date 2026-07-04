---
id: 121
date: 2026-07-04
type: decision
title: "Faelight Forest release process and naming convention"
status: complete
tags: [release, versioning, naming, process, decision, faelight]
---

## Decision
Two things are settled here: (1) HOW Faelight Forest does releases (a repeatable process,
so no release is ever "random things thrown together"), and (2) HOW releases are NAMED
(semver + a single forest-native codename).

## The release process (the "no random things" discipline)
1. TRIGGER -- a release is cut by a DELIBERATE, recorded decision. Never ambient/whim.
2. INCLUSION CRITERION -- a change is IN a release only if it is:
     stable + DEMONSTRATED (not merely declared) + not mid-flight.
   Fresh feature work and in-progress intents are OUT -> they wait for the next release.
   This single rule auto-sorts scope (it is what sends daemon/shell-feature/experimental
   intents to the NEXT version instead of the current one).
3. VERSION -- semver MAJOR.MINOR.PATCH. Tool versions must be ACCURATE before a release
   -> INT-111 (real per-tool version write path) is a HARD PREREQUISITE. Cannot cut a
   release on suggestion-theater versions.
4. CODENAME -- every release gets a single forest-native name, chosen deliberately.
5. FLOW -- faelight-release `plan` (dry-run: see exactly what will happen) -> `preview`
   (check the auto-generated changelog) -> verify -> `publish`. NEVER publish as the
   first test of the tooling.
6. SAFETY -- `rollback` + `gc-check` protect the release generation from garbage
   collection (the version triad, INT-034).

## Naming convention
Format:  Faelight OS  MAJOR.MINOR.PATCH  "Codename"
- Semver ORDERS the releases; the forest-name gives each one SOUL.
- Date is METADATA only ("released YYYY-MM-DD"), NEVER the version number.
- Codename theme: single forest-native names -- trees, fae, forest-light phenomena
  (Rowan, Foxfire, Heartwood, Alder, Yew, Hawthorn, Emberlight, Mosswind, ...).

## Why this theme (reasoning recorded)
- RICHEST + most Faelight: names come from the world the project already built
  ("the forest remembers"), not an imported identity.
- SELF-GENERATING: the forest always supplies another name -- no blank-page problem.
- SCALES: single names stay short, distinct, and memorable across 30+ releases.

## Rejected alternatives (recorded so they do not resurface)
- DATE-VERSIONING (e.g. 26.07.03) -- collides with NixOS's OWN year.month scheme
  (26.05 "Yarara"), and reintroduces the version-axis confusion INT-111 resolved.
- IMPORTED PANTHEONS (Greek/Roman/Egyptian gods) -- consistent but BORROWED identity;
  says nothing about Faelight's own world.
- TWO-WORD PHRASES (the old "14.1.0 -- Research and Resilience" style) -- evocative but
  harder to keep fresh/distinct past ~30 releases; single forest names scale better.

## The three version axes (referenced so this convention never re-conflates them)
Orthogonal -- see INT-111 for the full axiom:
- TOOL SEMVER        e.g. faelight-shell 2.5.0  (an artifact's code changed)
- RELEASE + CODENAME e.g. Faelight OS 1.0.0 "Morphwood"  (whole-system milestone)
- GENERATION COUNTER e.g. gen 300  (disposable system-rebuild count; means nothing about
                                    tool maturity; resets on fresh install)

## Inaugural release
Faelight OS 1.0.0 "MORPHWOOD" -- the forest that changed form.
Commemorates the Arch -> NixOS 26.05 (Yarara) migration + the full de-Arch
(INT-082/116: zero executable Arch code, demonstrated by sweep). The name earns its
moment: this is the release where the forest shed its old substrate and became native.

## Codename pool (held for future releases)
Mosswind (strong, atmospheric -- held for a future version), plus the forest supplies
more as needed (Rowan, Foxfire, Heartwood, Alder, Yew, Hawthorn, Emberlight, ...).

## Relates to
- INT-111 (per-tool versioning -- the hard prerequisite for accurate release versions).
- decisions/002-versioning-strategy-clarification (earlier versioning decision -- this
  supersedes/extends it for the release + naming layer).
- faelight-release (plan/preview/publish/rollback/query/gc-check -- the tool that
  executes this process).

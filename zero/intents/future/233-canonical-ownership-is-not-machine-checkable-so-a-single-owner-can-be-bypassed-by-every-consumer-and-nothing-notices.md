---
id: 233
date: 2026-08-27
type: arch
title: "canonical ownership is not machine-checkable, so a single owner can be bypassed by every consumer and nothing notices"
status: planned
tags: [architecture, rust, design]
---

## The Problem

A repository-owned accessor exists, is correct, and every consumer builds the
thing itself. The owner is bypassed, the bypass is invisible, and the drift
only surfaces when something breaks months later.

FOUR INSTANCES MEASURED IN ONE DAY, 2026-08-27:

- paths::scripts_dir() -- 17 call sites, only 2 used the accessor. The other 15
  built the string in six spellings. The directory it named was deleted in
  e733287d and six sites were EXECUTING a binary there.
- paths::bin_dir() -- did not exist. 36 sites decided where a binary lives,
  across five layouts.
- paths::aliases_file() -- a grep reported ZERO callers. The COMPILER found
  two, in doctor/aliases.rs and doctor/checks.rs, because the search used the
  wrong string. Nine sites read the shell config across three locations.
- fpatch -- AGENTS.md says edits go through it. Roughly fifteen edits that day
  went through hand-rolled python instead, reimplementing its anchor handling
  badly and omitting its read-back verification.

## Two invariants, not one

Collapsing these into one detector would produce something that does neither
well. They are related and distinct:

MUTATION DISCIPLINE -- changes to files must go through the controlled mutation
primitive. This is about HOW state changes. fpatch belongs here.

OWNERSHIP DISCIPLINE -- one owner per concept or path, and consumers go through
it. This is about WHERE KNOWLEDGE LIVES. scripts_dir, bin_dir, aliases_file and
shell_config belong here.

This intent is OWNERSHIP. Mutation discipline needs its own.

## The design constraint that matters

The obvious shape is a declared forbidden-strings list per owner:

    owner: shell_config()
    forbidden: ".config/faelight-shell/config.fsh"

THAT IS THE SHAPE THAT ALREADY FAILED. check_scripts matched by file name.
INT-218 found a live defect escaping a six-name list precisely because the list
was hand-maintained. The aliases_file miss above came from searching for a
remembered string. A forbidden list is a hardcoded census and inherits the
census failure mode: it catches what someone remembered.

The stronger form is DERIVED, not declared. shell_config() assembles a path
from known components. Any other site assembling those same components is a
bypass, and that is computable from the accessor body. Declaring the owner
should be enough; the detector derives what a bypass looks like.

Whether that is achievable is the open question this intent exists to answer.

## Acceptance test

WHEN A FIFTH CANONICAL OWNER IS ADDED SIX MONTHS FROM NOW, CAN ITS OWNERSHIP
RULE BE DECLARED WITHOUT WRITING ANOTHER BESPOKE DETECTOR?

If yes, this is an architectural primitive that pays for itself repeatedly. If
no, it is documentation of four examples and should not be built.

## Success Criteria
- [ ] An owner can be declared in one place, machine-readably
- [ ] Bypasses are DERIVED from the owner, not listed alongside it
- [ ] Watch it fail first: run the detector against e733287d and confirm it
      finds the 15 scripts_dir bypasses that existed then
- [ ] A fifth owner is added with no new detector code
- [ ] Reports, never rewrites -- same contract as the rest of deadwood

## Related

- INT-218 -- deadwood scoped a check by FILE while the rule was defined by ROLE.
  Same disease, one instance. This generalizes it.
- INT-231 -- code cites intents that do not exist. RELEVANT IMMEDIATELY: five
  comments already cite INT-233 (mod.rs:6447, mod.rs:9013, mod.rs:10079,
  exec.rs:513, main.rs:2216) for a DIFFERENT 233 that was never filed. This
  number is now taken by something else.

## Not this intent

- Mutation discipline and fpatch enforcement. Separate invariant.
- The fpatch transmission gap: patch_between protects its anchors, but
  new_lines still crosses the shell boundary as typed text, which INT-203
  proved can be corrupted by brace expansion in heredoc bodies under a quoted
  delimiter. fpatch.py is the WRONG LAYER to fix that -- the corruption happens
  before python receives the argument. Its read-back verification is correctly
  positioned as the last defense. That finding belongs with the fsh paste work.

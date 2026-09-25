---
id: 224
date: 2026-08-17
type: arch
title: "new code uses zero naming and agents.md enforces it"
status: planned
priority: medium
depends_on: []
tags: [architecture, rust, design]
---

## Vision

Every new tool, crate, directory and variable created from here on carries the Zero Core naming
registers, and a mechanical check says so rather than a person remembering.

⚠️ NOTHING EXISTING IS RENAMED. That is not a deferral, it is the point. This intent is entirely
forward-only.

## The Problem

Three spellings of the name appeared within one day: `ZERO-CORE` on the logo wordmark, `Zero Core`
in decisions 142 and 143, and `zero-shell` as the natural tool form. Left alone, all three
propagate into documentation, crate names, the boot entry, the prompt, and eventually a package
someone else installs.

Naming is cheap exactly once -- at creation. Every hour after that it gets more expensive, and the
ledger already paid that bill: two numbering eras shared one `INT-` prefix, the Arch-era archive
went missing, and 60+ code citations became unresolvable.

Decision 144 settles which form belongs where. This intent makes the decision take effect without
anyone having to remember it.

⏭ NOTE ON LINKING: this cannot declare `depends_on: [144]`, because the lifecycle dirs and
`decisions/` run separate counters -- **decision 144 and intent 144 both exist.** The reference is
therefore in prose. `depends_on: []` is written explicitly because an empty list is a decision and a
missing field is how 239 of 241 intents ended up unfed.

## The Solution

Put the register table in `AGENTS.md`, then give `faelight-deadwood` a check that flags a NEW
`faelight-*` identifier introduced after a cutoff date.

★ The cutoff is what makes this cheap. A blanket check would report ~34 crates and ~55 domains and
be ignored within a day. A dated check reports only what someone just created, which is the only
moment the fix costs nothing.

## Success Criteria

- [ ] The register table from decision 144 is in `AGENTS.md`, in the Naming section, with the note
      that `ZERO-CORE` is wordmark lettering rather than a name.
- [ ] `AGENTS.md` states the forward-only rule explicitly: new things are `zero-*`, existing
      `faelight-*` identifiers are left alone, and a rename is a separate deliberate act.
- [ ] The next tool or crate created in this repository is named `zero-*`. **Demonstrated by the
      artifact, not by intention** -- the first one to appear is the evidence.
- [ ] `faelight-deadwood` flags a NEW `faelight-*` crate, binary or top-level identifier created
      after the cutoff date. **Proven by watching it fail first:** create one, watch it be flagged,
      remove it, watch the flag clear.
- [ ] ★ **The anti-gate: nothing existing was renamed.** A count of `faelight` identifiers taken
      before and after this intent is identical. If the number moved, this intent did something it
      was explicitly not allowed to do.
- [ ] The env-var form is settled in writing before the first one exists: `ZERO_*` for new
      variables, `FAELIGHT_*` untouched, and no compatibility shim written speculatively.

## Prior art -- do not duplicate

- **decisions/144** -- the registers and the migration policy. This intent implements its
  forward-only half and nothing else.
- **decisions/143** -- the layer tree. Naming does not move directories.
- **INT-218** -- deadwood scopes a check by file while the rule it enforces is defined by role.
  ⚠️ Same trap available here: scope the new check by ROLE (new identifier), not by file list.
- **INT-107** -- decommissioned the stow subsystem, and its leftovers are still being found six
  weeks later. The cost of a migration nobody finished.

## Non-goals

- Renaming `faelight-*` anything. Not one crate, not one directory, not one variable.
- Renaming the `core` command to `zero`. That is an existing command and therefore a migration,
  not a naming rule.
- Moving `~/.config/faelight*` or anything in `state.db`. Persistent data is a data migration with
  its own plan, per decision 144.
- Changing the logo. It is already correct -- letterspaced capitals are typography, not a name.
- Deciding when the full Faelight to Zero migration happens.

## Risk

`user`. This intent writes documentation and one mechanical check. Nothing it does can affect boot,
login, disk, or a running service.

⚠️ The one way it could go wrong is scope creep -- a rename starting "while we are in here." The
anti-gate above exists to catch exactly that, and it is the gate to check first if this intent ever
feels larger than it should.

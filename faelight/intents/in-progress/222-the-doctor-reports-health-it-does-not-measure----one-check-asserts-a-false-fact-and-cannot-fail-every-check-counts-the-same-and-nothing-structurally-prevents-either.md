---
id: 222
date: 2026-08-17
type: arch
title: "the doctor reports health it does not measure -- one check asserts a false fact and cannot fail, every check counts the same, and nothing structurally prevents either"
status: in-progress
tags: [architecture, rust, design]
---

## Vision

The doctor becomes a scan engine with declared definitions, the way anti-virus software works:
one runner, a set of checks expressed as data, a quick scan and a full scan, per-item findings,
and one verdict that states its basis rather than a bare percentage.

The goal is not more coverage. It is that **a check which cannot fail becomes impossible to write
rather than difficult to find**, and that the number at the end means something a reader can act on.

⚠️ AND MOST OF IT ALREADY WORKS. 34 checks are wired and running, and every one sampled apart from
`check_dotmeta` does real work. This is instrumentation and consolidation, not repair. The engine is
not being rewritten; the shape of a check is being declared.

## The Problem

`d` reports a health percentage built from 34 checks. Three things are wrong with that number, and
they get worse in order.

### 1. One check asserts a fact it never verifies

`faelight/engine/src/domains/doctor/checks.rs:359-367`:

```rust
pub fn check_dotmeta() -> CheckResult {
    CheckResult {
        id: "dotmeta".into(),
        name: "Package Metadata".into(),
        status: Status::Pass,
        message: ".dotmeta files intentionally removed (stow conflict resolution)".into(),
        fix: None,
    }
}
```

No filesystem read. No branch. **It cannot fail.** And the claim is false right now: `docs/.dotmeta`
exists on disk, containing `name: docs / description: System documentation and guides / category:
documentation / stowable: false` -- stow metadata orphaned when INT-107 decommissioned the stow
subsystem. It is wired live at `mod.rs:1306`, so it feeds the score.

⚠️ THE DAMNING ADJACENCY: the function immediately below it, `check_intents()` at `checks.rs:370`,
opens with *"INT-135 Gate 7: was decoration -- hardcoded Status::Pass, a phantom active/ folder, no
in-progress, and a substring match for status: complete over whole files. Now calls the ONE
validator."* **Same file. Adjacent function. Identical defect. INT-135 found one and walked past its
neighbour.** That is the whole argument for a structural fix rather than another manual pass.

### 2. Every check counts the same

The boot chain and "docs generated" carry equal weight. A reader seeing 97% cannot tell whether the
missing 3% is a stale doc or a failing boot check. `check_dotmeta` was never the real problem with
the score -- **equal weighting is.**

### 3. Nothing structurally prevents either

There is no place in the code where a check declares what it measures, how much it matters, or what
it can produce. Both defects above are invisible by construction. That is why this is `arch` and not
`fix`.

## Evidence

All measured 2026-08-17 by live read. Line numbers are from that read.

### Inventory

- `checks.rs` defines 32 `pub fn check_`.
- `mod.rs:1298-1331` wires **34 checks** -- the 32 plus `check_deadwood` (local to `mod.rs:1246`)
  and `check_schema_validation` (from the `schema` module, imported at `mod.rs:39`).
- **Nothing is defined-but-unwired.** The code is consistent; the DOCS are the stale part.
  `WORKFLOWS.md` says 22 checks. `POLICIES.md` says 14. The code says 34. Only the code was measured.
- A second, shorter list exists at `mod.rs:1121-1126`: stow, services, broken_symlinks, git, scripts,
  disk_space. **Six checks. This is already a quick scan and nothing names it.**

### What is NOT wrong -- scoped honestly

- **`check_stow` is not a defect.** It is a hardcoded Pass, but its statement -- "Managed by
  home-manager (NixOS)" -- is **true**, and INT-107's evidence line records the reframing
  deliberately (*"reframed check_stow -> Dotfile Symlinks: Managed by home-manager"*). It is a
  **label**, not a lie.
- **`check_deadwood` is a real check.** It shells to `faelight-deadwood --summary`, splits on `|`,
  and warns only on high-confidence structural orphans (registry + modules) rather than the raw
  total. Its fix text -- *"reports only -- you decide every cut"* -- is the manual-control principle
  in code.

### Two heuristics were tried and both failed

Recorded so nobody re-derives them:

1. **Indentation depth of `status: Status::Pass`.** Eight spaces was taken to mean an unbranched
   return. It flagged `check_stow` (a legitimate label) and three checks that turned out to have
   real logic above the return.
2. **Underscore-prefixed parameters.** Scored one true positive (`check_dotmeta`, which takes no
   parameters at all) against two false alarms (`check_stow`, `check_deadwood`).

⚠️ **Neither is reliable. Bodies must be read.** A tell that is wrong two times in three is not a
tell -- which is itself the argument for declaration over detection.

### Not investigated

`check_sandbox` (1047), `check_vm_state` (1210), `check_compositor` (1248) each end in an unbranched
`Status::Pass` after real logic. **Suspects, not findings.** They are unresolved on purpose; the
first success criterion settles them.

## The class

Third sighting of one disease:

| Intent | The gate that was doing nothing |
| --- | --- |
| INT-113 | faelight-hooks -- never wired into `.git/hooks` |
| INT-119 | rustfmt "sandboxed, reproducible, unskippable" -- the hook file did not exist |
| this | `check_dotmeta` -- hardcoded Pass asserting a false fact |

`docs/CONVENTIONS.md` already names the tell: *a gate you have only watched pass might be doing
nothing.* What is missing is a way to catch it without reading 1700 lines by hand.

The distinction this intent establishes, and it is the durable output:

- A hardcoded Pass stating a **true** fact is a **label**. Legitimate, and some checks should be one.
- A hardcoded Pass stating a **false** fact is a **lie**. A bug.
- **A check that cannot fail must DECLARE itself as such.** Not be discovered.

## The Solution

### Engine and definitions

The runner is code. The checks are data. Updating the check set does not rebuild the engine.

This is the `fsh-test` shape -- declarative cases, one runner -- and it makes the defect
structurally impossible: **a definition must declare an assertion, so a definition without one is
invalid rather than a silent pass.**

### The escape hatch, enumerated

Some checks need real Rust: journal parsing, nix store queries, sqlite, subprocess. A definition
needing code declares a **registered probe** (`probe: <id>`) drawn from a fixed, reviewable registry.

⚠️ This is NOT a general run-arbitrary-code door. That door is how this becomes INT-193's disease --
two owners of the same job, drifting apart. One registry, one runner, and adding a probe is a
deliberate act someone can review.

### Declared severity ranges

**Each definition declares which severities it can produce.**

- Dotfile symlinks: green or red. Binary -- either home-manager owns them or it does not.
- Git repository: green or yellow. A dirty tree is never an emergency.
- Generation count: green, yellow, red -- red only past a threshold.

⚠️ **THIS IS THE KEY STRUCTURAL IDEA OF THE INTENT.** A definition declaring "pass only" **is a
label, by construction**. Labels announce themselves in the format instead of being found by hand,
and `check_dotmeta` could not have hidden. It also means the label/lie distinction above becomes
mechanical rather than editorial.

### Four states, three colours

PASS green, WARNING yellow, ERROR red -- the vocabulary the forest already uses; git status already
shows dirty in yellow, so nothing new is being taught.

The model carries a fourth state internally: **UNKNOWN, for a check that could not run**, rendered
yellow with wording that says it could not run rather than that something is wrong.

⚠️ Without this the tool cannot distinguish "checked and bad" from "could not check" -- exactly the
silence INT-192 was filed against. INT-148 claims `Status::Unknown` already exists and is excluded
from health math. **That claim is unverified and is a success criterion below.**

### Severity and tier are different axes

Severity is the check's **output**. Tier is the check's **importance**, and that vocabulary already
exists and is trusted: `RISK.toml`, critical / system / user, per directory, with a promotion rule
already written down (*"promote to critical if a profile ever carries boot, login, or disk
settings"*).

Do not invent a second scale. Map checks onto the tiers already in use.

### Scoring

- **Any critical-tier ERROR caps the reported health.** It cannot report healthy regardless of what
  else passes. One critical threat means at risk, no matter how many files were clean.
- system-tier and user-tier reduce the score proportionally to tier.
- **Labels are excluded from the denominator** and reported separately as declared, not measured.
- **The output states its basis** -- "31 measured, 3 declared, 1 critical failing" -- not a bare
  percentage. Anti-virus never reports 97% healthy; it reports items scanned, threats found, action
  taken.

### Quick scan and full scan

Both lists already exist and neither is named. Quick scan at session start (INT-124 freshness);
full scan on demand and after `dep`.

★ This is also what makes the definition model affordable: 34 subprocess-backed checks at every
session start would not be, and a slow doctor is a doctor people stop running.

### Red explains itself

A red check carries INT-199's shape: result first, then reason, what was compared, likely cause,
recovery. `CheckResult` already has a `fix:` field to carry it. `fpatch`'s `_refuse` is the reference
implementation.

### Thresholds are derived, never typed

Generation-count red derives from the physical limit -- `/boot` is 4G and lanzaboote's
`configurationLimit` is 15 -- not from a number typed once and forgotten.

⚠️ A typed threshold goes stale exactly the way the check COUNT did: 22 in one doc, 14 in another,
34 in the code.

## Success Criteria

### Phase 0 -- establish the truth

- [ ] `check_sandbox`, `check_vm_state` and `check_compositor` are read and classified
      real / label / lie, with the deciding lines quoted as evidence.
- [ ] All 34 wired checks are classified real / label / lie in a table in this intent, one line
      each. **No check is left unclassified**, including the ones that look obvious.
- [ ] INT-148's `Status::Unknown` claim is verified against the code, with evidence either way.
      ⚠️ If false, that is a third completed intent claiming something untrue and it is recorded
      here rather than quietly worked around.
- [ ] Every check is assigned a RISK.toml tier (critical / system / user), and the assignment is
      justified in one line each. Disagreement is expected and is the point.

### Phase 1 -- decide the format before writing engine code

- [ ] The definition format is DECIDED and written into this intent: fields for id, name, tier,
      declared severity range, assertion or probe, threshold source, and recovery text.
- [ ] The probe registry is enumerated and closed. Adding a probe is a deliberate, reviewable act;
      **there is no path from a definition to arbitrary code.**
- [ ] Scoring is DECIDED and written: severity x tier, critical caps, labels excluded from the
      denominator. ★ A decision to keep flat scoring is a valid discharge of this criterion --
      declining with reasons is still proof.
- [ ] The four-state / three-colour rendering is DECIDED, including the exact wording an UNKNOWN
      check shows so it cannot be misread as a failure.
- [ ] Quick scan and full scan are named, and it is stated which runs when and what each contains.

### Phase 2 -- build, proving each gate by watching it fail

- [ ] `check_dotmeta` is corrected or removed, and `docs/.dotmeta` is dealt with in the same change
      so the repo and the check agree. ⚠️ Fixing only one of the two leaves the contradiction.
- [ ] A definition declaring pass-only is treated as a label and excluded from the denominator.
      **Proven by watching it work: fabricate a pass-only definition, watch it be excluded and
      reported as declared, then remove it and watch the denominator return.**
- [ ] A definition with no assertion and no probe is REJECTED. **Proven by watching it fail:** write
      one, watch it be refused, then complete it and watch it accepted.
- [ ] A critical-tier ERROR caps the reported health. **Proven by watching it fail:** force a
      critical check red and confirm the verdict cannot read healthy.
- [ ] An UNKNOWN check is excluded from health math and rendered as could-not-run. **Proven by
      watching it fail:** make a probe unavailable and confirm the output says so rather than
      reporting clean.
- [ ] `faelight-deadwood` gains a mechanical check for a check that cannot fail. **Proven by
      watching it fail first:** reintroduce a hardcoded-Pass definition, watch it be flagged, remove
      it, watch the flag clear. ⚠️ Without the fail-first proof this gate is itself decoration --
      a fourth sighting of the disease, inside the fix for it.

### Phase 3 -- output and hygiene

- [ ] The health output states its basis rather than a bare percentage.
- [ ] Every red check renders INT-199 shape: result first, reason, comparison, likely cause,
      recovery.
- [ ] Generation-count thresholds derive from ESP size and `configurationLimit` rather than typed
      constants.
- [ ] The stale check counts are corrected wherever they appear, and the number is DERIVED rather
      than typed so it cannot go stale a fourth time.
- [ ] `rebuild-safe` is reviewed against the new scoring and it is stated -- with a reason --
      whether it gates on the percentage or on critical-tier status.

## Prior art -- do not duplicate

All three doctor intents are in `complete/`. This extends them; it does not reopen them.

- **INT-050** doctor-v2
- **INT-124** health freshness -- refresh doctor event on session start if stale
- **INT-148** doctor first-class `Status::Unknown`, excluded from health math (claim unverified)
- **INT-192** forest tools cannot express an undetermined outcome, so failed checks report clean
- **INT-199** better error messaging -- the shape a red check must follow
- **INT-135** repair intent tooling -- fixed the identical defect in `check_intents`
- **INT-073** generation count control, prune policy, boot menu cap
- **INT-107** decommissioned the stow subsystem, which is what orphaned `docs/.dotmeta`
- **INT-211** owns the `type:` field inconsistency. ⚠️ Do not fix that here.
- INT-011, INT-151 -- individual doctor checks

## Non-goals

- Rewriting the doctor. 34 checks work and most of them do real work.
- Adding new checks. Nothing here asks for more coverage; that is a separate conversation.
- Changing `rebuild-safe` behaviour before the scoring criterion is discharged.
- Fixing the `type:` field inconsistency, or the missing `arch-era/` archive. Both are real, both
  are elsewhere.

## Risk

`system`. Nothing here is lockout-class. The failure mode of getting it wrong is a health score that
is wrong in a new way, which is the situation today, so the floor is low.

⚠️ Promote to `critical` if this intent ends up changing what `rebuild-safe` will and will not
proceed through. That is a safety gate, and changing a safety gate deserves the higher tier.

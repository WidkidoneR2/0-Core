---
id: 222
date: 2026-08-17
type: arch
title: "the doctor reports health it does not measure -- one check asserts a false fact and cannot fail, every check counts the same, and nothing structurally prevents either"
status: in-progress
tags: [architecture, doctor, health, rust, design]
---

## Vision

The doctor becomes a scan engine with declared definitions, the way anti-virus software works:
one runner, a set of checks expressed as data, a quick scan and a full scan, per-item findings,
and a single verdict that states its basis. A check that cannot fail becomes impossible to write
rather than difficult to find.

⚠️ AND MOST OF IT ALREADY WORKS. 34 checks are wired and running, and every one sampled apart from
`check_dotmeta` does real work. This is consolidation and instrumentation, not repair.

## The Problem

`d` reports a health percentage built from 34 checks. Two things are wrong with that number, and
the second is larger than the first.

**One check asserts a fact it never verifies.** `checks.rs:359-367`:

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

No filesystem read. No branch. It cannot fail. And the claim is false right now: `docs/.dotmeta`
exists on disk, containing stow metadata (`stowable: false`), orphaned when INT-107 decommissioned
the stow subsystem. It is wired live at `mod.rs:1306`, so it feeds the score.

**Every check counts the same.** The boot chain and "docs generated" carry equal weight. So the
percentage does not mean what a reader thinks it means. `check_dotmeta` was never the real problem
with the score. Equal weighting is.

⚠️ THE DAMNING ADJACENCY: the function immediately below it, `check_intents()` at `checks.rs:370`,
opens with *"INT-135 Gate 7: was decoration -- hardcoded Status::Pass, a phantom active/ folder ...
Now calls the ONE validator."* Same file, adjacent function, identical defect. INT-135 found one and
walked past its neighbour. That is the argument for a structural fix rather than another manual pass.

## Evidence (measured 2026-08-17, live reads)

- `checks.rs` defines 32 `pub fn check_`; `mod.rs:1298-1331` wires **34 checks** (plus
  `check_deadwood` local to mod.rs and `check_schema_validation` from the schema module).
- **Nothing is defined-but-unwired.** The docs are the stale part: `WORKFLOWS.md` says 22 checks,
  `POLICIES.md` says 14, the code says 34. Only the code was measured.
- A second, shorter list exists at `mod.rs:1121-1126` -- six checks (stow, services,
  broken_symlinks, git, scripts, disk_space). **This is already a quick scan. Nothing names it.**

### What is NOT wrong -- scoped honestly

- **`check_stow` is not a defect.** It is a hardcoded Pass, but its statement -- "Managed by
  home-manager (NixOS)" -- is true, and INT-107 recorded the reframing deliberately. It is a
  **label**, not a lie.
- **`check_deadwood` is a real check.** It shells to `faelight-deadwood --summary` and warns only on
  high-confidence structural orphans (registry + modules), not the raw total.

### Two heuristics were tried and both failed

Indentation depth of `Status::Pass`, and underscore-prefixed parameters. The underscore tell scored
one true positive against two false alarms. **Neither is reliable. Bodies must be read.** This is
recorded so the next person does not re-derive them.

### Not investigated

`check_sandbox` (1047), `check_vm_state` (1210), `check_compositor` (1248) each end in an unbranched
`Status::Pass` after real logic. **Suspects, not findings.** G1 settles them.

## The class

Third sighting of one disease: INT-113 (faelight-hooks never wired into `.git/hooks`), INT-119
(rustfmt "unskippable" while the hook file did not exist), and this. `docs/CONVENTIONS.md` already
names the tell -- *a gate you have only watched pass might be doing nothing.* What is missing is a
way to catch it without reading 1700 lines.

The distinction this intent establishes:

- A hardcoded Pass stating a **true** fact is a **label**. Legitimate.
- A hardcoded Pass stating a **false** fact is a **lie**. A bug.
- **A check that cannot fail must declare itself as such.** Not be discovered.

## The design

### Engine and definitions

The runner is code. The checks are data. Updating the check set does not rebuild the engine. This is
the `fsh-test` shape -- declarative cases, one runner -- and it makes the defect above structurally
impossible: a definition must declare an assertion, so a definition without one is invalid rather
than a silent pass.

### The escape hatch, enumerated

Some checks need real Rust: journal parsing, nix store queries, sqlite. A definition needing code
declares a **registered probe** (`probe: <id>`) from a fixed registry. Not a general run-arbitrary-
code door. One registry, one runner, no second system.

⚠️ This is the gate that keeps this from becoming INT-193's disease -- two owners of the same job.

### Declared severity ranges

**Each definition declares which severities it can produce.** Dotfile symlinks: green or red.
Git repository: green or yellow. Generation count: green, yellow, red.

⚠️ THIS IS THE KEY STRUCTURAL IDEA. A definition declaring "pass only" **is a label, by
construction**. Labels announce themselves in the format instead of being found by hand.

### Four states, three colours

PASS green, WARNING yellow, ERROR red -- the same vocabulary the forest already uses (git status
already shows dirty in yellow). But the model carries a fourth state internally: **UNKNOWN, for a
check that could not run**, rendered yellow with wording that says it could not run rather than that
something is wrong.

⚠️ Without this the tool cannot distinguish "checked and bad" from "could not check" -- exactly the
silence INT-192 was filed against. INT-148 claims `Status::Unknown` already exists and is excluded
from health math. **That claim is unverified (G4).**

### Severity and tier are different axes

Severity is the check's **output**. Tier is the check's **importance**, and the vocabulary already
exists and is trusted: `RISK.toml` critical / system / user, per directory, with a promotion rule
already written (*"promote to critical if a profile ever carries boot, login, or disk settings"*).

Scoring rules proposed:

- **Any critical-tier ERROR caps the reported health.** It cannot report healthy regardless of what
  else passes. One critical threat means at risk, no matter how many files were clean.
- system-tier and user-tier reduce the score proportionally to tier.
- **Labels are excluded from the denominator** and reported separately as declared, not measured.
- **The output states its basis** -- "31 measured, 3 declared, 1 critical failing" -- not a bare
  percentage. Anti-virus never reports 97% healthy; it reports items scanned, threats found, action
  taken.

### Quick scan and full scan

Both lists already exist and neither is named. Quick scan at session start (INT-124 freshness);
full scan on demand and after `dep`. This is also what makes the definition model affordable -- 34
subprocesses at every session start would not be.

### Red explains itself

A red check carries INT-199's shape: result first, reason, what was compared, likely cause, recovery.
`CheckResult` already has a `fix:` field to carry it. `fpatch`'s `_refuse` is the reference.

### Thresholds are derived, never typed

Generation count red derives from the physical limit -- `/boot` is 4G, lanzaboote's
`configurationLimit` is 15 -- not from a number typed once. ⚠️ A typed threshold goes stale exactly
the way the check COUNT did.

## Gates

- [ ] **G1** -- `check_sandbox`, `check_vm_state`, `check_compositor` read and classified
      real / label / lie, bodies quoted as evidence.
- [ ] **G2** -- all 34 checks classified real / label / lie. A table in this intent, one line each.
      No check left unclassified.
- [ ] **G3** -- INT-148's `Status::Unknown` claim verified against the code. Evidence either way.
      If false, that is a third completed intent claiming something untrue and it is recorded here.
- [ ] **G4** -- `check_dotmeta` corrected or removed, and `docs/.dotmeta` dealt with in the same
      change so the repo and the check agree.
- [ ] **G5** -- the definition format is DECIDED and written here before any engine code: fields for
      id, name, tier, declared severity range, probe or assertion, threshold source, recovery text.
- [ ] **G6** -- the probe registry is enumerated and closed. Adding a probe is a deliberate,
      reviewable act; there is no path to arbitrary code.
- [ ] **G7** -- a definition declaring pass-only is treated as a label and excluded from the
      denominator. **Proven by watching it work: fabricate a pass-only definition, watch it be
      excluded and reported as declared, then remove it and watch the denominator return.**
- [ ] **G8** -- scoring DECIDED and written: severity x tier, critical caps, labels excluded.
      A decision to keep flat scoring is a valid discharge of this gate.
- [ ] **G9** -- quick scan and full scan are named, documented, and it is stated which runs when.
- [ ] **G10** -- the health output states its basis rather than a bare percentage.
- [ ] **G11** -- generation-count thresholds derive from ESP size and `configurationLimit`, not from
      typed constants.
- [ ] **G12** -- the stale check counts are corrected wherever they appear, and the number is
      derived rather than typed so it cannot go stale again.

## Prior art -- do not duplicate

All three are in `complete/`. This intent extends them; it does not reopen them.

- **INT-050** doctor-v2
- **INT-124** health freshness -- refresh on session start if stale
- **INT-148** doctor first-class `Status::Unknown`, excluded from health math (claim unverified, G3)
- **INT-192** forest tools cannot express an undetermined outcome, so failed checks report clean
- **INT-199** better error messaging -- the shape a red check must follow
- **INT-073** generation count control, prune policy, boot menu cap
- **INT-211** the intent ledger has no canonical document shape -- owns the `type:` field mess.
  ⚠️ Do not fix that here.
- INT-011, INT-151 -- individual doctor checks

## Non-goals

- Rewriting the doctor. 34 checks work.
- Adding new checks. Nothing here asks for more coverage.
- Touching `rebuild-safe` behaviour before G8 is decided.
- Fixing the `type:` field inconsistency (INT-211's).

## Risk

`system`. Nothing here is lockout-class. The failure mode of getting it wrong is a health score that
is wrong in a new way, which is the situation today, so the floor is low.

⚠️ Promote to `critical` if this intent ends up changing what `rebuild-safe` will and will not
proceed through. That is a safety gate, and changing it deserves the higher tier.

# INT-XXX — The doctor reports health it does not measure

**Status:** proposed
**Risk:** system — `rebuild-safe` gates on the health score, so this is a safety mechanism, not a report
**Depends on:** nothing. Blocks nothing. Can be worked in isolation.

---

## Problem

`d` reports a health percentage built from 34 checks. Two things are wrong with that number,
and the second is bigger than the first.

**1. At least one check asserts a fact it never verifies.**
**2. Every check counts the same.** The boot chain and "docs generated" carry equal weight,
so the percentage does not mean what a reader thinks it means.

The first is a bug. The second is a design question, and it is the one worth the session.

---

## Evidence (measured 2026-08-17, all line numbers from a live read)

### The proven defect

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

No filesystem read. No branch. It cannot fail. And the claim is **false right now** —
`docs/.dotmeta` exists on disk, containing stow metadata (`stowable: false`), orphaned when
INT-107 decommissioned the stow subsystem.

It is wired live at `doctor/mod.rs:1306`, so it is one of the 34 checks feeding the score.

### The damning adjacency

The function immediately below it, `check_intents()` at `checks.rs:370`, opens with:

> INT-135 Gate 7: was decoration -- hardcoded Status::Pass, a phantom "active/" folder, no
> "in-progress", and a substring match for "status: complete" over whole files. Now calls the
> ONE validator.

**Same file. Adjacent function. Identical defect.** INT-135 found one and walked past its
neighbour. That is the argument for a mechanical check rather than another manual pass.

### What is NOT wrong — scoped honestly

- **34 checks are wired and running** (`mod.rs:1298-1331`). Nothing is defined-but-unwired.
- **The docs are the stale part.** `WORKFLOWS.md` says 22 checks, `POLICIES.md` says 14, the
  code says 34. Only the code was measured.
- **`check_stow` is not a defect.** It is a hardcoded Pass, but its statement — "Managed by
  home-manager (NixOS)" — is true, and INT-107 recorded the reframing deliberately. It is a
  **label**, not a lie.
- **`check_deadwood` is a real check.** It shells to `faelight-deadwood --summary` and warns
  only on high-confidence structural orphans.
- Two heuristics were tried and **both failed**: indentation depth of `Status::Pass`, and
  underscore-prefixed parameters. The underscore tell scored 1 true positive against 2 false
  alarms. Neither is reliable. Bodies must be read.

### Not investigated

`check_sandbox` (1047), `check_vm_state` (1210), `check_compositor` (1248) each end in an
unbranched `Status::Pass` after real logic. **Suspects, not findings.** Gate 1 settles them.

---

## The class

This is the third sighting of one disease:

| Intent | The gate that was doing nothing |
| --- | --- |
| INT-113 | faelight-hooks — never wired into `.git/hooks` |
| INT-119 | rustfmt "sandboxed, reproducible, unskippable" — the hook file did not exist |
| this | `check_dotmeta` — hardcoded Pass asserting a false fact |

`docs/CONVENTIONS.md` already names the tell: *a gate you have only watched pass might be doing
nothing.* What is missing is a way to catch it without a human reading 1700 lines.

**The useful distinction, and the one this intent should establish:**

- A hardcoded Pass stating a **true** fact is a **label**. Legitimate. Should not count as a check.
- A hardcoded Pass stating a **false** fact is a **lie**. A bug.
- A check that **cannot** fail should be declared as such, not discovered.

---

## Design question: weighted health

The open idea, and the reason this is a discussion intent rather than a fix.

**Do not invent a weight vocabulary.** `RISK.toml` already defines critical / system / user, per
directory, and that vocabulary is already understood across the repo. Map each check to a tier
instead of inventing numbers.

**Weighting alone is insufficient.** A weighted average can still hide a critical failure behind
thirty passes. A machine that will not boot is not 94% healthy.

Proposed shape, for argument:

- Each check declares a tier.
- **Any critical-tier failure caps the reported health** — it cannot report healthy, regardless
  of what else passes.
- system-tier and user-tier failures reduce the score proportionally to tier.
- **Labels are excluded from the denominator** and reported separately as "declared, not measured".
- The output states the basis: *"31 measured, 3 declared, 1 critical failing"* rather than a bare
  percentage.

Open questions worth arguing before any of this is built:

1. Is a single percentage the right output at all, or should health be a tier vector?
2. Who owns the tier assignment — the check, or a table beside it?
3. Should `rebuild-safe` gate on the percentage, or only on critical-tier status?
4. What does a check that errors (cannot determine) report? INT-192 says silence is the worse
   failure. Is UNDETERMINED a fourth status?

---

## Gates

- [ ] **G1 — The three suspects are read and classified.** `check_sandbox`, `check_vm_state`,
      `check_compositor` each labelled real / label / lie, with the body quoted as evidence.
- [ ] **G2 — Every one of the 34 checks is classified** real / label / lie. A table in this
      intent, one line per check. No check left unclassified.
- [ ] **G3 — `check_dotmeta` is corrected or removed**, and the decision is recorded. If removed,
      `docs/.dotmeta` is dealt with in the same change so the repo and the check agree.
- [ ] **G4 — A mechanical check exists that flags a check which cannot fail**, in
      `faelight-deadwood`. Proven by watching it FAIL first: reintroduce a hardcoded-Pass check,
      watch it be flagged, then remove it and watch the flag clear.
- [ ] **G5 — Labels are declared, not inferred.** Whatever mechanism is chosen, a legitimate
      label (`check_stow`) passes G4 without a warning, and does so because it is marked, not
      because the checker guessed.
- [ ] **G6 — The weighting design is DECIDED and written here** before any scoring code changes.
      All four open questions above answered, with reasons. A decision to keep flat scoring is a
      valid discharge of this gate.
- [ ] **G7 — The health output states its basis**, not a bare percentage.
- [ ] **G8 — The stale counts are corrected** wherever they appear, and the corrected number is
      derived rather than typed, so it cannot go stale again.

---

## Non-goals

- Rewriting the doctor. 34 checks work; this is about the ones that do not and the number they feed.
- Adding new checks. Nothing here asks for more coverage.
- Touching `rebuild-safe` behaviour before G6 is decided.

---

## Risk

`system`. Nothing here is lockout-class. The failure mode of getting it wrong is a health score
that is wrong in a new way, which is the situation today, so the floor is low.

Promote to `critical` only if this intent ends up changing what `rebuild-safe` will and will not
proceed through — that is a safety gate, and changing it deserves the higher tier.

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

- [x] `check_dotmeta` is corrected or removed, and `docs/.dotmeta` is dealt with in the same change
      so the repo and the check agree. ⚠️ Fixing only one of the two leaves the contradiction.
      <!-- 2026-08-27: check_dotmeta was deleted earlier as proven decoration. THE GATE WARNED
      ABOUT EXACTLY WHAT HAPPENED NEXT: the file outlived the check, so a grep for dotmeta in
      Rust returned nothing while docs/.dotmeta still sat there. It held GNU stow metadata
      (stowable: false) from the Arch era; ROADMAP.md:98 already called it orphaned, and
      DEC-044 records removing all .dotmeta files -- it survived that sweep.

      AND SO DID A SECOND ONE, found only because the first was checked for siblings:
      faelight/rust-tools/.dotmeta, a museum of wrong facts -- version 10.3.0, tool_count
      42, last_updated February, tools compile to ~/0-core/scripts/ (deleted in e733287d),
      registry at 01-registry/tools.toml (moved long ago). Nothing read it, so nothing
      corrected it. Both files removed; the check and the repo now agree. -->
- [ ] A definition declaring pass-only is treated as a label and excluded from the denominator.
      **Proven by watching it work: fabricate a pass-only definition, watch it be excluded and
      reported as declared, then remove it and watch the denominator return.**
- [ ] A definition with no assertion and no probe is REJECTED. **Proven by watching it fail:** write
      one, watch it be refused, then complete it and watch it accepted.
- [ ] A critical-tier ERROR caps the reported health. **Proven by watching it fail:** force a
      critical check red and confirm the verdict cannot read healthy.
- [x] An UNKNOWN check is excluded from health math and rendered as could-not-run. **Proven by
      watching it fail:** make a probe unavailable and confirm the output says so rather than
      reporting clean.
      <!-- 2026-08-27 DEMONSTRATED, not observed. Ran the real doctor with PATH set to an empty
      directory, so no external probe resolved at all:
        PATH=/tmp/nobin ~/.local/bin/core doctor run
      Five checks went UNKNOWN and NAMED THEMSELVES -- System Services, Rust Docs, Reboot
      Needed, Update Readiness, Orphan Packages -- rather than reporting clean. Health fell to
      47% and the unknowns were excluded from the denominator. The same command on the real
      machine before and after reads 84% with one unknown, so nothing on disk changed.
      ONE CHECK PASSED WITH NO BINARIES AND WAS SUSPECTED OF LYING: Network reported online
      with DNS resolving. It is honest -- check_network uses Rust TCP and ToSocketAddrs, no
      external command, so an empty PATH cannot blind it. Suspicion was checked against the
      code rather than assumed either way.
      The live unknown on this machine is System Services: faelight-session.target does not
      exist on Omarchy, and the check says could not read rather than calling 0/0 healthy --
      which its own comment names as the free pass this intent exists to remove. -->
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

## Phase 0 -- the census (measured 2026-08-18)

**34 checks, matching the panel count.** Across three files: `checks.rs` (32), `mod.rs`
(`check_deadwood`), `schema.rs` (`check_schema_validation`).

⚠️ The file header of `checks.rs` says *"all 23 health check functions"*. There are 32 in it. That
is the **third** hand-maintained count with a different answer -- after WORKFLOWS.md saying 22 and
POLICIES.md saying 14. Nothing derives this number from the registry.

### Reproduce the census

```
python3 - << PYEOF
import re, pathlib
root = pathlib.Path("faelight/engine/src/domains/doctor")
rows = []
for f in sorted(root.rglob("*.rs")):
    parts = re.split(r"\n\s*(?:pub )?fn (check_[a-z_0-9]+)", f.read_text())
    for i in range(1, len(parts), 2):
        name, body = parts[i], parts[i+1]
        if "CheckResult" not in body[:200]:
            continue
        rows.append((name, f.name, sorted(set(re.findall(r"Status::([A-Za-z]+)", body))), body.count("\n")))
print("checks found:", len(rows))
for name, fname, variants, lines in sorted(rows):
    mark = "  <-- CANNOT FAIL" if variants == ["Pass"] else ""
    print("%-26s %-12s %-30s %4d%s" % (name, fname, ",".join(variants), lines, mark))
PYEOF
```

### The four categories

Phase 0 set out expecting three. The census produced four.

| Category | Definition | Health score? |
| --- | --- | --- |
| **real** | measures something and can report poor health | ✅ counts |
| **label** | states a true fact that needs no test and can never change | ❌ excluded |
| **lie** | asserts something it never measured, and the assertion is false | ⚠️ defect |
| **reporter** | measures truly, prints it, never renders a judgement | ❌ excluded |

★ **Labels and reporters both inflate the denominator.** If a check cannot indicate poor health, it
must not be one of the things health is computed from. That is the same argument INT-148 already
won for `Status::Unknown`.

### The three that cannot fail

Only three of the 34 emit `Status::Pass` and nothing else.

**`check_stow`** (9 lines) -- **TOMBSTONE. Delete.**
Its only job is to say a decommissioned subsystem does not exist. INT-107 retired stow; the check
was reframed to *"Managed by home-manager (NixOS)"*. It states something structural and permanent,
it can never change, and it is already implied by running NixOS at all.
⚠️ An earlier reading called this a legitimate label. That was too generous -- a tombstone is not a
health signal.

**`check_dotmeta`** (8 lines) -- **LIE. The defect this intent was filed for.**
Hardcoded `Status::Pass`, no filesystem read, and the claim is false: `docs/.dotmeta` exists,
carrying `stowable: false` from the retired stow subsystem. It asserts a fact it never checked.
📍 The function immediately below it, `check_intents`, opens with *"INT-135 Gate 7: was decoration
-- hardcoded Status::Pass"*. **INT-135 found one and walked past its neighbour.**

**`check_compositor`** (29 lines) -- **REPORTER. Legitimate, but exclude from the score.**
NOT hardcoded: it pgreps for mango and pinnacle and names which is running. "No compositor detected
(TTY or headless)" is `Pass` deliberately, with the reason written in -- *none is not a fault, d can
run from a TTY, so report it as info rather than crying wolf*. `check_vm_state` is the same shape.

### ⚠️ The structural finding

**Only SEVEN of 34 checks can ever report `Fail`:** `alias_coverage`, `binaries`,
`broken_symlinks`, `rust_toolchain`, `sandbox`, `security_audit`, `security_hardening`.

**The other 27 top out at `Warn`.**

⭐ So the panel's `❌ Failed: 0` is very nearly guaranteed **by construction, not by health.**
The doctor is a warning system that presents itself as a pass/fail system. That is this intent's
thesis, at a scale nobody had measured.

### A fifth problem: non-determinism

`Rust Docs` reported `✅ cargo doc clean, 0 warnings` and then `❔ unknown` on the very next run,
same machine, nothing changed between them.

⚠️ **A check that changes its mind while the system stands still is a different defect from one
that cannot fail**, and neither the taxonomy above nor the health score has anywhere to put it.

📍 `Status::Unknown` appears in exactly two checks -- `check_rust_docs` and `check_services`. The
INT-148 mechanism is real (verified 2026-08-17: 29/(34-1) = 87%, so unknowns are excluded from the
denominator) but barely used.

### What Phase 1 must now decide

- Whether `check_stow` is deleted, and whether deleting a check is a normal act or a rare one.
- Where the excluded categories go: dropped, or shown outside the score in their own section.
- Whether `Warn` and `Fail` mean different things to the score, given 27 checks can only warn.
- What to do about non-determinism -- retry, cache, or classify as `Unknown` by design.
- Whether the check count is derived from the registry rather than written in three places.

### ⚠️⚠️ A SIXTH PROBLEM, found 2026-08-23: THERE ARE TWO DOCTORS

Everything above censuses `core doctor` -- 34 checks in the engine. **`fsh doctor` is a SECOND
doctor, seven checks, in a different binary, and it has never been censused.** Found while fixing
INT-227's hardcoded paths, not by looking for it.

Three of its seven were wrong, and each is a different failure:

    ✗  fsh binary       tested a HARDCODED path inside one user's checkout
                        (/home/christian/0-core/scripts/faelight-shell). That file does not
                        exist on this machine -- verified with ls -- so the check reported
                        `missing!` EVERY TIME IT RAN, since the day it was written.
                        A check that has never passed is this intent's thesis inverted:
                        not a check that cannot fail, but one that cannot succeed.
                        FIXED under INT-227: it now asks current_exe() and reports the path.

    ✗  focus intent     hand-builds $HOME/.local/state/0-core/intent/focus.toml as a string,
                        while the check TWO LINES ABOVE it uses paths::state_db() correctly.
                        Same file, two conventions. Reports "no focus.toml" while `intl`
                        shows INT-222 active and the bar reads the focus fine.
                        NOT FIXED -- INT-115 owns routing paths through paths.rs.

    ✗  cargo in PATH    reports "missing -- run: source ~/.profile" while cargo demonstrably
                        works; the entire session that found this was built with it.
                        ⚠️ AND THE ADVICE IS A BASHISM. `source ~/.profile` is not how fsh
                        restores a path, and on another distro it may not exist at all.
                        NOT FIXED.

★ THE POINT FOR THIS INTENT: the thesis is not specific to `core doctor`. A seven-check doctor
had one check that could never succeed, one false negative carrying advice for a different
shell, and one hand-built path its own neighbour looks up properly. **Three of seven.** The
34-check census found a 27-of-34 structural defect; a second, much smaller doctor was found to
be wrong at a similar rate the first time anyone read it.

⏭ SO PHASE 1 GAINS A QUESTION: does the definition format, the probe registry and the scoring
serve BOTH doctors, or does `fsh doctor` remain a separate thing that will drift the same way?
A shared engine is the obvious answer and may be the wrong one -- `fsh doctor` checks the SHELL
(its binary, its database, its aliases) while `core doctor` checks the SYSTEM, and they may
genuinely want different tiers. **The decision belongs here rather than being made by accident.**

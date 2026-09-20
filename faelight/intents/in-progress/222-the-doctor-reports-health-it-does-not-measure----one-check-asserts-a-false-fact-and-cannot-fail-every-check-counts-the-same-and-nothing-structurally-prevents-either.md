---
id: 222
date: 2026-08-17
type: arch
title: "the doctor reports health it does not measure -- one check asserts a false fact and cannot fail, every check counts the same, and nothing structurally prevents either"
status: in-progress
tags: [architecture, rust, design]
---

> ⚠️ **COUNTS IN THIS DOCUMENT ARE DATED.** Everything below the RESCOPED marks was measured
> 2026-08-17, when the doctor wired **34** checks. It wires **27** today; the difference went
> with the Omarchy migration. Historical numbers are left as written rather than edited,
> because they record what was true when the work was done. The live count is derived:
> `all_checks().len()`, printed in the doctor header.

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

## The switch, planned 2026-09-19

★ ONE COMMIT, AND THE MEASUREMENT DECIDED IT RATHER THAN A PREFERENCE.

The question was whether to switch and delete together, or switch first and delete after. Easier
is not the same as right, so it was measured: DO THE 28 CHECK FUNCTIONS HAVE ANY CALLER BESIDES
all_checks()? They do not. One caller, so there is no window in which two producers coexist and
nothing separate to isolate. One commit.

```text
    checks.rs       26 functions
    mod.rs:1342     check_deadwood
    schema.rs:5     check_schema_validation
    ------------------------------------------
    bins.rs:157     check_binaries(quiet)   NOT ONE OF THE 28
```

⚠️ DELETE BY LOCATION, NEVER BY NAME. `bins.rs:157` is a DIFFERENT `check_binaries`, serving
`core doctor bins`, and it stays. Two functions sharing a name is exactly what made the first
census miscount, and deleting the wrong one would take a live command with it.

The adapter is mechanical: `CheckResult` and `Outcome` are field-for-field identical (six fields,
same names, same order, `fix` vs `recovery` the only difference), and both `Status` and `Tier`
have the same variants in the same order. `all_checks()` is rewritten IN PLACE -- same name,
same signature -- so both call sites and everything downstream are untouched.

The enum mapping is EXHAUSTIVE with no catch-all, so a new variant fails to compile instead of
defaulting silently. Same rule as the probe dispatcher.

## Success Criteria

### Phase 0 -- establish the truth

- [x] `check_sandbox`, `check_vm_state` and `check_compositor` are read and classified
      real / label / lie, with the deciding lines quoted as evidence.
      <!-- DONE 2026-09-04, read line by line.

      check_sandbox -- REAL. Probes with `which faelight-sandbox`, which asks the question
      the shell answers, then reads sandbox-policies.toml through paths::registry_dir and
      counts `name =` lines. Three branches, all reachable: Fail when the binary is absent,
      Warn when the policies file is missing, Pass with the measured count otherwise. The 5
      in `5 policies active` is counted, not asserted.
      ⚠️ One INT-192 note: unwrap_or(0) on the read means a file that EXISTS but cannot be
      read reports 0 policies and still passes. Low severity -- the existence check above
      catches the common case -- but it is the same collapse and belongs in that census.

      check_vm_state -- REAL, and already correct on the axis INT-192 cares about. Runs
      `pgrep -f -c qemu-system` and parses the count, with an explicit Err arm returning
      Warn and the message `Could not check for running VMs`. It distinguishes no VMs from
      could not look, which most checks in this file do not. Tier::Info, so a running VM is
      reported without being a fault -- the comment says VM-first development means VMs are
      often up by design.

      check_compositor -- ABSENT. There is no such function. `Compositor` and `Compositor
      Keybinds` survive only in two display lists in cockpit.rs, and the note at
      cockpit.rs:149 already records them as checks deleted when Omarchy replaced their
      subjects.

      ⭐ WHICH ADDS A FOURTH CLASS THIS INTENT DID NOT HAVE: PHANTOM -- a name in the
      display with no check behind it. Not real, label or lie, because there is nothing to
      classify. The three classes above all assume a function exists.

      And it is the OPPOSITE direction from the drift the 09-02 catch-all fixed. That made
      an unclaimed CHECK visible; a claimed NAME with no check is still silent, because
      filter_map drops it without complaint. Same two-owners defect, other end. Known
      phantoms today: Dotfile Symlinks, Compositor Keybinds, Theme Packages, Package
      Metadata, and `Scripts` in git_names. -->
- [x] All 34 wired checks are classified real / label / lie in a table in this intent, one line
      each. **No check is left unclassified**, including the ones that look obvious.
      <!-- DONE 2026-09-04. Enumerated from all_checks(), not grepped, and read one by one.

      ⚠️ IT IS 27, NOT 34. all_checks lists twenty-seven and the doctor header says 27/27.
      The 34 in this intent is stale -- checks went with their subjects at the migration.
      Gate: the stale check counts are corrected wherever they appear.

      | check | class | evidence |
      | --- | --- | --- |
      | check_services | REAL | asks systemd what faelight-session.target Wants; does not name services |
      | check_broken_symlinks | REAL | walks ~/.config depth 6; runtime-link exclusion is a property, not a name list |
      | check_binaries | REAL | probes 12 binaries; list is hand-maintained and its comment says treat it as code |
      | check_git | REAL, FAILS OPEN | two git probes, both unwrap_or(false) -- unrunnable git reports a clean tree and PASSES |
      | check_hooks | REAL | core.hooksPath plus the executable bit, three failure modes, Unknown when git cannot be asked |
      | check_rust_docs | REAL | cargo doc bounded at 3s by thread + recv_timeout; a stuck cargo is Unknown, never a hang |
      | check_intents | REAL | calls intent::validate_issues, the one validator. Was a hardcoded Pass until INT-135 gate 7 |
      | check_deadwood | REAL | parses the summary line; a non-numeric field is Unknown since 2026-09-04 |
      | check_faelight_config | REAL, SWALLOWS | parses three TOMLs; a file that exists but cannot be READ counts as no issue |
      | check_security_hardening | REAL | asks five possible firewall units rather than one name -- a false alarm is its own failure |
      | check_security_audit | REAL | reads last-scan.json; Warn naming skipped sub-scans since 2026-09-04 |
      | check_alias_coverage | REAL | parses the live config; an unparseable file is Fail, not zero missing |
      | check_rust_toolchain | REAL, fails CLOSED | cargo and rustc probes; an unrunnable probe reports missing -- a false alarm, not a false pass |
      | check_disk_space | REAL, SWALLOWS | df -h parsed through a filter_map chain; an unparseable mount yields no warnings and passes |
      | check_tool_installation | REAL | returns early on a missing registry. Its comment records printing All 0 key tools installed on a VM with 32 |
      | check_path_resilience | REAL | same fix: a percentage of nothing is not zero percent. Printed 0/0 tools deployed (0%) |
      | check_schema_validation | REAL | schema_dir missing is Warn; validates each registry file against its schema |
      | check_sandbox | REAL, SWALLOWS | which probe plus a registry read; an unreadable policies file reports 0 policies and PASSES |
      | check_boot_errors | REAL | separates journal crit from err deliberately; benign hardware noise is the baseline and is not an alarm |
      | check_boot_time | REAL | systemd-analyze time; a non-success exit returns early rather than a fabricated number |
      | check_reboot_needed | REAL, exemplary | running_kernel failure is Status::Unknown WITH the reason. The pattern the others should copy |
      | check_update_readiness | REAL | separates blockers from unreadable, which is this axis built in from the start |
      | check_package_cache | REAL | read_dir on the pacman cache; Err returns early with the reason |
      | check_orphan_packages | REAL | pacman -Qdtq; a spawn failure is Unknown |
      | check_friday | REAL | opens state.db through paths::state_db; open failure returns early |
      | check_network | REAL | TCP connect to an IP with a 1s cap, then bounded DNS. Cannot hang |
      | check_vm_state | REAL | pgrep -f -c qemu-system; an explicit Err arm returns Warn rather than a false zero |
      | check_compositor | PHANTOM | NO SUCH FUNCTION. Compositor and Compositor Keybinds survive only in cockpit.rs display lists |

      ⭐ TWENTY-SIX REAL, ONE PHANTOM, ZERO LIES -- and that changes this intent premise.
      It was written 2026-08-17 expecting decoration. check_dotmeta, the hardcoded Pass
      asserting a false fact that the whole charter is built on, is already gone. So are
      the others: check_intents, check_tool_installation, check_path_resilience,
      check_services, check_hooks and check_security_hardening all carry fix comments
      citing this intent or its siblings. The doctor was repaired check by check while
      this intent sat open.

      ⚠️ WHAT IS ACTUALLY LEFT IS TWO THINGS, and neither is finding a lie:

      1. FOUR QUIET-DIRECTION COLLAPSES, listed above as FAILS OPEN or SWALLOWS. Each
         reports health it did not measure when its probe cannot run: check_git,
         check_faelight_config, check_disk_space, check_sandbox. All four are INT-192
         shape and all four pass while blind.

      2. THE STRUCTURAL PREVENTION this intent Vision asks for -- making an unfailable
         check impossible to WRITE rather than difficult to FIND. The census proves the
         manual pass works and also proves what it costs: twenty-seven functions read by
         hand, which is exactly what the charter says must not be the durable answer. -->
- [x] INT-148's `Status::Unknown` claim is verified against the code, with evidence either way.
      ⚠️ If false, that is a third completed intent claiming something untrue and it is recorded
      here rather than quietly worked around.
      <!-- VERIFIED 2026-09-04 with arithmetic, not assertion. The claim holds.

      mod.rs, and the comment states it: Unknown and Blocked are excluded from the ratio,
      couldn't determine and blocked are not failures and must not drag health down.

          let determinable = total - unknown - blocked;
          let health = if determinable > 0 { (passed * 100) / determinable } else { 0 };

      PROVEN BY A BLIND RUN -- every probe made unreachable with PATH=/nonexistent:
          27 total, 9 unknown, 18 determinable, 7 passed
          700 / 18 = 38 with integer division
          the doctor printed Health: 38%
      Excluding them would give 7/27 = 25%. The formula is doing what the comment says.

      The verdict function agrees separately: Critical + Unknown returns Red, any other
      Unknown sets amber. An unknown critical check is treated as a failure, which is the
      right direction -- not knowing whether the disk is full is not the same as it being
      fine.

      ⚠️ AND THIS IS WHERE THE NEXT DEFECT LIVES. (passed * 100) / determinable weights
      every check IDENTICALLY, which is complaint 2 in this intent's own Problem section:
      a failing Critical Disk Space and a Warn on Alias Coverage move the number by the
      same amount. Worse, Warn counts as not-passed, so today's four warnings cost exactly
      what four failures would. That is the scoring gate, not this one. -->
- [x] Every check is assigned a RISK.toml tier (critical / system / user), and the assignment is
      justified in one line each. Disagreement is expected and is the point.
<!-- DONE 2026-09-06. Two corrections to the gate's own premises first:

     (a) THE TIERS ARE ALREADY ASSIGNED. Every check sets `tier:` on every return arm today,
     and all 27 are INTERNALLY CONSISTENT -- no check assigns two different tiers in two
     branches. So this gate was never "assign them"; it is "justify them, and say where the
     assignment is wrong". The tier IS repeated per arm rather than declared once per check,
     which is the shape Phase 1's definition format should fix.

     (b) THERE ARE FOUR TIERS, NOT THREE. Tier::Info exists in mod.rs:29 and the doc comment
     above it explains why: the check measures something true but never renders a judgement,
     so it is excluded from the verdict entirely. The gate's wording (critical/system/user)
     predates it.

     THE RULING THAT DECIDES THE TABLE, stated by Christian 2026-09-06: on Omarchy the tier
     is about WHAT BREAKS THE SYSTEM, not what looks alarming. The only genuine lockout paths
     are the boot chain and the login shell. Everything else is degradation, and TTY2 plus
     bash is the way back from all of it.

     population: all_checks() in mod.rs:1410, 27 entries, read 2026-09-06.

     CRITICAL (3)
       check_binaries         12 binaries absent means the system is not usable. Earns it.
       check_boot_errors      the boot chain is one of the two real lockout paths.
       check_disk_space       ** DISAGREE -- see below. Left as found.

     SYSTEM (12)
       check_services         a dead unit degrades a subsystem; login survives it.
       check_broken_symlinks  breaks tools, not boot.
       check_faelight_config  a bad config costs the shell session, not the machine.
       check_security_hardening  firewall/sshd posture: serious, not a lockout.
       check_security_audit   advisory over dependencies; nothing stops working.
       check_rust_toolchain   no toolchain means no builds; the system still runs.
       check_tool_installation  the forest is degraded, the OS is not.
       check_path_resilience  deployment coverage; recoverable from any shell.
       check_sandbox          policy enforcement absent is a posture loss, not a halt.
       check_network          offline is severe and is not a lockout; local login works.
       check_reboot_needed    a pending kernel is a scheduling fact about the system.
       check_schema_validation (schema.rs) registry integrity; tools misbehave, boot does not.
       check_boot_time        ** DISAGREE -- see below. Left as found.

     USER (9)
       check_rust_docs        stale docs cost a reader nothing but time.
       check_git              a dirty tree is never an emergency.
       check_hooks            hooks failing costs process discipline, not the machine.
       check_intents          ledger validity is a workflow property.
       check_alias_coverage   missing aliases cost convenience.
       check_friday           the reasoning layer being quiet breaks no command.
       check_update_readiness advisory before an update.
       check_orphan_packages  housekeeping.
       check_deadwood         (mod.rs:1332) reports only -- you decide every cut.

     INFO (2)
       check_vm_state         a running VM is reported without being a fault; VM-first
                              development means VMs are often up by design.
       check_package_cache    a cache size is a number, not a judgement.

     ** TWO DISAGREEMENTS, RECORDED NOT CHANGED. Re-tiering moves the score, and the scoring
     criterion above is already closed; changing an input to a closed decision without
     re-proving it would be the thing this intent exists to stop.

     1. check_disk_space is CRITICAL and should not be. Critical caps reported health
        outright. A disk at 85% capping the system to at-risk is not the class of a boot
        failure -- it is a gradient, and the check already has three arms to express it.
        Nothing on Omarchy fails to boot at 85%.

     2. check_boot_time is SYSTEM and is closer to INFO. A slow boot broke nothing; it is a
        measurement, not a fault. check_vm_state already uses Info for exactly this reasoning
        one screen away. Note the asymmetry that makes this visible: boot_errors (a real
        lockout path) is Critical while boot_time (a stopwatch) is System -- two checks over
        the same subsystem, tiered by subject rather than by consequence.

     Both belong to the scoring work, not here. Fixing them means re-proving the critical-cap
     and denominator gates against the new inputs. -->

### Phase 1 -- decide the format before writing engine code

- [x] The definition format is DECIDED and written into this intent: fields for id, name, tier,
      declared severity range, assertion or probe, threshold source, and recovery text.
      See "THE SIXTH PROBLEM IS ANSWERED" below -- the format is decided together with WHERE it
      lives, because the two questions could not be separated.
- [x] The probe registry is enumerated and closed. Adding a probe is a deliberate, reviewable act;
      **there is no path from a definition to arbitrary code.**
      Enumerated by census, not by design -- see "THE PROBE REGISTRY, MEASURED" below. FIFTEEN
      probes. One of them, resolve_binary_path, is the arbitrary-code door and is flagged for
      rewrite in Phase 2.
- [x] Scoring is DECIDED and written: severity x tier, critical caps, labels excluded from the
      denominator. ★ A decision to keep flat scoring is a valid discharge of this criterion --
      declining with reasons is still proof.

## SCORING DECIDED 2026-09-04

A Critical-tier Fail, Unknown or Blocked CAPS the reported health at 50. Everything else
stays one check, one vote: (passed * 100) / determinable, with Unknown and Blocked out of
the denominator per INT-148.

PROVEN BY WATCHING IT FAIL. brightnessctl is one of the twelve binaries check_binaries
requires and nothing else in the forest uses it. Moved aside:
    Binary Dependencies -- 1 binaries missing
    Health: 50% (21 of 26 determinable)
Raw was 21/26 = 80%. The cap forced 50. Restored, health returned to 84%. A desktop
notification fired on the critical failure, so the event path works end to end.

⚠️ UNKNOWN AT CRITICAL TIER CAPS TOO, for the reason the verdict function already treats
them alike: not knowing whether the disk is full is not the same as it being fine.

★ 50 RATHER THAN 0, so the gradation below survives -- a critical failure with everything
else passing must still score better than a critical failure with half the machine broken.

⚠️ REJECTED: tier-weighted arithmetic (Critical x3 / System x2 / User x1). More precise,
and impossible to verify by eye. The gate requiring the health output to state its basis
makes legibility a requirement, and a cap is one rule a reader can hold in their head.

⚠️ REJECTED: Warn costing less than Fail. Half-credit softens the number, and a soft
number is easier to ignore. Four warnings costing what four failures cost is deliberate.

STILL OPEN in this area: labels excluded from the denominator. There are no labels today --
the census found zero -- so there is nothing to exclude until the definition format exists.

- [x] The four-state / three-colour rendering is DECIDED, including the exact wording an UNKNOWN
      check shows so it cannot be misread as a failure.
<!-- DECIDED 2026-09-06, from shipped code rather than from design. Four states, three colours:
       Pass     green    OK
       Warn     amber    something is wrong and nothing is lost
       Fail     red      something is wrong and something IS lost
       Blocked  red      rendered with Fail; the check was prevented from running by policy
       Unknown  amber    THE CHECK COULD NOT RUN
     Unknown renders the glyph U+2754 and carries the check's OWN message as the reason, so the
     line reads e.g. "could not check tools registry at <path>: No such file or directory". The
     wording rule: the message says what could not be reached and why, never what is wrong --
     that is what stops it being misread as a failure.

     ⚠️ THE NON-OBVIOUS PART, and it is already in the code: UNKNOWN IS NOT UNIFORMLY NEUTRAL.
     mod.rs:64 returns Verdict::Red for (Critical, Unknown) and (Critical, Blocked), while every
     other Unknown only sets amber. A critical-tier gate that could not run HAS NOT PASSED --
     the same rule faelight-deadwood --strict already applies by exiting 1 on a skip. So
     "excluded from the verdict" is true at system/user/info tier and FALSE at critical tier.
     Any future definition format must carry this distinction; flattening it would let the one
     check that matters go quiet by failing to run.

     Tier::Info is excluded from the verdict loop entirely (mod.rs:61), which is the separate
     label case -- measured, never judged.

     AND THE WORD AND THE COLOUR COME FROM ONE CALL. run_quick derives both from the same
     verdict(&checks) (mod.rs:1236 and 1242). They used to be separate rules, so an Unknown at
     critical tier printed DEGRADED in bright green -- the word and the colour disagreeing
     inside one string. Fixed under this intent; recorded here because the fix is the decision. -->
- [x] Quick scan and full scan are named, and it is stated which runs when and what each contains.
<!-- DECIDED 2026-09-06. ⚠️ THE PREMISE OF THIS GATE WAS WRONG AND THE CODE IS BETTER THAN IT ASKED
     FOR. The gate says "both lists already exist and neither is named". There is now ONE list:
     run_quick (mod.rs:1202) calls all_checks() and filters to Tier::Critical. The six-check
     second list this intent censused at mod.rs:1121-1126 on 2026-08-17 is GONE -- those line
     numbers are forecast code today.

       full scan   = all_checks(), 27 checks. Runs on `core doctor run` (the `d` alias) and is
                     the panel everything else quotes.
       quick scan  = the same all_checks(), filtered to Tier::Critical -- today check_binaries,
                     check_boot_errors, check_disk_space. Runs on the hand-typed
                     `core doctor quick`.

     ★ QUICK SCAN IS DERIVED, NOT COPIED, and that is the durable result. The comment at
     mod.rs:1208 records what the hardcoded version cost: the old five contained git-is-dirty and
     scripts-executable while boot errors and disk space were absent ENTIRELY -- a quick scan
     that skipped the boot chain. Deleting a check meant editing two lists and only the compiler
     noticed the second. A tier filter cannot drift from the tier table; a hand-kept list always
     will. This is the same argument as declaration-over-detection, applied to the scan lists.

     CONSEQUENCE OF THE TIER RULING (gate above): the quick scan's contents are now decided by
     the tier assignment, not chosen separately. So the check_disk_space disagreement recorded
     there is ALSO a disagreement about what quick scan contains -- re-tiering it to System
     would remove it from quick scan. That coupling is correct and should be preserved by any
     definition format: what breaks the system is what a quick scan looks at.

     ⚠️ NOT DECIDED HERE: run_quick still runs all 27 eagerly and discards 24 results. The
     comment at mod.rs:1403 states this is deliberate -- one hand-typed caller, nobody waits on
     it. That reasoning DIES if quick scan is ever wired into session start (INT-124 freshness),
     which the Solution section proposes. Lazy evaluation is a prerequisite for that, not for
     this gate. -->

## THE SIXTH PROBLEM IS ANSWERED, 2026-09-18 -- A SEPARATE CRATE

The sixth problem asked whether the definition format, probe registry and scoring serve BOTH
doctors, or whether `nsh doctor` stays separate and drifts the same way.

**DECIDED: neither. The doctor engine becomes its own crate, and both doctors use it.**

### Why not "share by importing core"

Measured 2026-09-18:

    novashell/Cargo.toml   faelight-core, faelight-git.  NO EDGE TO `core`.
    engine/Cargo.toml      name = "core", 57 domains in one crate

For `nsh doctor` to use a format defined inside `core`, novashell would have to depend on
`core` -- which today means depending on ~57 domains including Intelligence. ⚠️ THAT IS THE
EXACT EDGE INT-223 EXISTS TO REMOVE, and decision 147 built its whole ownership model around
not creating it.

So "one shared engine, living in core" is not available. It was the obvious answer and it is
the wrong one, for a reason that has nothing to do with doctors.

### Why not "leave them separate"

Both doctors were wrong at a similar rate the first time anyone read them -- 27 of 34 in the
big one, 3 of 7 in the small one -- and the defects were STRUCTURAL, not domain-specific:
a hardcoded path, a hand-built path beside a correct one, a check that could never succeed.
Separate implementations reproduce that class forever.

### What the crate owns, and what it does not

    THE CRATE OWNS      the definition format, the four states, the three colours, the
                        scoring rule, pass-only-is-a-label, the no-assertion-no-probe
                        rejection, the INT-199 red-render shape, the probe registry MECHANISM
    EACH CALLER OWNS    its own registry CONTENTS and its own tiers

⭐ SEPARATE REGISTRIES ARE THE POINT. `core doctor` checks the SYSTEM; `nsh doctor` checks the
SHELL -- its binary, its database, its aliases. They genuinely want different tiers, and 222
already said so. A shared engine prevents the structural class; separate registries keep each
doctor's subject its own.

### ★ AND THE EXTRACTION IS A MEASUREMENT, NOT JUST A MOVE

`domains/doctor/` holds 35 `check_` sites and is among the largest domains in a crate that now
carries FIFTY-SEVEN of them -- including `nix`, `friday_arch`, `bootstrap` and `daemon`, written
when the system had more tools and a different operating system.

Pulling the doctor out is the first honest answer to "how much of `core` is actually core".
Whatever slimming that reveals is NOT authorised here and is not this intent's work.

### ⚠️ WHAT THIS DECISION DOES NOT AUTHORISE

Decision 147's own warning applies to this intent as much as to the event bus:

    "Treating a resolved decision as permission to start work is the thread-proliferation
     failure already named in this project's own history."

DECIDED NOW: the format, and that it lives in its own crate.
BUILT AT: Phase 2 of this intent, in phase order, with each gate proven by watching it fail.
NOT AUTHORISED: engine slimming, domain retirement, or any crate split beyond the doctor.

## THE PROBE REGISTRY, MEASURED 2026-09-18

Enumerated from what the code ALREADY DOES, not from what a design imagined it would need.

### Method, and the false start worth keeping

The first census scanned `check_` function BODIES for external calls and reported SEVEN of 33 as
"pure -- no external access", which would have made them labels by the rule above.

⚠️ IT WAS WRONG, and `check_reboot_needed` proved it: reported pure, while visibly comparing the
running kernel against the installed one in every `d` run. It calls `running_kernel()` and
`installed_kernels()` -- the access is ONE LEVEL DOWN, in helpers.

★ THAT IS THE FINDING, not a footnote. The probes are not the checks; they are the HELPERS the
checks call. A registry derived from check bodies would have been confidently empty in seven
places. Re-run against helpers: SEVENTEEN touch the outside world.

### The twelve binaries

`subprocess` is too coarse to be a probe. The doctor shells out to twelve distinct commands:

    systemctl 7   git 5   which 4   sh 2   rustc 2   cargo 2
    uname 1   systemd-analyze 1   pgrep 1   pacman 1   journalctl 1
    faelight-deadwood 1

A definition saying `probe: subprocess` is not reviewable. One saying `probe: pacman_orphans` is.
⭐ THE PROBE IS NAMED FOR WHAT IT ASKS, NEVER FOR HOW IT ASKS.

### The registry -- fifteen, closed

    systemctl_units        is a unit loaded / active
    git_status             working tree state, hooks path, HEAD
    which_binary           is a command on PATH
    rustc_version          toolchain present and its version
    cargo_doc              doc build clean
    kernel_running         uname -r
    installed_kernels      what is on disk to boot
    boot_time              systemd-analyze
    process_running        pgrep
    pacman_orphans         pacman -Qdtq
    journal_errors         journalctl since boot
    deadwood_scan          faelight-deadwood
    resolve_binary_path    package name -> real binary path      ⚠️ SEE BELOW
    sqlite_query           the forest database
    fs_read                read_to_string / read_dir / exists / metadata

⚠️ ADDING A PROBE IS AN EDIT TO THIS LIST AND TO THE CRATE. A definition can only NAME one. There
is no field that carries a command string, and that absence is the gate.

### ⚠️ resolve_binary_path IS THE DOOR, AND IT IS OPEN TODAY

Two sites in `entropy.rs` (156, 250) -- the same operation, written twice:

    Command::new("sh").args(["-c", &format!("readlink -f $(command -v {}) 2>/dev/null", pkg)])

`pkg` is INTERPOLATED INTO A SHELL STRING. Today the names come from a registry rather than from
input, so nothing exploits it -- but this is structurally the "path from a definition to arbitrary
code" this gate exists to forbid, sitting inside the doctor that is supposed to forbid it.

RECORDED HERE, REWRITTEN IN PHASE 2: the probe is defined as the OPERATION -- resolve a package
name to its real binary path -- and its implementation becomes native Rust (`which`, already a
workspace dependency, plus `fs::canonicalize`). No `sh`, no interpolation, and the duplication
collapses to one.

★ The registry cannot be called CLOSED while one of its members is a shell. It is enumerated now;
it is closed when that rewrite lands.

## WHAT IS ALREADY BUILT, AND THE DEFECT THAT PROVES GATE 3, 2026-09-18

Read before Phase 2 writes anything: MORE OF THIS INTENT EXISTS IN CODE THAN THE GATE LIST
IMPLIES, and the new crate should ADOPT that vocabulary rather than invent a parallel one.

### Already in domains/doctor/mod.rs

    Status       Pass, Warn, Fail, Blocked, Unknown
                 Unknown is documented exactly as this intent wanted: "the check could not
                 run -- NOT a failure: excluded from the health denominator" (INT-148)
    Tier         Critical, System, User, Info
                 The first three are RISK.toml's, deliberately. Info is NOT RISK.toml's
                 fourth and the code says so: it means "measures truly but never judges".
    verdict()    critical-Fail -> Red. critical-Unknown or Blocked -> Red. Anything else
                 bad -> Amber. Info EXCLUDED from the loop entirely.
                 And it refuses arithmetic on purpose: "a weight factor is subjective and a
                 number built from one has to be defended forever."

★ SO THE SCORING SECTION OF THIS INTENT IS SUBSTANTIALLY IMPLEMENTED. `Tier::Info` is the label
mechanism, built by tier rather than by declared severity range.

### ⚠️ AND THAT IS WHERE THE DEFECT IS. THEY ARE NOT THE SAME THING.

    Tier::Info    THIS CHECK never judges          -- a property of the CHECK
    pass-only     THIS DEFINITION can only pass    -- a property of the DEFINITION

Nothing enforces that an Info-tier check emits only Pass. `check_vm_state` proves it, at
checks.rs:1556-1564:

    Err(_) => {
        return CheckResult {
            tier: Tier::Info,
            status: Status::Warn,
            message: "Could not check for running VMs",

⭐ AN INFO-TIER CHECK RETURNING WARN IS A WARNING THAT CANNOT REACH THE VERDICT. `verdict()`
filters Info out at line 60, so this branch renders a yellow line and changes nothing. The
system can say "could not check" and still report Green.

AND THE STATUS IS WRONG TWICE OVER: "could not check" is precisely what `Status::Unknown` was
added for, per its own doc comment. This arm reaches for Warn instead, in a tier where Warn is
inert -- so the one state that WOULD have been honoured is the one not used.

### What gate 3 therefore means, concretely

Not "add a label concept" -- one exists. It means: **a definition's declared severity range is a
CONSTRAINT the engine enforces, so a pass-only definition CANNOT return Warn.** `check_vm_state`'s
shape must become unrepresentable, not merely discouraged.

That is the difference between the tier mechanism (a convention two of three checks happen to
follow) and the format mechanism (a rule the engine applies).

⚠️ NOT FIXED HERE. `check_vm_state` is corrected when it migrates in Phase 2 step 3, by the
engine refusing it -- which is also how gate 3 gets its "proven by watching it work".

## THE DIVISION OF LABOUR, DECIDED 2026-09-18 -- AND IT NARROWS AN EARLIER PROMISE

Step 2 began by asking what argument a probe takes. The census answered a bigger question instead.

### What the 33 checks actually look like

Counting external calls per check body (Command::new, read_to_string, read_dir, exists, sqlite,
plus calls to the doctor's own helpers):

    14 of 33    SIMPLE      one call or none
     6          two calls
    13          COMPOSE     three or more

    check_drift              1608 lines, 24 calls
    check_orphan_packages     519 lines,  8 calls
    check_conflicts           276 lines, 15 calls
    check_services            102 lines,  3 calls -- and it DISCOVERS its own unit list from
                              `systemctl show -p Wants`, then asks is-active for each

⚠️ `check_drift` IS NOT A CHECK. It is a subsystem with a check-shaped return value. No TOML
format short of a programming language expresses it, and this intent explicitly forbids becoming
one: "there is no path from a definition to arbitrary code."

### ⚠️⚠️ THE NUMBERS ABOVE ARE WRONG. CORRECTED 2026-09-18, SAME DAY.

The census that produced "13 of 33 compose, check_drift 1608 lines" was WRONG THREE WAYS, and
every error inflated the complexity in the direction that flattered the decision:

    counted check_binaries TWICE       two functions share the name in different files
    swept in the NEXT function's       `check_update_readiness` showed Info+User because the
    trailing comment                   scan ran to the next `fn` and caught a comment about
                                       the function AFTER it
    counted code that never runs       `check_drift` is `fn check_drift(&EntropyBaseline) ->
                                       DriftReport` in entropy.rs, referenced ZERO times in
                                       mod.rs. IT IS NOT A DOCTOR CHECK. It is doctor entropy's
                                       own machinery that happens to start with `check_`.

Re-run against `all_checks()` -- the registration list, which is the authority:

    28 registered
    6 of 28 compose (3+ external calls)
    LARGEST IS check_security_audit AT 147 LINES
    check_orphan_packages is 50 lines, not 519

★ NOTHING IN THE DOCTOR IS 1608 LINES. The argument "no TOML expresses this without becoming a
programming language" was made against a function that is not in the check set.

### THE DECISION SURVIVES, FOR A BETTER REASON -- READ THIS ONE INSTEAD

`check_reboot_needed` was read in full as the strongest case AGAINST the split: 60 lines, zero
subprocess calls in its own body, about as simple as a check gets. What it contains:

    MEASURE      running_kernel(), installed_kernels()
    COMPARE      installed.iter().any(|k| k == &running)          <- ONE LINE
    CONSTRUCT    four CheckResult literals                        <- ~45 OF ITS 60 LINES

⭐ THE LOGIC IS A MEMBERSHIP TEST. THE BULK IS BOILERPLATE.

So the real division is not "simple checks are data, complex ones are code". It is:

    THE MEASUREMENT AND ITS COMPARISON ARE INSEPARABLE. "is the running kernel among the
    installed ones" means nothing without knowing what those two calls return. Splitting them
    into a probe that fetches and a TOML that compares would put half a thought in each place.

    THE DECLARATION IS ENTIRELY SEPARABLE. Tier, severity range, recovery text -- none of it
    depends on what the measurement returns.

    AND THE CONSTRUCTION IS PURE BOILERPLATE THE ENGINE SHOULD OWN. Four near-identical struct
    literals per check, times 28. The `Err(_) => Status::Unknown` arms are copied verbatim
    everywhere.

### ★ THAT THIRD LINE IS THE REAL PRIZE, AND IT WAS NOT IN THE ORIGINAL ARGUMENT

EVERY DEFECT THIS INTENT WAS FILED AGAINST LIVED IN A HAND-WRITTEN `CheckResult` LITERAL:

    check_vm_state        `tier: Tier::Info` beside `status: Status::Warn`, in one literal
    check_dotmeta         `status: Status::Pass`, typed, unconditional
    "Login shell ✅"       the same
    27 of 34 structural   no declared range, because a literal cannot declare one

Nobody wrote those defects on purpose. They wrote them because writing a CheckResult by hand
means re-deciding the tier and the status at every construction site, forty-five lines at a time,
with nothing checking the answer.

Under this design a probe returns a measurement and its verdict; the ENGINE constructs the
result, enforces the declared range, decides the denominator and renders. NOBODY HAND-WRITES A
CheckResult AGAIN, and the class of defect becomes unwritable rather than merely discouraged.

### ★ THE DECISION: THE DEFINITION DECLARES, THE PROBE MEASURES

    THE DEFINITION OWNS      id, name, tier, declared severity range, threshold source,
                             recovery text                    -- DATA, for all 27
    THE PROBE OWNS           the measurement, however complex, behind a name in the closed
                             registry                         -- RUST

`check_drift` becomes `probe: drift_scan`. Its 1608 lines stay in Rust, reachable only through a
registered name. The definition beside it declares what tier it is, what severities it may emit,
and what to do when it is red.

### Why this is the right split, and not a retreat

⭐ LOOK AT WHAT THE DEFECTS ACTUALLY WERE. Every one this intent was filed against:

    check_dotmeta          hardcoded Pass, could not fail          DECLARATION
    check_vm_state         Tier::Info + Status::Warn, inert        DECLARATION
    "Login shell ✅"        asserts a fact it never verifies        DECLARATION
    27 of 34 structural    no declared range at all                DECLARATION

NONE OF THEM WERE MEASUREMENT BUGS. The lying was in what the checks CLAIMED ABOUT THEMSELVES,
and that half is uniform, small, and data -- for all 27, including the 1608-line one.

Putting the declaration in TOML fixes the entire defect class. Putting the measurement there
would fix nothing that was broken and would build the arbitrary-code door this intent forbids.

### ⚠️ AND IT NARROWS "THE CHECKS ARE DATA", HONESTLY

The Solution section above says "updating the check set does not rebuild the engine". Under this
split that is HALF TRUE, and the half matters:

    NO REBUILD      retier a check, change its severity range, fix its recovery text, mark it
                    a label, add or remove a check from the active set
    REBUILD         a genuinely NEW measurement -- because that is a new probe, and adding a
                    probe was always meant to be a deliberate, reviewable act

★ That second line is not a regression. It is the probe-registry gate restated: the registry is
closed, and closed means new measurements cost a code review. What changed is the admission that
t
MOST checks need a probe rather than an assertion -- the `assertion` field serves the 14 simple
ones, not the majority.

## GATES 3 AND 4: BUILT, AND PROVEN RED FIRST, 2026-09-18

`faelight-doctor` exists: `status.rs` (adopted vocabulary), `probe.rs` (the fifteen, closed),
`definition.rs` (the format and the rules). 10 tests, 10 passing, zero warnings.

### The rules, in code

    validate()    no assertion + no probe -> NoAssertionNoProbe, NAMING THE ID
                  both -> BothAssertionAndProbe (one answer, two sources)
                  no severities -> NoSeverities
    is_label()    declares only Pass -> it is a label, by construction
    permits()     the declared range is a CONSTRAINT, not a description

### ⭐ PROVEN RED FIRST, WHICH IS THE POINT OF "proven by watching"

Both rules were DELIBERATELY STUBBED and the suite re-run:

    (None, None) => Ok(())               // gate 4 broken
    _ => true                            // gate 3 broken

    FAILED  no_assertion_and_no_probe_is_refused
    FAILED  completing_it_makes_it_accepted
    FAILED  a_label_cannot_report_warn_or_fail
    FAILED  a_warn_only_definition_cannot_fail
    ok      the six unrelated to the broken rules

And the compiler added its own evidence: `warning: method `permits` is never used` -- stubbing
the rule made its helper dead code. Restored: 10/10, no warnings.

⭐ A TEST THAT HAS NEVER BEEN SEEN RED IS AN ASSERTION, NOT A PROOF. The same rule caught a
false pass in INT-251 the day before.

### Still open

The TOML registry (step 3), the probes (step 2), and gate 5's INT-199 render. The engine is
built and no check has moved -- which is the order the scope asked for.

## PHASE 2 SCOPE, AGREED 2026-09-18 -- WHAT GETS BUILT

### The crate

    faelight-doctor     the ENGINE. Depends on faelight-core ONLY.
        ^          ^
    core        novashell        each brings its own registry

### Contents

    Definition    id, name, tier, declared severity range, assertion | probe,
                  threshold source, recovery text
    Status        Pass / Warn / Fail / Unknown
    Tier          FROM RISK.toml -- critical / system / user. Not invented; 222 already
                  ruled "do not invent a second scale".
    Probe         the fifteen, AS AN ENUM. A definition NAMES one. There is no field that
                  takes a string, and that absence is the gate.
    Scoring       critical-Fail caps the verdict; labels excluded from the denominator;
                  output states its basis, not a bare percentage
    Render        INT-199 shape for red: result, reason, comparison, likely cause, recovery
    Validation    no assertion and no probe -> REJECTED
                  pass-only -> LABEL, excluded from the denominator

### DECIDED: definitions are TOML, in faelight/registry/doctor/

222 already said "the runner is code, the checks are data -- updating the check set does not
rebuild the engine". TOML honours that; a const array in Rust does not.

⚠️ THE COST, STATED: a malformed definition becomes a RUNTIME failure where a Rust array would
have been a compile error. That is acceptable ONLY because validation is a gate of this intent --
a definition with no assertion and no probe is REJECTED, and the rejection is proven by watching
it happen. Without that gate, TOML would be trading one silent failure for another.

`faelight/registry/` already holds tools.toml and the schema-validated registry files, and the
doctor check-set is the same kind of thing.

### DECIDED: all 27 checks migrate AT ONCE, not incrementally

★ CHRISTIAN'S CALL, AND IT IS THE RIGHT ONE. Incremental migration means two mechanisms both
claiming to be the doctor for however long it takes -- which is EXACTLY the defect class this
whole week has been spent removing: two alias owners, two doctors, two state trees, two writers
of the caret. A migration that creates a second owner to fix a second owner has not understood
its own thesis.

Big-bang is longer and it is honest. There is never a moment where "which doctor answered this"
is a question anyone has to ask.

### What lands in what order

    1. the crate, the types, the validation -- with NO registry. Gates 3 and 4 are proven here,
       against fabricated definitions, before any real check moves.
    2. the fifteen probes, ported from the helpers the census enumerated
    3. all 27 core-doctor definitions, in one change
    4. `nsh doctor`'s seven, in one change
    5. gate 5 -- the INT-199 red render

⚠️ STEP 1 PROVES GATES 3 AND 4 BEFORE STEP 3 EXISTS. That ordering is deliberate: the rejection
and the label rule must be demonstrated on definitions written to fail, not discovered while
migrating real ones.

### Not in scope, again

Engine slimming. Domain retirement. The 57-domain question. `nsh doctor`'s three known defects
beyond routing them through the new engine. INT-102's version bump for nsh-test.

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
- [x] A definition declaring pass-only is treated as a label and excluded from the denominator.
      **Proven by watching it work: fabricate a pass-only definition, watch it be excluded and
      reported as declared, then remove it and watch the denominator return.**
- [x] A definition with no assertion and no probe is REJECTED. **Proven by watching it fail:** write
      one, watch it be refused, then complete it and watch it accepted.
- [x] A critical-tier ERROR caps the reported health. **Proven by watching it fail:** force a
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
- [x] `faelight-deadwood` gains a mechanical check for a check that cannot fail. **Proven by
      watching it fail first:** reintroduce a hardcoded-Pass definition, watch it be flagged, remove
      it, watch the flag clear. ⚠️ Without the fail-first proof this gate is itself decoration --
      a fourth sighting of the disease, inside the fix for it.

### Phase 3 -- output and hygiene

      <!-- DONE 2026-09-04, fail-first proven.
      Rule: a function returning CheckResult whose body mentions Status::Pass and no other
      Status variant. Comments stripped first -- a comment naming Status::Fail does not give
      a function a way to return one.
      clean tree: clean. Decoy appended, in check_dotmeta exact shape:
        [MED] checks.rs:1859 check_zzdecoy can only return Pass -- it cannot fail
      Decoy removed: clean. git diff --stat empty, no residue. -->
- [x] The health output states its basis rather than a bare percentage.
- [x] Every red check renders INT-199 shape. DONE 2026-09-19 in `faelight-doctor/src/render.rs`,
      proven on live red outcomes (alias_coverage, update_readiness, security_audit).
      <!-- THE SHAPE ABOVE WAS WRITTEN WRONG AND IS CORRECTED HERE. This gate said "result
      first, reason, comparison, likely cause, recovery" -- it OMITTED Status and invented a
      "comparison" section INT-199 does not have. Read from 199 itself on 2026-09-19, the real
      shape is: Status, Result, Reason, Possible causes, Recovery, Debug. The four questions it
      answers, in order, are WHAT HAPPENED / DID ANYTHING CHANGE / WHY / WHAT DO I DO NEXT.

      Two deliberate departures, both recorded rather than silent:

      NO "POSSIBLE CAUSES" SECTION. fpatch could list causes because it knew what it had
      attempted. A CHECK KNOWS WHAT IT MEASURED, NOT WHY THE MACHINE IS THAT WAY. Inventing
      plausible causes would be decoration presented as diagnosis -- the defect this intent
      exists to remove -- so the section is omitted rather than guessed.

      RESULT IS CONSTANT, AND THAT IS WHY IT IS STATED. INT-199 principle 2 is "tell the user
      what did not happen". A doctor only reads, so the answer never varies: "Nothing was
      changed." A reader who has just been told something is wrong must not be left wondering
      whether the checking made it worse. That is the exact fact fpatch never printed.

      Status maps onto 199's taxonomy: Pass->Info, Warn->Warning, Fail->Failure, and
      Unknown/Blocked->SAFE ABORT -- the check stopped rather than report something it could not
      stand behind, which is 199's central distinction seen from the other side.

      RED ONLY, per 199's own scope guardrail ("adopt it where failures are actually being
      read"). A passing line is read as a tick; six sections would bury the report.

      ⚠️ AND IT IMMEDIATELY FOUND SOMETHING: only 6 of 28 definitions declare a recovery, so 22
      red checks render "No recovery step is declared for this check." That is honest and it is
      not finished -- the old fix: strings exist but several are stale (configuration.nix, a
      scripts/ deploy path, `deploy <tool>`, and a 3b link instruction already carried out).
      Auditing each against this machine is INT-253's group 1, whose gate demands every
      replacement be RUN here before it is declared. Recorded there, not absorbed here. -->
- [~] Generation-count thresholds derive from ESP size and `configurationLimit` rather than typed
      constants.
      <!-- DEAD 2026-09-04. ESP size and configurationLimit are NixOS concepts and there
      are no generations to count. The subject went with the migration on 2026-08-28,
      the same way INT-129's adapt-or-retire options did. Kept as history. -->
- [x] The stale check counts are corrected wherever they appear, and the number is DERIVED rather
      than typed so it cannot go stale a fourth time.
      <!-- DONE 2026-09-04, and NOT by rewriting thirteen numbers.

      grep found 34 in thirteen places in this document and NOWHERE in code or docs. So the
      stale count never reached anything that runs -- WORKFLOWS.md said 22 and POLICIES.md
      said 14, but neither says a number now.

      ⚠️ AND MOST OF THE THIRTEEN ARE HISTORY, NOT ERROR. The census at Phase 0, the finding
      that only seven of 34 could report Fail, the note that a second doctor exists -- all
      were measured on 2026-08-17 when the count WAS 34. Editing them would falsify the
      record of what was true when the work was done.

      The correction is the banner at the top of this intent, and the derivation is what
      the doctor already does: the header prints 22/27 from all_checks().len(), not from a
      typed constant. Nothing computes a check count from a written number.

      TODAY: 27 checks. 2026-08-17: 34. The difference is checks deleted with their
      subjects at the Omarchy migration -- stow, mango, the NixOS theme packages, and
      check_dotmeta itself. -->
- [~] `rebuild-safe` is reviewed against the new scoring and it is stated -- with a reason --
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

      <!-- DEAD 2026-09-04. There is no rebuild-safe to review.
      It survives as a NAME in three lists -- one deadwood keyword list and two nsh
      completion lists -- with no implementation and no command behind it. Same class as
      check_compositor: PHANTOM, per the census above.
      ⚠️ And nsh offers tab-completion for a command that does not exist, which is its own
      small defect and not this intent's. -->
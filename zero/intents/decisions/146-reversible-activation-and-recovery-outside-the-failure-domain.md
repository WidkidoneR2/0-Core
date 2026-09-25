---
id: 146
date: 2026-08-17
type: decision
title: "reversible activation and recovery outside the failure domain"
status: decided
tags: [decision]
---

## Context

Decision 145 answers what Zero Core owns. This one answers a different question: **what is
guaranteed when the system transitions from one state to another.**

The claim that prompted it was wrong, and correcting it produced a better rule than the one it
replaced.

⚠️ THE WRONG CLAIM: *"activation is transactional, so a bad configuration rolls back rather than
bricking anything."*

NixOS does not guarantee that. It gives immutable, independently bootable generations plus explicit
rollback. Activation itself is not atomic -- activation scripts, systemd changes, file creation and
migrations can fail halfway **with side effects**, and nothing returns the external world to its
previous state.

★ This is not theoretical here. The greetd/tuigreet lockout and the `vm down` unsynced-write loss
were both activation-side-effect failures on this system.

## Decision

### 1. Reversible, not transactional

The correct statement of what the substrate provides:

> System generations are immutable and independently bootable, so a bad configuration can be
> tested, selected as a generation, and rolled back **without reconstructing the previous system
> from backups.**

And the rule that follows:

> **Do not promise transactional activation. Design for reversible activation.**

### 2. The definition, and it is testable

> **An activation is reversible when a failed or undesirable activation leaves a documented,
> independently reachable path to a known-good system state.**

Note what this does *not* claim: that activation is atomic, that side effects are undone, that
systemd state rolls back, or that external state is transactional.

★ THE TEST: **deliberately break activation, then try to reach the documented recovery path.**
If you get back to a known-good state, the module satisfies the contract. If you do not, it does
not.

⚠️ That is the same discipline `docs/CONVENTIONS.md` already requires of gates -- *a gate you have
only watched pass might be doing nothing.* Applied to activation instead of to checks.

### 3. The activation contract

Every module that performs activation-time side effects declares:

- **Side effects** -- what it changes outside the store
- **Partial failure** -- what state can exist after a half-completed activation
- **Retry semantics** -- can activation be safely rerun after correction
- **Recovery path** -- how a known-good state is reached
- **Lockout class** -- `none` | `recoverable` | `lockout`

Worked example, greetd:

```
side effects:   installs and enables the display manager configuration
partial failure: graphical login may become unavailable
retry:          safe to rerun after correction
recovery:       F3 at the greeter, or TTY2 -> SafeShell
lockout class:  lockout
```

### 4. RISK.toml is the enforcement surface, not prose

⚠️ A contract written as prose in each module is the exact shape of every failure this repository
has already had: faelight-hooks defined and never wired, the rustfmt hook "unskippable" while the
hook file did not exist, `check_dotmeta` asserting a fact it never verifies, four documents accurate
on the day they were written. **In six months there would be twelve modules with contracts and
twenty without, and no way to know which.**

★ Do not invent a schema. `RISK.toml` already exists per directory and **already encodes the
lockout class**: *critical -- boot, login, disk. Failure means the machine does not come back.*

The contract fields become `RISK.toml` keys, and `faelight-deadwood` gains a check that flags a
module with activation side effects and no contract.

⚠️ The check must be proven by watching it fail first, or it joins the list above.

### 5. Recovery must live outside the failure domain

> **A lockout-class component must have a recovery path that does not depend on that component
> being functional.**

★ This is considerably stronger than "we have a rollback." A rollback command is useless if the
system prevents you from reaching the environment that runs it.

```
greetd            -> recovery: TTY2 / SafeShell     (independent)
desktop shell     -> recovery: independent TTY      (independent)
boot configuration-> recovery: known-good generation
network service   -> recovery: local console
```

### 6. Unverified recovery must be distinguishable from tested recovery

> **RISK.toml should not merely describe risk. It should expose missing recovery dependencies.
> A critical component whose recovery artifact is UNVERIFIED must be mechanically distinguishable
> from one whose recovery path has actually been tested.**

A `recovery_verified` field carrying a date, or the absence of one.

⚠️ This is INT-192 applied to recovery -- a tool that cannot express UNDETERMINED reports clean.
Without it, "we have a backup" and "we have a backup that works" look identical, and the difference
only surfaces on the day it matters.

## Applied -- the boot chain fails this today

Measured 2026-08-17.

**The chain is healthy.** Secure Boot enabled (user), TPM2 present, Measured UKI yes, PK/KEK/db all
present in `/var/lib/sbctl/keys`, firmware menu reachable via `systemctl reboot --firmware-setup`.

⚠️ **But the recovery material is not independently recoverable.**

- The signing keys exist in exactly one place -- `/var/lib/sbctl`, on the LUKS-encrypted root. They
  sign every UKI. If the disk or the LUKS header is lost, no bootable signed image can be produced.
- The rescue USB is **rejected** while Secure Boot is enforcing -- measured twice, and the second
  time it silently fell through and booted the signed disk, so the media appears to do nothing at
  all.
- The documented path is therefore: firmware menu, supervisor credential, factory reset,
  re-enrolment. **That depends on firmware access and on a password, not on anything the system can
  guarantee.**

★ **THE RECLASSIFICATION THAT FOLLOWS: `/var/lib/sbctl` and the EFI-variable backup are part of the
boot chain's recovery boundary, not backup data.** Components get contracts. Backups get forgotten.

⏭ The verification work is INT-225, not this decision.

## Deliberately not decided here

- Whether to change anything about Lanzaboote. The chain is healthy; the recovery material is the
  problem.
- The exact `RISK.toml` key names and types.
- Whether boot counting is enabled. Unknown, and worth knowing -- it would be the only genuinely
  independent recovery mechanism in the boot chain, needing no human, no firmware menu and no keys.

## Consequences

- "Reversible" becomes something a module either satisfies or does not, checked by breaking it.
- The lockout class stops being a feeling and becomes a field.
- A critical component with an untested recovery path is visible instead of silent.
- The highest-risk component in the system now has a named weakness rather than an assumed strength.

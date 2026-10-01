# Fingerprint

How Project 0 decides that this machine and this tree are the ones it thinks they are.

The rules agents must not break live in AGENTS.md under Fingerprint. This file is the flow those rules describe. Do not copy the rules back into AGENTS.

Written 2026-09-30. Collector paths below are the declared sites to look first. If the adapter has moved and this list is stale, fix this file in the same change that finds the new site. Do not add a second collector to paper over a stale path.

## What it is

A fingerprint is a declared identity of two things at once:

- The machine — facts the kernel already publishes.
- The tree — facts this checkout already owns.

It is not a browser fingerprint, not a biometric, not a session token, and not a version string. Version strings lie (the shipped nsh can trail HEAD with no warning). A fingerprint is asked about; a version is printed.

It is computed once, in one adapter, and every consumer asks that adapter. Same class of rule as INT-230: one place the shell asks about 0-Core. Absent capability degrades visibly. Runtime proof, not a feature flag.

## The flow

```text
declared inputs
    → one collector
        → fingerprint record (facts + missing-set)
            → compare against last known / expected
                → PASS | FAIL | UNDETERMINED
                    → doctor / integrity / DevBox / devshell Law 0
```

Nothing in that chain invents a field. If a probe cannot read an input, the record carries the gap. Comparison does not fill gaps with defaults.

### 1. Declared inputs

Four probe families, named by INT-257's launch checks:

| Family | Asks | Typical sources |
|---|---|---|
| identity | who this installation claims to be | hostname, machine-id the kernel exposes |
| machine | what hardware and substrate this is | arch, kernel, the Omarchy host facts |
| process | what is running this check | uid, the binary path actually exec'd |
| filesystem | what tree and state dirs are real | `~/0-core`, `~/.local/state/zero`, `~/.config/nsh` |

A field that is not on the declared list is not part of the fingerprint. Adding a field is a contract change. It needs an intent, a red-first test, and an update to this table.

### 2. One collector

Look first (historical names still in tree during INT-247):

- integrity code next to zero-core / zero-* (`integrity/mod.rs` is the name AGENTS already cites)
- `zero-doctor/src/probes/` (where checks run) and `registry/doctor/checks.toml` (where they are declared)
- devshell Law 0 launch probes (`zero/scripts/devshell` and `devshell-lib` — INT-252 moves the directory; the adapter moves with it)

If two of those compute a hash independently, that is a defect. One writes the record. The others ask.

### 3. The record

The record is facts plus a missing-set, not a single opaque digest pretending every input was read.

- Every declared input is either a value or an explicit absence.
- Absence is UNDETERMINED material, not a zero, not a skip, not a "close enough".
- The digest, if one exists, is over the declared tuple only.
- Secrets, undeclared home paths, network reachability, and timestamps are not inputs. A fingerprint that moves because a session started is not a fingerprint.

### 4. Compare

Compare live record to last known / expected.

| Live | Expected | Result |
|---|---|---|
| all declared inputs read, equal | present | PASS |
| all declared inputs read, differ | present | FAIL |
| any declared input unread | anything | UNDETERMINED |
| expected missing and that was the claim | expected missing | PASS on that axis only if the missing-set matches |

Doctor already distinguishes clean from UNDETERMINED (INT-192). Do not collapse the two so a dashboard looks greener.

FAIL is a mismatch. UNDETERMINED is "we did not establish the answer." A tool that cannot answer must say so.

### 5. Consumers

They read. They do not grow a private copy.

- **doctor** — health surface. Prints PASS / FAIL / UNDETERMINED. Does not repair the fingerprint as a side effect of printing it.
- **integrity** — the adapter's home. Owns the record format.
- **DevBox verify** — case files, crate census. A case that needs identity asks the adapter; it does not hash the fixture tree itself and call that a fingerprint.
- **devshell Law 0** — ten checks at launch (INT-257). Identity / machine / process / filesystem probes consume the fingerprint. Any one failing means the session refuses to start rather than starting degraded. Each check was proven by breaking it on its own.

Law 0 consumes the fingerprint. It does not invent it.

## Stability

Rename is not a new machine. INT-247 path and display-name changes must not flip the fingerprint by themselves. If a rename moves a probe input (for example a state directory moving to `~/.local/state/zero`), the adapter is updated in the same change or the gate stays red.

Compatibility links are not a second identity. As of 2026-09-24 the real directories are `~/.local/state/zero`, `~/.config/zero`, `~/.cache/zero`, `~/.local/share/zero`, `~/.config/nsh`. The old names that stood beside them were links, removed in INT-247. The fingerprint follows the real directories.

A new checkout on the same machine is a new tree, not a new machine. Identity / machine axes should match; filesystem axes should not, and that difference is FAIL on the tree axis, not UNDETERMINED, once both trees were actually read.

A sandbox is not the host. devshell Law 7: leaving puts the machine back exactly as it was. A fingerprint taken inside the sandbox is the sandbox's fingerprint. Do not write it over the host record.

## What this is not

- Not Secure Boot. Secure Boot is not enforcing (measured 2026-09-05). Do not describe the fingerprint as a signed boot measurement.
- Not the nsh version string. That string does not change per commit.
- Not `git rev-parse HEAD`. HEAD is the tree's tip, useful, and a declared tree input if the collector lists it — it is not the whole fingerprint.
- Not network presence. Nothing inside devshell reaches the network; a fingerprint that needs the network cannot be checked there.

## Proof

The tell is the same as every other gate: watch it FAIL first.

- Break one declared input (rename a real state dir, point a probe at a missing file). Doctor must say UNDETERMINED or FAIL, never PASS.
- Change only a display name. Fingerprint must not move.
- Compute from two crates in one session. The records must be the same object, not two similar hashes.

If a gate has only ever passed, it has not been shown to test anything.

## When this file is wrong

The collector in tree is the authority. This document is a map.

If the map and the collector disagree, the collector wins and this file is updated in the same commit that noticed. Do not "fix" the disagreement by adding another hash.

# Fingerprint

How Project 0 decides that this machine and this tree are the ones it thinks they are.

The rules agents must not break live in AGENTS.md under Fingerprint. This file is the flow those rules describe. Do not copy the rules back into AGENTS.

Written 2026-09-30; corrected 2026-10-01 to what INT-269 built. Collector paths below are the declared sites to look first. If the adapter has moved and this list is stale, fix this file in the same change that finds the new site. Do not add a second collector to paper over a stale path.

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

Nine, approved 2026-10-01 (INT-269). Each is readable without privileges.

| Family | Input | Source |
|---|---|---|
| identity | `identity.machine_id` | `/etc/machine-id` |
| identity | `identity.hostname` | `/proc/sys/kernel/hostname` |
| machine | `machine.arch` | the build target (`std::env::consts::ARCH`) |
| machine | `machine.os_id` | `ID=` in `/etc/os-release` |
| machine | `machine.board` | `/sys/class/dmi/id/board_vendor` and `product_name` |
| machine | `machine.cpu` | `model name` in `/proc/cpuinfo` |
| process | `process.uid` | `Uid` in `/proc/self/status` |
| tree | `tree.repo` | inode of `~/0-core`, and the origin URL from its `.git/config` -- not its device number (INT-270) |
| tree | `tree.dirs` | the five real directories are directories, not links |

Excluded on purpose: boot-id (changes every boot), the kernel version (changes every update; `d` reports it), repo HEAD (the tip, not the identity), the product serial (needs root, and is close to a secret), the Omarchy version (absent on this machine, and per-update anyway), the running binary (differs between nsh and core, which would give two consumers two records), and the device number of `~/0-core` (btrfs hands a subvolume a new one at every mount: 58, then 59, across one boot on 2026-10-01). A test in fingerprint.rs scans the collector's own source and refuses device numbers, boot ids, clocks and uptime, so this list is enforced, not only written (INT-270).

A field that is not on the declared list is not part of the fingerprint. Adding a field is a contract change. It needs an intent, a red-first test, and an update to this table.

### 2. One collector

`zero/tools/zero-core/src/fingerprint.rs` -- `collect()` reads the nine inputs, `compare()` judges, `Record` holds them. Every tool links zero-core, so every consumer calls the same function and gets the same record. The digest is FNV-1a over the declared tuple only, written in that file and pinned by a test vector.

Two front doors use it today:

- `core fingerprint show` / `record` -- `zero/engine/src/domains/fingerprint/mod.rs`
- the doctor's Fingerprint check -- `zero/tools/zero-doctor/src/probes/identity.rs`, declared in `registry/doctor/checks.toml`

The expected record lives at `~/.local/state/zero/fingerprint` (`zero_core::paths::fingerprint_file()`). Only `core fingerprint record` writes it.

If two of those compute a hash independently, that is a defect. One writes the record. The others ask.

### 3. The record

The record is facts plus a missing-set, not a single opaque digest pretending every input was read.

- Every declared input is either a value or an explicit absence.
- Absence is UNDETERMINED material, not a zero, not a skip, not a "close enough".
- The digest, if one exists, is over the declared tuple only.
- Secrets, undeclared home paths, network reachability, and timestamps are not inputs. A fingerprint that moves because a session started is not a fingerprint.

The record carries `schema=2`, a line outside the declared list, so it never moves the digest. A record from another schema answers a different question, so compare is UNDETERMINED naming `schema` and says to run `core fingerprint record` -- never FAIL. A record with no schema line is schema 1, the INT-269 shape. Whoever changes what a declared input means raises `SCHEMA` in the same change (INT-270).

### 4. Compare

Compare the live record to the expected one.

| Live | Expected | Result |
|---|---|---|
| all declared inputs read, equal | present | PASS |
| all declared inputs read, some differ | present | FAIL, naming each input that differs with its recorded and live value |
| anything | from another schema, or none | UNDETERMINED -- re-record with `core fingerprint record` |
| any declared input unread | anything | UNDETERMINED, naming what was not read |
| anything | no record | UNDETERMINED -- not recorded yet |

`core fingerprint record` refuses to write a record with a missing input, so an expected record never carries a gap. Exit codes, so scripts can ask: `show` 0 PASS, 1 FAIL, 2 UNDETERMINED; `record` 0 written, 1 refused. `core doctor check fingerprint` answers with the same three.

Doctor already distinguishes clean from UNDETERMINED (INT-192). Do not collapse the two so a dashboard looks greener.

FAIL is a mismatch. UNDETERMINED is "we did not establish the answer." A tool that cannot answer must say so.

### 5. Consumers

They read. They do not grow a private copy. Status as of 2026-10-01:

- **core** -- built. `core fingerprint show` compares and never writes; `core fingerprint record` is the one writer.
- **doctor** -- built. The Fingerprint check, system tier, severities pass and fail. No record or an unread input is unknown, never a pass. It never writes the record. `core doctor check fingerprint` runs it alone, read-only, with no health history written.
- **integrity** -- planned, not a consumer yet. The record format is owned by zero-core, not by integrity.
- **DevBox verify** -- planned, its own intent. A case that needs identity asks the collector; it does not hash the fixture tree itself and call that a fingerprint.
- **devshell Law 0** -- planned, its own intent. Its natural use is the inverse check: the sandbox's fingerprint must DIFFER from the host's on the identity and machine axes. Today its ten launch checks (INT-257) ask that question themselves and do not read the fingerprint.

Law 0 consumes the fingerprint. It does not invent it.

## Stability

A reboot is not a new machine. No input may read a number the kernel hands out per mount or per boot; a btrfs subvolume gets a new device number at every mount (INT-270).

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

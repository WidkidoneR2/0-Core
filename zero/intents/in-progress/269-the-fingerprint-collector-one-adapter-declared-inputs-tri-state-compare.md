---
id: 269
date: 2026-10-01
type: future
title: "the fingerprint collector: one adapter, declared inputs, tri-state compare"
status: in-progress
tags: [fingerprint, novashell, nsh, agents]
---

## Vision

Project 0 can say, in one call, whether this machine and this tree are the ones it thinks they
are -- and when it cannot tell, it says so. docs/FINGERPRINT.md is the flow; AGENTS.md holds the
rules. This intent builds what both of them describe.

## The Problem

Recon 2026-10-01: nothing computes a fingerprint.

```text
    doctor        nine probe files, no identity probe
    DevBox        runs a clean room (HOME redirected), never checks identity
    integrity     INT-184, ledger and registry consistency -- not an identity record
    Law 0         ten devshell launch checks that ask the OPPOSITE question: that the sandbox
                  is NOT the host (uid 0 inside, hostname devshell, few processes)
```

AGENTS.md and docs/FINGERPRINT.md describe the collector in the present tense. Until it exists,
both overstate what is built. The good news in the recon: there is no private copy anywhere to
untangle, so ONE collector is a starting point, not a migration.

## The Solution

One module, zero-core/src/fingerprint.rs. Every tool links zero-core, so the doctor, core,
DevBox and nsh all call the same function and get the same record.

### The declared inputs -- nine, approved by Christian 2026-10-01

Each is readable without privileges (measured on the Framework 16 the same day).

```text
    identity     machine-id; hostname
    machine      arch; OS ID; board vendor and product; CPU model
    process      uid
    tree         ~/0-core device and inode, and the origin URL; the five real directories
                 (state, config, cache, share zero; ~/.config/nsh) exist as directories, not links
```

### Excluded on purpose, each by a FINGERPRINT.md rule

```text
    boot-id            changes every boot -- a fingerprint that moves when you blink
    kernel version     changes every update; an update is not a new machine (d reports it)
    repo HEAD          changes every commit; the tip, not the identity
    product serial     needs root, and is close to a secret
    Omarchy version    absent on this machine, and per-update anyway
    running binary     differs between nsh and core -- would break two consumers, one record
```

### The record, the compare, the writer

```text
    record     every declared input is a value or MISSING (the missing-set). The digest covers
               the declared tuple only, with a stable hash written in the code -- std's
               DefaultHasher may change between compiler versions
    compare    PASS: all read, all equal. FAIL: all read, some differ, the axes named.
               UNDETERMINED: any input unread. Missing is never filled with a default
    writer     only `core fingerprint record` writes ~/.local/state/zero/fingerprint. The doctor
               never writes; with no record its answer is UNDETERMINED: not recorded yet
```

### Order, each step red first

```text
    1  collector and compare in zero-core, unit tests with injected paths
    2  core fingerprint show / record
    3  doctor check: Fingerprint -- PASS, FAIL, UNDETERMINED
    4  docs: AGENTS.md (Christian's wording) and FINGERPRINT.md say what was built
```

### Not in scope

DevBox and Law 0 as consumers. Law 0's natural use is the inverse check -- the sandbox's
fingerprint must DIFFER from the host's on the identity and machine axes. Each gets its own
intent once this collector is proven.

## Success Criteria

- [ ] ONE COLLECTOR: zero_core::fingerprint::collect() reads the nine declared inputs, and a
      census shows no other crate reads them for identity
- [ ] RED FIRST: every proof test below is seen failing before the code that makes it pass
- [ ] MISSING IS NOT ZERO: an unreadable input (injected path) lands in the missing-set and the
      outcome is UNDETERMINED, never PASS
- [ ] A CHANGED INPUT IS FAIL: one differing input, all others read, gives FAIL naming its axis
- [ ] RENAME IS NOT A NEW MACHINE: a display-name change leaves the digest unchanged
- [ ] TWO CONSUMERS, ONE RECORD: core and the doctor produce identical records in one session
- [ ] STABLE DIGEST: the digest function is pinned by a test vector in the code
- [ ] ONE WRITER: only core fingerprint record writes the expected record; the doctor never does
- [ ] DOCTOR: a Fingerprint check shows PASS, FAIL and UNDETERMINED, each demonstrated on the
      deployed binary
- [ ] DOCS TRUE: AGENTS.md (wording approved by Christian) and docs/FINGERPRINT.md name the
      collector's real path and mark each consumer built or planned

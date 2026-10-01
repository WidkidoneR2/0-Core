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
      <!-- progress 2026-10-01: step 1 seen red -- 8 of 8 collector tests failed at todo!() before 98d6657b made them pass. Stays open until steps 2 and 3 are seen red too. -->
- [x] MISSING IS NOT ZERO: an unreadable input (injected path) lands in the missing-set and the
      outcome is UNDETERMINED, never PASS
      <!-- evidence: 98d6657b, zero-core fingerprint::tests::an_unreadable_input_is_missing_and_the_outcome_undetermined -- machine-id removed from a fake machine: missing-set names it, digest None, compare UNDETERMINED. Red at todo!() first. -->
- [x] A CHANGED INPUT IS FAIL: one differing input, all others read, gives FAIL naming its axis
      <!-- evidence: 98d6657b, fingerprint::tests::one_changed_input_is_fail_naming_it (hostname -> FAIL [identity.hostname]) and a_link_where_a_real_directory_belongs_is_fail_on_the_tree (-> FAIL [tree.dirs]). Red at todo!() first. -->
- [x] RENAME IS NOT A NEW MACHINE: a display-name change leaves the digest unchanged
      <!-- evidence: 98d6657b, fingerprint::tests::a_field_outside_the_declared_list_does_not_move_the_digest -- a display.name field added: same digest, compare PASS. The digest reads DECLARED only. Red at todo!() first. -->
- [ ] TWO CONSUMERS, ONE RECORD: core and the doctor produce identical records in one session
- [x] STABLE DIGEST: the digest function is pinned by a test vector in the code
      <!-- evidence: 98d6657b, fingerprint::tests::the_digest_function_is_pinned -- FNV-1a written in zero-core, fnv1a64("project 0") == 0x2501e42b18699b7e. Red at todo!() first. -->
- [ ] ONE WRITER: only core fingerprint record writes the expected record; the doctor never does
- [ ] DOCTOR: a Fingerprint check shows PASS, FAIL and UNDETERMINED, each demonstrated on the
      deployed binary
- [ ] DOCS TRUE: AGENTS.md (wording approved by Christian) and docs/FINGERPRINT.md name the
      collector's real path and mark each consumer built or planned

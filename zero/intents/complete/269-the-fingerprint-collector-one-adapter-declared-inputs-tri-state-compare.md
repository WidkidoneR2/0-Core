---
id: 269
date: 2026-10-01
type: future
title: "the fingerprint collector: one adapter, declared inputs, tri-state compare"
status: complete
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

## START HERE -- 2026-10-01

Written only if HEAD was 63228be9, clean and pushed; the commits below exist; and the recorded
fingerprint read back as digest 09c7905f4daea150 with `core fingerprint show` answering PASS.
A new chat opens with `ints 269` and this section.

### Done -- pushed

```text
    8f9db2d1   INT-269 filed and started: the design and ten gates
    98d6657b   step 1: zero-core/src/fingerprint.rs -- collect, compare, Record, FNV-1a. Eight
               tests on a fake machine, red at todo!() first, then green
    5f19ecb5   four gates ticked with evidence
    63228be9   step 2: core fingerprint show / record, and paths::fingerprint_file(). nsh-test
               core_fingerprint_lifecycle: red (unrecognized subcommand), then green -- the full
               UNDETERMINED, record, PASS, FAIL cycle on the deployed core in a temp state dir
    (disk)     Christian ran core fingerprint record: digest 09c7905f4daea150 at
               ~/.local/state/zero/fingerprint. show answers PASS
```

### Ruled, Christian 2026-10-01

```text
    no record, no move   record refuses any missing input; show without a record is UNDETERMINED
    exit codes           show 0 PASS, 1 FAIL, 2 UNDETERMINED; record 0 written, 1 refused
    step 3 is (a)        the doctor gains a read-only way to run ONE check, so FAIL and
                         UNDETERMINED are demonstrated on the deployed binary without writing a
                         test result into health history or the health cache
```

### Next -- step 3, in order

```text
    1  recon: how core doctor run writes -- the health history rows in state.db, the
       health-status cache, the trend and forecast inputs -- and every caller of the runner
    2  design (a): the read-only single-check mode; it writes nothing; Christian approves its
       name and shape before any code
    3  red first: the Fingerprint check -- a registry/doctor/checks.toml entry, Probe::Fingerprint,
       a probe mapping Pass, Fail, Undetermined to pass, fail, unknown. The count tests
       (27 checks, 25 judging) move to 28 and 26 on purpose
    4  demonstrate on the deployed binary through the read-only mode: PASS on the real record;
       UNDETERMINED and FAIL with ZERO_STATE_DIR pointing at a temp copy of the record
    5  tick DOCTOR, ONE WRITER, TWO CONSUMERS; the census for ONE COLLECTOR; then step 4, the
       docs, in Christian's wording
```

### Found, not fixed

```text
    nsh labels an external program's exit 2 as "misuse of shell builtin" -- seen on
    core fingerprint show, whose 2 means UNDETERMINED. For INT-265
    zero_state_dir() is ~/.local/state/0-core, a second state tree beside runtime_dir()'s
    zero -- noticed in step 2 recon. The fingerprint uses runtime_dir()
```

## Success Criteria

- [x] ONE COLLECTOR: zero_core::fingerprint::collect() reads the nine declared inputs, and a
      census shows no other crate reads them for identity
      <!-- evidence: census 2026-10-01, every Rust source under zero/. machine-id, dmi board, cpuinfo and .git/config are read only by zero_core::fingerprint. Other readers display or record and never compare: bootstrap and teach os_name (PRETTY_NAME, not ID), fetch and the nsh prompt (/etc/hostname), zero-update print_system_identity (/etc/hostname, a header), bootstrap get_git_remote (a clone line), snapshot git_remote (stored, never compared). Functional or a different object: zero-sandbox seccomp_filter (consts::ARCH picks the syscall table), zero-sandbox get_memory_kb (VmRSS, not Uid), nsh reload_nsh (dev+ino of the nsh binary, not ~/0-core). Scripts were not in scope: the gate says crate. -->
- [x] RED FIRST: every proof test below is seen failing before the code that makes it pass
      <!-- evidence: step 1 -- 8 of 8 collector tests failed at todo!() before 98d6657b. Step 2 -- core_fingerprint_lifecycle failed on unrecognized subcommand before 63228be9. Step 3a -- core_doctor_check_reads_only failed on unrecognized subcommand check (exit 2) before f7b567f8. Step 3 -- the zero-doctor count tests (probes 27->28, checks 27->28, judging 25->26) failed 3 of 60 and core_doctor_check_fingerprint failed with exit 64 before ecde113a. -->
- [x] MISSING IS NOT ZERO: an unreadable input (injected path) lands in the missing-set and the
      outcome is UNDETERMINED, never PASS
      <!-- evidence: 98d6657b, zero-core fingerprint::tests::an_unreadable_input_is_missing_and_the_outcome_undetermined -- machine-id removed from a fake machine: missing-set names it, digest None, compare UNDETERMINED. Red at todo!() first. -->
- [x] A CHANGED INPUT IS FAIL: one differing input, all others read, gives FAIL naming its axis
      <!-- evidence: 98d6657b, fingerprint::tests::one_changed_input_is_fail_naming_it (hostname -> FAIL [identity.hostname]) and a_link_where_a_real_directory_belongs_is_fail_on_the_tree (-> FAIL [tree.dirs]). Red at todo!() first. -->
- [x] RENAME IS NOT A NEW MACHINE: a display-name change leaves the digest unchanged
      <!-- evidence: 98d6657b, fingerprint::tests::a_field_outside_the_declared_list_does_not_move_the_digest -- a display.name field added: same digest, compare PASS. The digest reads DECLARED only. Red at todo!() first. -->
- [x] TWO CONSUMERS, ONE RECORD: core and the doctor produce identical records in one session
      <!-- evidence: ecde113a, 2026-10-01 -- core fingerprint show, core doctor check fingerprint and the d panel all read digest 09c7905f4daea150 in one session. probes/identity.rs calls zero_core::fingerprint::collect() and compare(), the same functions core calls, and reads nothing itself. -->
- [x] STABLE DIGEST: the digest function is pinned by a test vector in the code
      <!-- evidence: 98d6657b, fingerprint::tests::the_digest_function_is_pinned -- FNV-1a written in zero-core, fnv1a64("project 0") == 0x2501e42b18699b7e. Red at todo!() first. -->
- [x] ONE WRITER: only core fingerprint record writes the expected record; the doctor never does
      <!-- evidence: core half -- nsh-test core_fingerprint_lifecycle (63228be9): show never writes, record is the one writer. Doctor half -- nsh-test core_doctor_check_fingerprint (ecde113a) runs the check with no record and asserts none exists afterwards; probes/identity.rs only reads paths::fingerprint_file(). -->
- [x] DOCTOR: a Fingerprint check shows PASS, FAIL and UNDETERMINED, each demonstrated on the
      deployed binary
      <!-- evidence: ecde113a -- nsh-test core_doctor_check_fingerprint on the deployed core, in a temp ZERO_STATE_DIR: no record UNDETERMINED (exit 2), then record and PASS (exit 0), then a changed hostname FAIL (exit 1) naming identity.hostname. By hand: core doctor check fingerprint PASS, digest 09c7905f4daea150; with an empty ZERO_STATE_DIR, Safe abort, not recorded yet, exit 2. -->
- [x] DOCS TRUE: AGENTS.md (wording approved by Christian) and docs/FINGERPRINT.md name the
      collector's real path and mark each consumer built or planned
      <!-- evidence: 87216662, 2026-10-01 -- docs/FINGERPRINT.md names zero/rust-tools/zero-core/src/fingerprint.rs as the one collector, lists the nine declared inputs and the excluded ones, gives the compare table and exit codes as built, and marks core and the doctor built; integrity, DevBox verify and devshell Law 0 planned. AGENTS.md lines 62, 64 and 69, wording approved by Christian 2026-10-01, name zero_core::fingerprint and the same built/planned split. Sweep: no live file names integrity/mod.rs as the fingerprint home. -->

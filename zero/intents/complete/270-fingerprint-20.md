---
id: 270
date: 2026-10-01
type: future
title: "Fingerprint 2.0"
status: complete
tags: [fingerprint, novashell, shell]
---

## Vision

Fingerprint 2.0. The same machine answers PASS after every boot, a changed question is never
reported as a changed machine, and a FAIL tells the reader what moved, from what, to what.

## The Problem

Measured 2026-10-01, after INT-268 closed:

```text
    d             Fingerprint FAIL -- differs from the record: tree.repo
    record        tree.repo=58:19901 git@github.com:WidkidoneR2/0-Core.git   (written 18:58)
    live          tree.repo 59:19901 git@github.com:WidkidoneR2/0-Core.git
    the number    device of ~/0-core: major 0, minor 59 -- an anonymous device. findmnt:
                  /dev/mapper/root[/@home] btrfs. btrfs assigns each subvolume such a number
                  at mount time
    the boot      uptime -s 2026-10-01 22:37:08 -- the first boot since the record existed
    inode, url    unchanged (19901; the origin URL)
```

Nothing on the machine changed. fingerprint.rs:262 formats meta.dev() into tree.repo, and that
number is handed out again at every mount. INT-269 recorded and compared inside one boot, so every
gate passed. Three gaps let it through:

```text
    1  the input    a per-mount number inside a declared input
    2  no guard     nothing refuses the next field that reaches for a per-mount or per-boot
                    value -- FINGERPRINT.md's boot-id exclusion was prose, not a test
    3  no values    the FAIL named tree.repo but not 58 against 59; three rounds of recon
    and one more    changing what a field means makes every existing record FAIL: the same
                    machine, asked a new question, reported as a different machine
```

## The Solution

```text
    INPUT     tree.repo becomes "<inode> <origin url>". The inode still catches ~/0-core being
              replaced; the URL still catches another repository. No new probe, no dependency
    GUARD     a test reads the collector's own source (include_str!, cut at the test module)
              and refuses device numbers, boot ids, clocks and uptime. Red on :262 today
    SCHEMA    the record carries schema=2, outside DECLARED, so the digest does not move. A
              record from another schema -- or with no schema line, the INT-269 shape -- is
              UNDETERMINED naming schema, with the next move: core fingerprint record. Never
              FAIL. Still tri-state: no fourth outcome
    EXPLAIN   zero_core::fingerprint::explain(live, expected): one line per input that keeps the
              answer from PASS -- "record X, now Y", "could not be read now", or the schema
              sentence. core fingerprint show and the doctor print it; nsh-test keeps asserting
              FAIL and the input name, both still present
    COLOR     show speaks in colour: PASS green, FAIL red, UNDETERMINED yellow; in each explain
              line the input is bold, the recorded value red, the live value green -- what moved
              is visible before it is read
    DOCS      FINGERPRINT.md: the tree.repo row, the exclusions, the schema, the compare table,
              a reboot line under Stability. AGENTS.md is Christian's: wording proposed, not
              edited
```

### Order, red first

```text
    A  the four tests and compiling stubs (SCHEMA, explain todo!). cargo test -p zero-core:
       four red, eight green
    B  the implementation, core show and record (with colour), the doctor probe, FINGERPRINT.md.
       Green
    C  ship; show on the old record answers UNDETERMINED schema 1; Christian runs
       core fingerprint record; show PASS; d 0 failed; nsh-test all passing
    D  reboot; d Fingerprint PASS without re-recording
```

### Not in scope

INT-269's found-not-fixed list (REAL_DIRS beside paths.rs, the hostname sources, exit labels,
zero_state_dir, the friday probe). Each stays where 269 put it.

## Progress 2026-10-01

```text
    A  red        cargo test -p zero-core fingerprint: 8 passed, 4 failed -- each on its own
                  assertion (the guard on .dev() at :356, the repo shape at :368, no schema fact
                  at :378, explain's todo! at :171)
    B  green      25 of 25 zero-core tests; cargo build --workspace clean, no warnings
    C  deployed   ship 0 failed; the INT-269 record answered UNDETERMINED schema 1; Christian ran
                  core fingerprint record -- REPLACED a schema 1 record, digest a56683c54812378b;
                  show PASS; d 0 failed with Fingerprint passing
    C2 colour     nsh-test core_fingerprint_show_colours moved to the 270 contract (it had pinned
                  INT-269's red UNDETERMINED and went red, 211 of 212) and now also proves the
                  explain line; the doctor's Safe abort panel renders amber, Failure stays red.
                  Both approved by Christian. nsh-test 212 of 212
    AGENTS.md     the two Fingerprint sentences Christian approved, lines 66 and 68
```

### Found tonight, not fixed -- each for its own intent

```text
    fsearch       ignores > file: its table reaches the terminal and no file is created
    globs         /usr/bin/cat -n zero/intents/*/268-*.md reached cat with the asterisks intact
    exit labels   nsh calls an external exit 2 "misuse of shell builtin" (core's UNDETERMINED,
                  ls's cannot-access) -- already INT-265's
```

## Success Criteria

- [x] RECON recorded here: the drift measured -- record against live, the device class, the
      filesystem, the boot time
<!-- evidence: demonstrated 2026-10-01. The record file against core fingerprint show (58:19901 against 59:19901); os.stat major 0 minor 59; findmnt /dev/mapper/root[/@home] btrfs; uptime -s 22:37:08. Recorded under The Problem above. -->
- [x] RED FIRST: the four new tests seen failing before the code that makes them pass
<!-- evidence: 2026-10-01, cargo test -p zero-core fingerprint after step A: 8 passed, 4 failed. The guard panicked at fingerprint.rs:356 on .dev(); the repo test at :368 (dev:ino against ino); the schema test at :378 (no schema fact); the explain test at :171 (todo). After step B, 25 of 25. -->
- [x] BOOT-STABLE INPUTS: tree.repo is the inode and the origin URL, and a test scanning the
      collector's source refuses device numbers, boot ids, clocks and uptime
<!-- evidence: fingerprint::tests::no_input_reads_a_number_the_kernel_assigns_per_mount_or_boot and the_repo_is_its_inode_and_origin_url green. Deployed core fingerprint show reads tree.repo 19901 git@github.com:WidkidoneR2/0-Core.git. -->
- [x] SCHEMA: the record carries schema=2; a record from another schema, or none, compares
      UNDETERMINED naming schema with the next move, never FAIL
<!-- evidence: a_record_from_another_schema_is_undetermined_never_fail green. Deployed: the INT-269 record answered UNDETERMINED, not established: schema, with the re-record sentence (exit 2); core fingerprint record printed REPLACED a schema 1 record; this one is schema 2. -->
- [x] FAIL EXPLAINS ITSELF: core fingerprint show and the doctor print each differing input with
      its record and live value
<!-- evidence: a_fail_shows_the_record_and_the_live_value green. Deployed, on a temp copy with tree.repo=58:19901: show printed tree.repo record 58:19901 now 19901 (exit 1); core doctor check fingerprint gave Failure with both values in its Reason. -->
- [x] COLOR: show prints PASS green, FAIL red, UNDETERMINED yellow, and each explain line with
      the input bold, the record red, the live value green -- demonstrated on the deployed core
<!-- evidence: nsh-test core_fingerprint_show_colours on the deployed core, 212 of 212 on 2026-10-01: yellow UNDETERMINED, green PASS, red FAIL, and under the FAIL the input bold, the recorded value red, the live value green, read as escape codes. Christian saw the doctor's Safe abort panel amber and its Failure panel red. -->
- [x] DOCS TRUE: FINGERPRINT.md says what was built; AGENTS.md wording proposed to Christian
<!-- evidence: docs/FINGERPRINT.md, step B: the tree.repo row, the exclusions, the schema paragraph, the compare table row, the reboot line under Stability. AGENTS.md Fingerprint section lines 66 and 68 carry the two sentences Christian approved 2026-10-01. -->
- [x] DEPLOYED: shipped; the old record answers UNDETERMINED schema 1; re-recorded by Christian;
      show PASS; d 0 failed; nsh-test all passing
<!-- evidence: ship 2026-10-01, 0 failed; Christian ran core fingerprint record, digest a56683c54812378b; core fingerprint show PASS; d 0 failed with Fingerprint passing (25 of 28); nsh-test 212 of 212. -->
- [x] SURVIVES A REBOOT: after a reboot, d Fingerprint PASS with no new record
<!-- evidence: the first commands after the reboot that followed 7a90e018. ~/0-core device 59 before, 57 after (major 0 both; inode 19901 both): btrfs handed out a new number, the exact condition that broke INT-269. d Fingerprint PASS and core fingerprint show PASS on digest a56683c54812378b, schema 2, with no core fingerprint record in between. -->

## Relationship

- INT-269 -- built the collector; closed, so this is the forward fix, not a retrofit
- INT-268 -- its commit waits on d 0 failed, which this intent restores

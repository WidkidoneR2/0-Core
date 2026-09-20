---
id: 149
date: 2026-09-20
type: decision
title: "tools.toml has nine readers; this is a "one owner" ruling"
status: decided
tags: [owner, nsh, nova]
---

## The ruling

`faelight-core` owns the PATH to tools.toml. It owns the PARSE too. One function, typed entries,
the flags honoured, an error returned rather than swallowed -- and every reader asks it.

## What was measured, 2026-09-20

NINE readers of the same file, across six domains and two crates:

```text
    engine/domains/audit/mod.rs:45          read_to_string().unwrap_or_default()
    engine/domains/bootstrap/mod.rs:339     line scan, no flags
    engine/domains/deps/mod.rs:363
    engine/domains/doctor/aliases.rs:31     line scan, its own filter
    engine/domains/doctor/mod.rs:117        read_to_string().unwrap_or_default()
    engine/domains/narrative/mod.rs:77      read_to_string().unwrap_or_default()
    engine/domains/snapshot/mod.rs:80       read_to_string().unwrap_or_default()
    faelight-release/src/changelog.rs:544
    faelight-doctor/src/probes/tools.rs:38  serde, flags honoured, error returned
```

★ AND THEY DISAGREE ON SCREEN, AT THE SAME TIME. On 2026-09-20 the doctor reported
`24/24 tools deployed (100%)` while `core bootstrap verify` reported `25 tools not deployed`,
about the same machine and the same file. Both were reading tools.toml. Only one was reading it
correctly: bootstrap takes EVERY name and ignores `deployable` and `retired`, so it reports the
deliberate absence of retired tools as a defect.

⚠️ AND FOUR OF THEM END `unwrap_or_default()`. An unreadable registry becomes an empty string,
which becomes zero tools, which prints as a count. That is the collapse INT-222 spent itself
removing from the doctor -- and it is still sitting in audit, narrative, snapshot and the
doctor's own source listing, which the port did not touch because they list rather than check.

## Why this happened, and it is not carelessness

Nobody decided there should be nine. Each was written by whoever needed the registry that day,
in the file they were already in, and each was the shortest path to an answer. No step in the
process asked WHETHER SOMETHING ALREADY READS THIS FILE.

⭐ THE SAME MECHANISM PRODUCED THE COCKPIT'S EIGHT NAME LISTS AND THE QUICK SCAN'S HARDCODED
SIX, both removed by INT-222. The defect is not the duplicate; it is the absence of a place
where the question gets asked.

## The work this implies -- NOT STARTED

The reader already exists and is correct: `faelight-doctor`'s `read_registry()`, with serde, both
flags, and `Result` rather than a default. The job is to MOVE THAT ONE DOWN into faelight-core
and delete the other eight. It is not a design problem, it is nine call sites.

- [ ] `faelight_core::registry::tools()` exists, typed, flags honoured, returning Result
- [ ] ⭐ AN UNREADABLE REGISTRY IS AN ERROR AT EVERY CALL SITE, never an empty list. Proven by
      moving the file aside and watching all nine report that they could not read it.
- [ ] All nine call sites use it; `grep tools_registry()` finds only the owner
- [ ] `core bootstrap verify` and the doctor agree about how many tools are deployed, and the
      number is checked against `ship` output rather than against each other
- [ ] nsh-test green

## Relationship

Found while finishing INT-253. Filed rather than started: a nine-site refactor does not belong
inside an intent about NixOS strings, and the audit this project already survived was about
work that happened without being recorded.

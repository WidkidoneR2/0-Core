---
id: 240
date: 2026-09-01
type: arch
title: "sixteen sites build state paths by hand instead of asking paths.rs and two of them broke silently today"
status: planned
tags: [architecture, rust, design]
---

## Vision

One place decides where state lives.

## The Problem

paths.rs exists to be the path authority, and sixteen sites route around it with
home().join(...). Measured 2026-09-02 with grep for state/0-core across the tree:

    security      4 sites   last-scan.json, scan-history, first-seen -- NONE EXIST
    intent/focus  4 sites   core, friday-chat and novashell each build it by hand
    sandbox       3 sites   two different crates
    profile       2 sites   current-profile -- DOES NOT EXIST
    fetch         1 site    current-profile again, third constructor
    doctor        1 site    reads security/last-scan.json, which is not there
    friday        1 site    focus.toml, fourth constructor

⚠️ THIS IS NOT HYPOTHETICAL. Two independent bugs on ONE DAY came from exactly
this, and both were silent:

faelight-docs read registry/tools.toml and engine/src/domains, two paths INT-061
moved under faelight/. Both reads failed. Both unwrap_or(0) turned the failure
into a number. faelight-docs sync WROTE "0 custom Rust tools" into the first
paragraph of the repository README and printed a green check. It had been saying
that since the restructure.

faelight_state_dir hardcoded .local/state/0-core while runtime_dir -- in the same
file, twenty lines away -- resolves .local/state/faelight with an existence check
and a comment promising there is deliberately no window where the code points
somewhere the data is not. One was pointed at the new name during the rename and
the careful one was left alone, so live state split across two directories and
the daemon socket sat apart from the database it serves.

⭐ AND THE PATTERN IS WORSE THAN DUPLICATION. A hardcoded path that goes stale
does not fail loudly. It reads nothing, a default fills in, and the tool reports
a confident wrong answer. That is the same shape as the doctor checks INT-222
catalogues and the guard list that failed open: the danger is not the missing
file, it is the number written in its place.

## The Solution

Route every site through paths.rs. The machinery already exists and is good --
runtime_dir checks existence, falls back, and migrates on its own without a
window where code and data disagree. Nothing new needs designing; sixteen call
sites need to ask instead of assume.

Shape, following daemon_socket which was done 2026-09-02 as the first one:

- a function per artifact, returning the FULL path rather than a directory --
  every caller joined the filename immediately, so the directory was an
  abstraction that existed only to be discarded
- derived from runtime_dir, so moving the state directory stays the one-line
  change INT-061 built the seam for
- named for what it returns, not for a brand -- faelight_state_dir said faelight
  and returned 0-core, which is how the split went unnoticed

⚠️ AND THE READS NEED A DECISION SEPARATELY. Several of these paths point at
files that have never existed. Whether a missing security scan is "no findings"
or "never scanned" is not a path question, and routing it through paths.rs will
not answer it. Do not let the mechanical change hide the ones that are lying.

## Success Criteria

- [ ] Watch it fail first: pick one stale path, confirm what its consumer
      reports today when the file is absent, and record whether that answer is
      honest
- [ ] grep for state/0-core across --include=*.rs returns only paths.rs
- [ ] Each artifact has ONE function in paths.rs, and the four focus.toml
      constructors become one
- [ ] Every function derives from runtime_dir rather than home()
- [ ] Any read whose absent-file default was a lie is fixed or filed, not
      silently carried through the move

## Notes

- daemon_socket (01c07fb9) is the first one done and the template for the rest.
  Six sites built that path by hand; they call one function now.
- INT-061 built the seam. This intent is the discovery that sixteen places never
  used it.
- ⚠️ Do not batch all sixteen in one commit. Each subsystem has its own question
  about what a missing file means, and a mechanical sweep would carry those
  answers forward unexamined.

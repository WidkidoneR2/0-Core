---
id: 283
date: 2026-10-08
type: future
title: "faelight gone, registry true -- no faelight outside history, nsh at its real version, the drift check never skips a name"
status: planned
tags: [registry, faelight, cleanup, integrity, rename, finding]
---

## START HERE

Begin at G0, read-only. Nothing is edited until the G1 rulings are recorded in this file.
Measured 2026-10-08 during the INT-282 close (F-0024, e6e2c325).

## Vision

Faelight is history, not inventory. Outside the ledger and git history the word does not
appear in the tree, in the registry, or in anything on this machine that starts a program.
The registry says what is true: every live tool at its real version, every row compared,
and a name the checks cannot resolve is reported, never skipped.

## The Problem

INT-252 (7b79c725, 2026-09-24) moved the source tree from faelight/ to zero/. d reports
Zero Paths "5 directories, one name each -- no old name remains". That speaks for five
directories only. A read-only audit of zero/registry/tools.toml against the tree and
~/.local/bin (2026-10-08, sha256 4031fd3e6268) found:
- 51 rows, 24 retired. 14 are faelight rows: faelight, faelight-diff, -bootstrap, -browser,
  -cleanup, -fm, -glog, -menu, -notify, -term, -clipboard, -wallpaper, -vault, -notifyctl.
  None has a crate in the tree or a binary in ~/.local/bin. They are rows, nothing more.
- nsh is recorded at 3.9.0; the shell is 5.3.1. registry_version_drift
  (integrity/mod.rs:1047) resolves a row to a crate by name and continues on None. nsh is
  neither the package (novashell) nor its directory, so the row is never compared and
  integrity reads 100.
- zero-zone is retired but its crate is still in zero/tools/zero-zone (not deployed).
- nsh-test has no retired field; every other row has one.
- Seven retired rows still say deployable = true: core-diff, faelight-bootstrap,
  faelight-cleanup, faelight-menu, keyscan, latest-update, verify-bootstrap.
Not yet measured: faelight anywhere else in the tree, and outside it.

## The Solution

1. Census first (G0): every faelight occurrence, inside and outside the repo; every reader
   of tools.toml and what it does with retired and deployable; how Zero Paths decides.
2. Rulings (G1) on what history keeps, whether retired rows stay, zero-zone, the nsh row,
   and where the guard lives.
3. A guard that fails while faelight remains outside the exempt paths, and a drift test that
   fails while a row resolves to no crate -- both watched red first.
4. Remove the remnants; correct the registry; make the drift check report an unresolvable
   name as an issue instead of continuing.

## Non-goals

- Rewriting history: complete intents, findings, decisions, the CHANGELOG past entries and
  git history keep the word faelight. (Confirmed or changed in G1.)
- Omarchy files. Anything Omarchy owns is read, never edited.
- The cargo-* rows (third-party, versions like latest); they stay as they are.
- d Rust Docs missing the 15 warnings: a separate finding.
- Renaming any live binary.

## Scope against neighbours

INT-252 stays complete; this finishes what its sweep did not reach. F-0024 is taken forward
here. INT-277 (login shell) is untouched: nothing here changes how nsh starts.

## Dependencies

depends_on is empty on purpose: INT-282 is complete and F-0024 is filed.

## Success Criteria

Each gate is watched failing before it is watched passing. Anchors stay ASCII-only. No sudo.

- [ ] G0 CENSUS, read-only, with file:line: (a) every faelight occurrence in the tree,
      grouped as code, config, scripts, docs, registry, tests, ledger; (b) outside the repo:
      ~/.config (hypr, systemd/user, autostart), ~/.local/bin, ~/.local/share/applications,
      rc files and config.nsh, the user crontab; (c) every reader of registry/tools.toml and
      what it does with retired and deployable; (d) every reference to zero-zone; (e) what
      the Zero Paths check compares.
- [ ] G1 RULINGS recorded with reasons: the history exemption list; retired rows kept as
      tombstones or removed; zero-zone removed or un-retired; the nsh row renamed novashell
      or given a package field; where the faelight guard lives (d Zero Paths, deadwood, or
      nsh-test).
- [ ] G2 RED FIRST: the faelight guard names every remnant on the current tree and fails;
      a drift test fails because a row that resolves to no crate is skipped in silence.
- [ ] G3 FAELIGHT GONE: zero occurrences outside the G1 exemptions; zero faelight rows in
      tools.toml; the guard green.
- [ ] G4 REGISTRY TRUE: the nsh row records 5.3.1 and is compared; an unresolvable row
      raises an integrity issue; nsh-test has retired = false; no retired row is deployable;
      zero-zone matches its ruling. The audit re-run shows no row marked differs.
- [ ] G5 OUTSIDE THE REPO: the G0(b) sweep re-run finds no faelight caller, or each one is
      listed with what was done to it. Omarchy files untouched.
- [ ] G6 NO REGRESSION, on the deployed binaries: d 0 failed with Integrity 100 and Schema
      Validation clean; core integrity run opens no new proposal; ship 0 failed with Path
      Resilience unchanged; nsh-test all passing; alias coverage and tool installation
      unchanged.
- [ ] G7 regression tests kept beside their code: the faelight guard and the drift test,
      each watched failing before its fix landed.
- [ ] G8 each gate carries evidence per INT-158.

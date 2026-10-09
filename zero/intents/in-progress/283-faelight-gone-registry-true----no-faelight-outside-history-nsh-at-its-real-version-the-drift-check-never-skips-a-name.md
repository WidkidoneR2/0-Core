---
id: 283
date: 2026-10-08
type: future
title: "faelight gone, registry true -- no faelight outside history, nsh at its real version, the drift check never skips a name"
status: in-progress
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

## G0 Census (2026-10-08, read-only)

Three read-only passes, payload sha256 prefixes 3b9dbdcde773, 22f9677d3d2f, b6411b67ebe1.
Matching is case-insensitive over tracked and untracked files.

(a) The tree.
- code 0, scripts 0, tests 0, config outside history 0.
- registry: zero/registry/tools.toml, 14 retired rows, 17 lines (the names, the comment at
  :42-47, the faelight-notifyctl description at :290). The guard does not see them: its
  retired-block exemption, nsh-test main.rs:1979-2000.
- the guard's second word in tools.toml: :62 faelight-browser and :263 faelight-vault (both go
  with their rows), :236 zero-release (a tombstone that stays).
- history, already exempt by the guard (nsh-test main.rs:1943-1949): the ledger, 2066 lines in
  237 files; five CHANGELOGs, 559 lines, none written since Project 0 (Christian, 2026-10-08);
  AGENTS.md, 5 lines, each a rule about the name; zero/meta/releases/ 10.3.0 and 10.4.0;
  zero/meta/INCIDENTS.md:12. All 50 file names that carry the word are ledger files.
- built from pieces with concat!, by the convention at zero-doctor probes/files.rs:68-69:
  nsh-test main.rs:1852-1853 and :1920 (the guards' word lists), zero-doctor
  probes/files.rs:90-91 (Zero Paths must name the old directories), zero-core paths.rs:1237,
  engine intent/mod.rs:3896 (cfg(test), the template guard).

(b) Outside the repo. ~/.config/hypr (9 files), ~/.config/systemd/user (9), ~/.config/autostart
(3), ~/.local/share/applications (10), ~/.config/nsh (1), ~/.bashrc, ~/.bash_profile: 0 hits.
Absent: ~/.local/share/systemd/user, ~/.config/uwsm, ~/.bash_login, ~/.profile. crontab is not
installed. ~/.local/bin: core and nsh-test only, 2 each, the concat! strings above; both equal
target/release (core c1e22340e88f, nsh-test 9768606b7b8c).

(c) Readers of tools.toml, twelve line scanners: engine deps/mod.rs:358, doctor/mod.rs:111,
friday/map.rs:16, integrity/mod.rs (241, 1057, 1125, 1262, 1623, 1882, 1926, 2034, 2068),
registry/mod.rs:8 and :232; zero-gate main.rs:384; ship main.rs:267 and :498; zero-deadwood
main.rs:767; zero-docs main.rs:212 and toolgen.rs:132; zero-doctor probes/tools.rs; nsh-test
main.rs:1981. Each that reads retired holds or skips the row, so the seven retired and
deployable rows are contradictions, not live faults. zero-gate main.rs:383 uses the retired
names as a spawn blocklist. Five readers take the row name as the binary on PATH: integrity
:1167, deadwood :788, ship :273 and :405, zero-doctor tools.rs:70.

(d) zero-zone: binary retired, library kept, ruled at INT-247:3515; imported by engine
doctor/aliases.rs:238; its row is retired with deployable = false. It matches its ruling.

(e) Zero Paths, zero-doctor probes/files.rs:56-95, compares five directory pairs outside the
repo. It says nothing about the tree or the registry.

The drift check, integrity/mod.rs:1047-1113, is silent in four places: :1060 registry
unreadable, :1089 no crate for the name, :1091 Cargo.toml unreadable, :1092 no version line.
It scans retired rows too. Registry now: 51 rows, 24 retired, nsh-test without retired, seven
retired and deployable, sha256 66289950b76f (the audit hash moved with 1bec9193 and 88a74e17).
tools.schema.json declares five fields with additionalProperties false; no row conforms, and
d Schema Validation does not parse it (zero-doctor tools.rs:144). A finding, not this intent.

## G1 Rulings (Christian, 2026-10-08)

R1 History is the guard's existing exemption list, unchanged: zero/intents/, CHANGELOG*,
   AGENTS.md, fonts, zero/meta/releases/, zero/meta/INCIDENTS.md. Reasons: ruled 2026-10-01;
   no changelog has been written since Project 0. A name a guard must carry is built from
   pieces (zero-doctor probes/files.rs:68-69).
R2 The 14 faelight rows are removed. The other 10 retired rows stay as tombstones. The guard's
   retired-block exemption is deleted, and zero-release's tombstone description loses the
   guard's second word. Reasons: registry retire writes tombstones by design
   (registry/mod.rs:154) and zero-gate reads them; the faelight rows live on in the ledger and
   git history; without the exemption the registry cannot hide the word.
R3 zero-zone: no change. Reason: it already matches INT-247:3515. Its stray test-zone.rs and
   unused clap (INT-247:2728) are not this intent.
R4 The nsh row keeps its name and gains crate = "novashell". The drift check resolves crate
   when present, else the name. Reasons: five readers take the row name as the binary on
   PATH; data, not a second hardcoded case. The version reaches 5.3.1 through the check's own
   proposal, which proves the row is compared. The core to engine case (integrity :1082) is
   left as it is: one concern.
R5 The faelight guard is nsh-test no_live_retired_name_in_any_tracked_file. Reasons: one
   owner, already run on every push. Zero Paths answers a different question; deadwood gates
   nothing.
R6 The drift check skips only by stated rule -- a retired row, a row whose type is not rust
   -- and reports every other skip as an integrity issue: registry unreadable, no crate,
   Cargo.toml unreadable, no version line. Its test sits beside it in integrity/mod.rs and
   runs under cargo test -p core in pre-push; a real-tree case is red on nsh today.

## Success Criteria

Each gate is watched failing before it is watched passing. Anchors stay ASCII-only. No sudo.

- [x] G0 CENSUS, read-only, with file:line: (a) every faelight occurrence in the tree,
      grouped as code, config, scripts, docs, registry, tests, ledger; (b) outside the repo:
      ~/.config (hypr, systemd/user, autostart), ~/.local/bin, ~/.local/share/applications,
      rc files and config.nsh, the user crontab; (c) every reader of registry/tools.toml and
      what it does with retired and deployable; (d) every reference to zero-zone; (e) what
      the Zero Paths check compares.
<!-- evidence: 2026-10-08, demonstrated: three read-only passes, payloads 3b9dbdcde773, 22f9677d3d2f, b6411b67ebe1; findings in the G0 Census section. -->
- [x] G1 RULINGS recorded with reasons: the history exemption list; retired rows kept as
      tombstones or removed; zero-zone removed or un-retired; the nsh row renamed novashell
      or given a package field; where the faelight guard lives (d Zero Paths, deadwood, or
      nsh-test).
<!-- evidence: 2026-10-08, R1-R6 in the G1 Rulings section, each with reasons, accepted by Christian in session. -->
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

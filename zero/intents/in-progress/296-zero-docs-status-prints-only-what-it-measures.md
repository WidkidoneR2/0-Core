---
id: 296
date: 2026-10-18
type: future
title: "zero-docs status prints only what it measures"
status: in-progress
tags: [docs, novashell, nsh, zero]
---

## Vision
zero-docs status prints only what zero-docs measures on the spot and owns. A line that copies a number
another tool owns -- the release version and codename, d's health, the intent ledger's counts -- is removed,
so status can no longer disagree with its owner. The root README is person-owned: zero-docs never writes it.
What is left of zero-docs says only what it can prove, and the eight subcommands INT-291 left stay.

## The Problem
Measured 2026-10-18 at commit c1bb502a, before any code. Sources are file:line in zero/tools/zero-docs/src/main.rs
unless named.

1. status copies facts it does not own. cmd_status (565-657) prints Version and a codename (meta/VERSION through
   paths::version_file, and the theme from the CHANGELOG line that carries that version), Intents (a scan of the
   intent categories for the text status: complete), Health (a cached copy of d's last number from
   paths::health_status_file) and a README line. status printed Health: 0% while ~/.cache/zero/health-status held 92
   and d reported 92%. meta/VERSION and zero/meta/CHANGELOG.md own the release version and codename (decision 121).
2. The root README patch is dead. cmd_readme patches three numbers: N custom Rust tools, N+ native domains and
   N complete intents. git grep for those three phrases in README.md returns one line, README.md:154, which says
   native domains with no number before it. None of the three patterns exists, so cmd_readme patches nothing and its
   only write is the footer. The constant footer INT-291 G3b wrote says the live numbers above are refreshed by
   zero-docs sync; this README has no live numbers, so the sentence is untrue.
3. A retired name survives in a message. status line 651 tells the reader to run zero-docs readme, a subcommand
   INT-291 G1c retired. The INT-291 sweep covered retired tool names, not hints that name a subcommand.
4. Four answers to one question. Tools: status printed 19; the index says 17 active tools (19 crates read); the
   cicomplete checkpoint printed 18; d reports 17/17 deployed. Each is computed by its own code.
5. The same defect is already on record. INT-265 (complete) noted that zero-docs status shows v1.0.0 -- Morphwood
   and a tool count that disagrees with the index. The note was made and the defect was never closed.
6. Nothing depends on the lines. The only caller of status is the alias docs-status (~/.config/nsh/config.nsh:142).
   the recon found no test, script or engine code in zero/shell, zero/scripts or zero/engine that reads its output.
7. The intent count has a second reason to go. Christian, 2026-10-18: whether completed intents are kept at all is
   still to be decided, so no tool should print or patch a count of them.

## The Solution
Remove what zero-docs copies, keep what it measures, and report what it can verify.
- status keeps Tools, Core domains and Commits (the registry read through zero-core, a directory count, git
  rev-list) and replaces the README line with the line it can prove: whether tools/README.md matches every
  Cargo.toml (check_index), naming the stale tools, with a hint that names a subcommand that exists.
- gather_state loses version, theme, health, intent_complete and intent_planned. LiveState shrinks, and the intent
  scan and the CHANGELOG theme lookup go with them.
- cmd_readme loses its three number patches. If the root README is person-owned (ruling d) cmd_readme goes
  entirely: sync becomes index + links + public, check becomes index + links, and the footer line at the end of
  README.md is removed once, by a plan Christian approves, because the file is his.
- status and the index count tools with one function, with a test that they agree.
- A class test: every zero-docs <word> hint in zero-docs' own messages names a subcommand the dispatch accepts, so a
  retired name cannot survive in a message again.

## Success Criteria
- [x] G0 census, read-only, output kept in ~/.cache/zero: (a) every line status prints against its source and its
      owner; (b) the match count in README.md of each cmd_readme pattern (expected 0 of 3) and the lines the footer
      replaces; (c) the four tool counts with the function behind each and which one answers the tools Project 0 has;
      (d) every zero-docs <word> in the source's messages and whether the word is a subcommand; (e) every reader of
      version, theme, health, intent_complete and intent_planned, file:line; (f) every caller of the status output
      (aliases, scripts, tests, the engine); (g) what cmd_public reads from the root README and whether it needs
      BOUNDARY_MARKER or any field this intent removes; (h) whether docs/public states a version, a date or a count.
<!-- evidence: census 2026-10-18 (a, b, d, e, f, g; ~/.cache/zero/int296-g0.txt) and 2026-10-21 (c, h; ~/.cache/zero/int296-g0ch.txt, int296-g0r.txt, int296-g0s.txt, int296-g2recon.txt). (c) tool counts, five sites: zero-docs status 19 (main.rs:205, registry rows filtered); the index and readme-index 17 (toolgen.rs:278 active_tools, binaries not retired); zero-doctor 17/17 (probes/tools.rs:114, deployable rust tools not retired); the checkpoint 18 (manifest.tool_versions, a snapshot list, a different question); bootstrap counts registry names deployed. Measured live on 2.0.2: status Tools 19, readme-index 17 active tools, 19 crates read. Answer for ruling e: active_tools, 17. (h) cmd_public writes the release version into docs/public/index.md (main.rs:65-72), so ruling a is extended (ruling h); the version and date inside PHILOSOPHY.md are the person text it copies. README.md holds END DYNAMIC SECTION once (line 37) and the footer on line 209. Live 2.0.2 status also printed Version v1.0.0 -- Morphwood and Health 92%. -->
- [x] G1 rulings, written here before any code, each approved by Christian: a. the release version and codename
      (status prints neither); b. intent counts (no tool prints or patches them); c. health (status prints none;
      d owns it); d. the root README (person-owned, cmd_readme retired, footer removed once) or kept; e. the one tool
      count and its definition; f. the lines status keeps and the line that reports the index; g. BOUNDARY_MARKER
      and cmd_public's use of the README.
<!-- evidence: approved by Christian 2026-10-18 19:26 (message: i approved), all seven as written in the section Rulings (G1). Ruling g is closed by the G0g finding; ruling e's tool count is settled by G0c. -->
- [x] G2 red. Cases that fail on 2.0.2, on a stand-in tree: status prints no Version, Intents or Health line;
      status Tools equals the index count; status names a tool whose index line was edited and its hint names an
      existing subcommand; sync leaves a root README that never carried the footer byte-identical (today it appends
      the footer); and a zero-docs unit test that every zero-docs <word> in its own messages is a subcommand (fails
      today on zero-docs readme, main.rs:651). The plan text is saved before apply (AGENTS.md); the seals of the
      red plans are cited in the commit.
<!-- evidence: red plan Seal 0b6cb63419e779e2, applied 2026-10-21 07:44 (plan text ~/.cache/zero/int296-plan-red.txt). Run against the deployed zero-docs 2.0.2 (ZERO_DOCS_BIN) on stand-in trees: the suite ran 252 of 256, the four new nsh-test cases failed and the six older zero-docs cases passed (~/.cache/zero/int296-red-suite.txt). Reasons: zero_docs_status_prints_only_what_it_measures - status prints a Version, an Intents, a Health and a README line; zero_docs_status_tool_count_is_the_index_count - status counts 5 tools, the index counts 4 (a library crate that only the registry lists); zero_docs_status_names_a_stale_tool_with_a_real_hint - status says numbers outdated, run: zero-docs readme, and never names alpha; zero_docs_sync_leaves_a_person_owned_root_readme_alone - sync appended the retired footer to a README that never carried it. Unit: cargo test -p zero-docs hint_tests fails on main.rs:651, zero-docs readme (~/.cache/zero/int296-red-unit.txt). -->
- [x] G3 fix. G2 green. The smallest change per ruling, one concern per change, each tested on the debug build;
      no new crates. clippy -D warnings, fmt --check, cargo doc, the zero-docs tests and the zero-core registry
      tests clean.
<!-- evidence: G3 done on the debug build 2026-10-21, one concern per plan, each plan's text saved under ~/.cache/zero/int296-plan-*.txt. Plans (Seals): retire cmd_readme, sync and check, FOOTER, the help text and the old rootstamp case: d2783ea8f4a64ef1 (~/.cache/zero/int296-g3a-build.txt, int296-g3a-unit.txt: the build was clean, the unit test still failed on the status hint, as expected). status measures only Tools, Core domains, Commits and the Index line, with toolgen::active_tool_count as the one tool count, LiveState, gather_state, regex_replace_first and BOUNDARY_MARKER removed: 34d94351af234e3c (int296-g3b-*.txt: build 0, cargo test -p zero-docs 6 of 6, clippy -D warnings 0, fmt --check 0, nsh-test suite 256 of 256 with the four red cases green). Ruling h: red case plan bd4e20f1ff2d8cb6 (suite 256 of 257, zero_docs_public_index_states_no_version failing: the index stated the release version and changed when only the version changed, int296-g3c-red-suite.txt), fix plan 978c1792117d4b5a (cmd_public no longer reads the version); green: build, unit 6 of 6, clippy -D warnings and fmt --check on zero-docs and nsh-test, cargo doc, cargo test -p zero-core, suite 257 of 257 (int296-g3c-green-*.txt). -->
- [x] G4 live. The deployed zero-docs on the real tree: status prints exactly the lines ruling f keeps and
      nothing it does not measure; README.md changes by the one footer line and nothing else; a second sync leaves
      the tree identical; check exits 0, and names a planted stale tool; docs-status prints the new output.
<!-- evidence: G4 on the deployed zero-docs 2.0.2, real tree, 2026-10-21 08:12 (ship: 2 shipped, 15 unchanged, 0 failed). status exit 0 printed exactly Tools 17, Core domains 54, Commits 5198 and the Index line (matches every Cargo.toml); Tools 17 equals readme-index --dry-run (17 active tools, 19 crates read). README.md differs from HEAD by the one removed footer line and nothing else (git diff -U1 -- README.md). sync #1 exit 0 added only docs/public/index.md to the changed files (ruling h: the index no longer states the version); sync #2 exit 0 left the tree identical (diff sha equal). check exit 0 on the clean tree; with a line planted on core in tools/README.md, check exited 1 and printed: tools/README.md is stale for: core -- run `zero-docs sync`, and status printed Index: out of date: core -- run: zero-docs sync; tools/README.md was restored byte-identical (sha256 verified) and the tree returned to its post-sync state. docs-status and docs-check ran with the new output; docs-check, docs-status, docs-sync and zdocs resolve to zero-docs check, status, sync and zero-docs (config.nsh). Not part of this intent: an uncommitted move of intent 280 (future to in-progress) is in the tree; it stays out of the INT-296 commits. -->
- [x] G5 doors: one ship at the end; the deployed zero-docs tested in a new shell; nsh-test all passing; the
      engine integrity fix still runs (zero-docs sync, integrity/mod.rs:253); the census command still answers
      (zero-docs --version); every alias resolves (docs-check, docs-status, docs-sync, zdocs); d 0 failed;
      core intent validate clean. (Amended 2026-10-21: the bump offer of the close cannot precede the close, because
      cicomplete refuses while a gate is open; its output is cited in the closing commit.)
<!-- evidence: G5 2026-10-21 ~08:15, after one ship: deployed nsh-test 257 / 257; zero-docs --version 2.0.2; integrity/mod.rs:253 runs zero-docs sync (exit 0 twice); zdocs, docs-check, docs-status, docs-sync ran; no caller of a retired subcommand outside or inside the repo; d 26 passed, 2 warnings, 0 failed; core intent validate all 332 valid. -->
- [ ] G6 commits: Intent INT-296 and the final plan's Seal with its note under refs/notes/seals; pushed with the
      pre-push hook passing; main and refs/notes/seals reach GitHub; core intent trace reads every seal intact.
- [ ] G7 each gate carries evidence per INT-158.

## Rulings (G1)
Approved by Christian 2026-10-18, all seven as written. Ruling g is closed by the census and ruling e's number by G0c.
a. Release version and codename: status prints neither. meta/VERSION and the CHANGELOG own them; anyone who wants
   them reads those files. Why: a copy can drift (INT-265 found it drifting) and Christian has said it is not needed.
b. Intent counts: neither status nor any README states them. core intent and ints own the ledger. Why: whether
   completed intents are kept is undecided, and a count of them would be wrong the day that is decided.
c. Health: status prints none. d owns it. Why: status printed 0% from a cached copy while d reported 92%.
d. Root README: person-owned, never written by zero-docs. cmd_readme is retired, with regex_replace_first,
   strip_timestamp, FOOTER and the README line of status; the footer at the end of README.md is removed once by a
   plan Christian approves. Why: all three patterns match nothing (Problem 2), so the function maintains only a
   sentence that is no longer true. Alternative: keep cmd_readme and rewrite the footer to say nothing about numbers.
e. One tool count: status and the index call the same function, and a test holds them equal. The census (G0c)
   decides which of 17, 18 or 19 is the answer to the tools Project 0 has.
f. status keeps Tools, Core domains and Commits, and ends with the index line: matches every Cargo.toml, or stale
   for the named tools with run zero-docs sync.
g. BOUNDARY_MARKER goes with cmd_readme. The census (G0g) found it used only by cmd_readme and the README check in status, not by cmd_public.
h. cmd_public (ruling a, extended; approved by Christian 2026-10-21 07:33, message: yes): docs/public/index.md no longer states the release version; the generated index keeps its title and its contents list. Why: it is the same copy of meta/VERSION that status printed (main.rs:65-72), refreshed only when sync runs, so a version bump leaves it stale (INT-292). The version and date inside PHILOSOPHY.md are the person text, copied and not written, and stay.

## START HERE (next session)
State at the end of 2026-10-18: cistart 296 is done (in-progress). No code has changed. G1 is approved. G0 is
partly done and stays unticked. Open the session with: ints 296.

G0 census findings (output saved to ~/.cache/zero/int296-g0.txt; file:line in zero/tools/zero-docs/src):
- (a) status prints Version and theme (main.rs:573-577), Tools, Intents, Commits, Health (595-598), Core domains
  and a README line (645-655). Sources: version_file (178), CHANGELOG (184), registry::read (205), health_status_file
  (300).
- (b) README.md matches 0 of the 3 patterns. Its last line, 209, is the constant footer.
- (d) one hint names a retired subcommand: main.rs:651, zero-docs readme. Every other hint names a dispatch word
  (sync, readme-index, readme-generate). The header comment at main.rs:1 still says v2.0.0.
- (e) readers of the copied fields: status (573-597), the cmd_readme dry run (431-444), the two patches (395, 628).
- (f) one caller of status: the alias docs-status (~/.config/nsh/config.nsh:142). docs/inventory.md:41 lists it.
- (g) BOUNDARY_MARKER is used by cmd_readme (357-358) and the README check in status (611-612), not by cmd_public.

Open before G0 can be ticked:
- (c) Read the five places that count tools and decide which one answers the tools Project 0 has (ruling e):
  zero-docs main.rs:205; the index at toolgen.rs:290; the checkpoint at engine checkpoint/mod.rs:215 (printed 18 at
  the cistart); zero-doctor probes/tools.rs:124; engine bootstrap/mod.rs:133.
- (h) cmd_public reads the release version itself (main.rs:65). docs/public/index.md has 1 version string and
  PHILOSOPHY.md has 1 version and 1 date. Read cmd_public (main.rs:32-110) to see whether it writes them. If it
  does, ruling a needs an extension and goes back to Christian.
- Count how many times README.md contains the END DYNAMIC SECTION marker (0 or 1).

Then G2 (red cases), G3 (fix), one ship, G4 to G7. Intent 280 stays untouched.

## Non-goals
- No change to meta/VERSION, zero/meta/CHANGELOG.md or the release process (decision 121).
- No decision on whether completed intents are kept. This intent only stops zero-docs counting them.
- No change to d, or to the cicomplete checkpoint, which prints its own Version, Health and Tools. If G0 shows the
  same defect there, it is filed with core intent find and is not fixed here.
- No new crates. No sudo. Omarchy untouched. AGENTS.md is proposed against, never edited.

## Dependencies
INT-291 (complete): the index, check_index, the ownership rule and the eight subcommands. Related: INT-265 (the
first record of this defect), INT-292 (cicomplete bumps leave the index stale; check now catches it), INT-293
(per-tool records).


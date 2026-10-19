---
id: 291
date: 2026-10-17
type: future
title: "zero-docs generates READMEs that nothing keeps true and overwrites hand-maintained ones"
status: complete
tags: [zero-docs, docs, readme, registry, one-owner, tools]
---

## Vision
Every document that states a fact about a tool has one owner, and zero-docs writes only what it
owns. A version is stated by Cargo.toml and by what is derived from it. Prose is written by a person
and is never overwritten. The one generated index is checked by a test. zero-docs keeps the commands
something calls, and a regenerate run on a clean tree changes only what the rulings say it may.

## The Problem
Measured 2026-10-17 while closing INT-290.

- Nothing keeps the generated READMEs true. 13 of the 15 per-tool READMEs marked generated state a
  wrong version (zero-git 4.4.1 against 5.1.3, engine 3.2.9 against 4.4.1, db-browse 1.0.0 against
  2.1.1). Generation runs only when someone runs it, and cicomplete never does (INT-292).
- The generator overwrites files a person owns. `zero-docs readme-generate` wrote 19 per-tool
  READMEs (16 changed, 3 new) and reported 0 skipped. One was novashell/README.md, whose footer says
  HAND-MAINTAINED and which holds text no generator has: why bash keeps the login seat, the
  structured-pipe examples, the dependency list. toolgen.rs:487 documents exactly this danger
  (measured 2026-09-15) and counts skips, but no guard exists (G0a): the counter counts only
  write failures. The file was
  restored from git before any commit.
- zero-docs is itself a reader of tools.toml: toolgen.rs:127-136 line-parses it and main.rs:212 reads
  it again, with a past silent failure recorded at main.rs:200-205 (wrong path, unwrap_or(0)).
  Decision 149 rules that the core owns the path and the parse.
- It writes more than READMEs: the docs/public mirror, the root README dynamic section (lines 1-37
  are hand-maintained, main.rs:4), CHANGELOG files (changelog-tools says Generated while
  toolgen.rs:295 says the curated CHANGELOG.md is hand-written and linked, never copied), and three
  database tables (doc_history, doc_registry, engine_signals).
- It is 1,777 lines and 17 subcommands (version 2.0.1). The shell history holds 237,123 rows from
  2026-03-13 and 52 zero-docs or zdocs commands. By prefix match public appears 13 times,
  readme-index 9, status 2, readme-preview 2. The query overlapped (readme 11 is index 9 plus preview
  2), so G0 re-measures with exact words.
- Callers that must keep working: nsh-test (`readme-index --dry-run`, main.rs:2449), the engine
  integrity fix (`zero-docs sync`, integrity/mod.rs:253), the devbox census (`zero-docs --version`),
  and the aliases docs-check, docs-status, docs-sync and zdocs in ~/.config/nsh/config.nsh.

## The Solution
One owner per fact, ruled in G1 before any code:

- The version of a tool lives in its Cargo.toml. The generated index derives it. No per-tool README
  states it.
- Per-tool README prose belongs to a person. The generator never overwrites an existing per-tool
  README; it may scaffold one that does not exist.
- The tools index is the one generated document, and a test keeps its numbers true.
- Each of the 17 subcommands is kept, retired or merged by ruling, against its callers, with the
  outside-the-repo sweep done before any name goes.
- tools.toml is read through the core (decision 149), not line-parsed here.

This is a shrink and an ownership rule, not a new framework. ZERO//PROTOCOL stays in the idea bank
until INT-293 sizes a per-tool contract.

## Success Criteria
- [x] G0 census, read-only. Evidence is the output, kept in ~/.cache/zero:
      a. toolgen.rs 487-560: what the skip guard tests and why it did not skip novashell/README.md.
      b. `readme-generate --dry-run` and `changelog-tools --dry-run`: every file each would write, and
         which of them a person owns.
      c. Readers of doc_history, doc_registry and engine_signals.
      d. How many readers of tools.toml exist now, and whether zero-core offers the typed parse
         decision 149 asked for.
      e. What `public` and `readme` write, and who reads docs/public.
      f. The sweep outside the repo for callers: Hyprland, .desktop, systemd user units,
         ~/.local/bin scripts, rc files, crontab.
      g. Exact-word subcommand counts from the shell history, in place of the prefix counts.
      h. Whether zero-docs can run against a stand-in tree, so the tests do not touch the real one.
<!-- evidence: demonstrated 2026-10-18, read-only, each command run in the real shell. Kept outputs: ~/.cache/zero/int291-g0b-readme.txt and ~/.cache/zero/int291-g0b-changelog.txt. Everything else is cited by file:line so it can be re-read.
a. cmd_generate (zero/tools/zero-docs/src/toolgen.rs:526-561) has NO skip guard. skipped is incremented only when fs::write fails (line 558). The warning at 487-496 is a comment. This corrects The Problem, which says a guard exists and did not fire.
b. readme-generate --dry-run: 19 per-tool READMEs plus the index. 16 exist (15 carry the generated footer, novashell/README.md is HAND-MAINTAINED); 3 are new (ship, zero-doctor, zero-gate). changelog-tools --dry-run: 19 CHANGELOGs. 5 curated ones exist (engine, novashell, nsh-test, zero-git, zero-sandbox) and cmd_changelog_generate (toolgen.rs:708-732) never checks for one. core is aimed at zero/tools/core, which has no directory. Measured: 14 of 16 per-tool READMEs state a wrong version (novashell 3.9.0 against Cargo 5.3.4); the index states 0 stale versions of 17.
c. doc_history and doc_registry are written only by record_update (zero-docs/src/registry.rs:92), called only from main.rs:683; read at main.rs:705 and main.rs:822. 3 rows each, all written 2026-04-14 18:47-18:49 UTC. engine_signals belongs to the engine (engine/src/domains/engines/mod.rs:17); zero-docs inserts into it at registry.rs:120.
d. 16 files mention tools.toml. zero-docs line-parses it (toolgen.rs:128-224) and reads it again at main.rs:212. zero-core offers a typed parse (zero/tools/zero-core/src/registry.rs Row at 29-45, read() at 77) that lacks category, expected_usage, description and depends_on, which zero-docs needs.
e. cmd_public writes docs/public/*.md and index.md from docs/ and discards write errors (main.rs:80, 87). sync = cmd_readme + verify_links + cmd_public (main.rs:107-112). Code readers of docs/public: nsh-test main.rs:2561 and novashell commands/mod.rs:4522.
f. Callers outside the repo: the aliases docs-check, docs-status, docs-sync and zdocs at ~/.config/nsh/config.nsh:141-143 and 159. None in Hyprland, .desktop, systemd user units, ~/.local/bin scripts or rc files; crontab is not installed. In the repo: integrity/mod.rs:253 runs zero-docs sync; nsh-test main.rs:2449 runs readme-index --dry-run.
g. shell_history, 237378 rows, 2026-03-13 to 2026-10-18. Exact words: public 19, links 18, readme-index 14, command-guide 4 (not a subcommand), status 2, readme-preview 2, readme-generate 2, changelog-tools 2, --version 2, --health 2, docs-sync 1. Never typed: sync, check, readme, preview, record, log, why, diff, readme-tools, changelog-preview.
h. A stand-in tree works when HOME, XDG_STATE_HOME and ZERO_STATE_DB are all set (zero-core/src/paths.rs:24, 155, 289). XDG_STATE_HOME is set to the real path in the login environment, so omitting it would write the real database. -->
- [x] G1 rulings, written here before any code, each approved by Christian:
      a. Who owns a version, and whether a per-tool README states one.
      b. What proves a file is human-owned: the marker, or the file existing.
      c. Which of the 17 subcommands stay, retire or merge, and what happens to their aliases.
      d. The fate of the three database tables.
      e. CHANGELOG policy: generated, curated, or one of each with a boundary.
      f. tools.toml read through the core.
<!-- evidence: the rulings are the section Rulings (G1) below. Approved by christian 2026-10-18 by applying the plan that carries them. The measured basis is the G0 evidence comment above. -->
- [x] G2 red. Cases that fail on 2.0.1, on a stand-in tree: a file carrying the hand-maintained
      marker stays byte-identical after a generate run; the generated index states each tool's
      Cargo.toml version; `readme-index --dry-run` states the count it writes; a per-tool README
      follows G1a. Seals of the red plans are cited in the commit.
<!-- evidence: 2026-10-18. Five cases added to nsh-test (regression), each on a stand-in tree in a temp dir: HOME, XDG_* and ZERO_STATE_DB/ZERO_STATE_DIR are overridden, and a dry run must report exactly 4 per-tool READMEs or the case refuses before any write. ZERO_DOCS_BIN picks the binary; the existing readme_index_count_matches_its_file case honours it too. Run on target/debug/zero-docs 2.0.1: zero_docs_keeps_a_hand_maintained_readme FAIL (overwrote a README marked HAND-MAINTAINED); zero_docs_keeps_an_unmarked_readme FAIL (overwrote a README no marker protected); zero_docs_readme_states_no_version_or_date FAIL (a generated README states a version and a date); zero_docs_check_names_a_stale_index FAIL (check exited 0 and never named the stale tool, so the rule that the index states each Cargo.toml version has no check today). Controls: zero_docs_index_count_matches_stand_in PASS and readme_index_count_matches_its_file PASS, so the count the index states is true on 2.0.1 and that part of the gate is a control, not a red case. Suite 247/251 passed, the 4 failures being exactly the red cases; cargo fmt --check clean; crate_directory_has_one_owner passes (stand-in paths are built with join). Plan Seals: cases 431d85039fb68689, literal and format fix 07e28e7be9516e5d, zero/shell literal 1b83e152e8cf27e6. -->
- [x] G3 fix. G2 green. The smallest change per ruling, one concern per change, each tested on the
      debug build; no new crates.
<!-- evidence: 2026-10-18. Suite 251/251 on the debug build (247/251 at G2 red; the four red cases now pass). Seals: fix 3 31c373c294aa90fa (ownership guard, scaffold, honest counts); fix 4 b1cf2641857bb8ef (check names a stale index, sync refreshes it); fix 5 fb14c365e3be87f8 (17 subcommands to 8, registry.rs deleted, dirs and rusqlite dropped). Earlier G3 steps (Row fields, registry read through the core, core_root) were proven last session. clippy -D warnings, fmt --check, zero-docs tests (5) and zero-core registry tests (55) clean. Real-tree dry run: 15 rewritten once, 3 scaffolded, 1 kept (novashell, whose version badge is removed per G1a). Aliases docs-check, docs-status, docs-sync, zdocs resolve to check, status, sync and the bare tool. G4 wording amended to match G1a. -->
- [x] G3b fix. sync stamps nothing that goes stale: the root README footer states no version and no
      date, and a second `zero-docs sync` leaves the root README byte-identical. Found at G5
      (2026-10-18): the footer read v2.0.0 on a 2.0.1 binary and carried a minute-stamped date
      rewritten on every run, so docs-sync dirtied the tree each time. Red: the new nsh-test case
      fails on the deployed 2.0.1, green on the debug build. After one more ship, one live sync
      rewrites the footer once and a second changes nothing. Its bump is the same patch bump.
<!-- evidence: 2026-10-18. Found by the G5 idempotency test: the first docs-sync after the ship rewrote the root README footer (v2.0.0 to v2.0.1, last sync 2026-09-15 to 2026-10-18 18:33). Red: nsh-test zero_docs_sync_leaves_the_root_readme_stamp_free_and_idempotent fails on the deployed 2.0.1 (the root README states a date and states a version), 251/252. Green on the debug build, 252/252. The first green attempt failed because the stand-in registry had no version and no domains directory, so sync refused to write counts it did not measure; the case now builds a valid stand-in and fails loudly if sync never reaches the README (seal f2c0903c50444603). Seals: fix, test and gate 597f3221a365d0ab; clippy manual_contains in nsh-test repl.rs c6127efc60822d98. After the second ship (zero-docs 2.0.1) the real root footer became the constant line once; a second sync left README.md, zero/tools/README.md and git status byte-identical. nsh-test 252/252 against the deployed zero-docs. -->
- [x] G4 live. The deployed zero-docs on the real tree: a full generate run changes only the files
      the rulings allow, novashell/README.md loses only its version badge (G1a), the index is
      true, and `zero-docs --version` reads the new version.
<!-- evidence: 2026-10-18. Live run of the DEPLOYED zero-docs 2.0.1 on the real tree, after a guard read that the binary carries the ownership guard. First readme-generate: 19 per-tool READMEs, 3 scaffolded (ship, zero-doctor, zero-gate), 15 rewritten once, 1 kept (novashell), 0 failed, index written. Second run: 0 scaffolded, 0 rewritten, 19 kept, 0 failed (idempotent). zero-docs check exits 0 (tools/README.md matches every Cargo.toml); verify-links: all resolve; git status lists only the files the rulings allow; scan of the 19 per-tool READMEs found no version, date or generated footer. nsh-test 251/251 against the deployed zero-docs. cargo doc, clippy -D warnings and fmt --check are clean after the three doc-comment warnings in toolgen.rs were fixed. -->
- [x] G5 doors: the ship at the end of the fixes; the deployed zero-docs tested in a new shell; nsh-test all
      passing (its readme-index case included); the engine integrity fix still runs; the census
      command still answers; every alias resolves or was retired by G1c; d 0 failed; core intent
      validate clean. The close offers zero-docs its bump.
<!-- evidence: 2026-10-18. Two ships, not one: the second followed G3b, which the G5 idempotency test found after the first. nsh-test 252/252 against the deployed zero-docs, readme-index case included. Fresh nsh processes ran zero-docs --version (2.0.1), docs-check, docs-status, docs-sync and zdocs: all exit 0, and the tree was unchanged. The census door is zero-docs --version (G0 and the devbox census): it answers 2.0.1. The integrity fix runs exactly zero-docs sync (integrity/mod.rs:253, unchanged): run as a subprocess it exits 0 and leaves the tree byte-identical. d: 26 passed, 0 failed, 2 warnings (uncommitted changes, update readiness) that clear at the commit; Rust Docs clean; Intent Ledger 331 valid. cargo doc 0 warnings; clippy -D warnings and fmt --check clean for zero-docs and nsh-test (a manual_contains lint in nsh-test repl.rs was fixed). G5 wording amended to match the two ships. -->
- [x] G6 commits: Intent INT-291 and the last plan's Seal with its note under refs/notes/seals;
      the earlier plans' Seals are cited in the evidence comments above, and their text was not
      saved before apply, so no note can be rebuilt for them (AGENTS.md: save p.text() first).
      Pushed with the pre-push hook passing; main and refs/notes/seals reach GitHub; core intent
      trace reads every seal intact.
<!-- evidence: 2026-10-18. Commit 2d1b0a88 on main carries Intent: INT-291 and Seal: 42b36ff66300de81, the seal of the G6 wording plan, whose text was saved before apply. The note under refs/notes/seals was attached byte for byte and hashes to the Seal. Pushed with the pre-commit and pre-push hooks passing; main and refs/notes/seals on GitHub equal local (fa2db19c..2d1b0a88 and 69e8cb94..ccaa3f9c). core intent trace 2d1b0a88 and core intent trace 42b36ff66300de81 both exit 0 and read: seal 42b36ff66300de81 intact -- the plan text kept in refs/notes/seals hashes to it. The earlier plans of this intent have no note, because their text was not saved before apply; their Seals are cited in the evidence comments above and trace cannot re-check them. Intent 280 move is not part of the commit and stays uncommitted. -->
- [x] G7 each gate carries evidence per INT-158.
<!-- evidence: 2026-10-18. Read before this plan was applied: a script listed every gate of this file, and G0, G1, G2, G3, G3b, G4 and G5 each carry an evidence comment; G6 and G7 get theirs in this plan. -->

## Rulings (G1)

a. VERSION. Cargo.toml owns a tool's version. No per-tool README states a version, a date, a status or a category. The generated index derives them, and a test on the real tree checks the index against Cargo.toml. novashell/README.md loses its version badge and nothing else. Why: 14 of 16 per-tool READMEs state a wrong version (G0b), and nothing checks a generated file after the command that wrote it.

b. OWNERSHIP. A file is generated only when its last non-empty line is the exact generated footer and no line says HAND-MAINTAINED. Only a generated file may be overwritten. Any other existing file stays byte-identical and is counted as skipped. A missing file is scaffolded: title, Cargo.toml description, Build and Deploy, and no footer claim. The 15 files that carry the footer are rewritten once to the scaffold form and are person-owned from then on; ship, zero-doctor and zero-gate gain a scaffold. Why: no guard exists today (G0a). Positive proof of generation fails safe where a human marker fails open, and novashell/README.md wraps the generated sentence across two lines, so a substring test is unsafe.

c. SUBCOMMANDS. Keep 8: sync, public, check, verify-links, status, readme-preview, readme-index, readme-generate. Retire 9: readme and preview (sync and check already run them), readme-tools, changelog-tools with its alias changelog-generate, changelog-preview, record, log, why, diff. The aliases docs-check, docs-status, docs-sync and zdocs still resolve; none is retired. Why: the callers are integrity/mod.rs:253 (sync), nsh-test main.rs:2449 (readme-index) and the four aliases. The nine retired were typed 0 times in 237378 history rows, except changelog-tools (2, both today); the 3 record rows from 2026-04-14 are not in that history (G0g, G0c).

d. TABLES. doc_history and doc_registry stay in state.db untouched: dropping a table is a persistent-data change that AGENTS.md section 0 says to ask about, so it is a separate intent, filed with core intent find. zero-docs stops creating and writing them. Every use of REGISTRY in zero-docs main.rs is inside why (752, 790) or diff (814), so registry.rs is deleted in G3 once zero-deadwood shows no caller remains. engine_signals stays the engine's; the doc_synced signal ends with record, and nothing reads it (the only hit in the repo is the writer, registry.rs:121).

e. CHANGELOG. Curated only: a tool's CHANGELOG.md is written by a person. changelog-tools, changelog-generate and changelog-preview retire. The README keeps linking a curated CHANGELOG.md and saying so when there is none (toolgen.rs:295-310, unchanged). Why: the generator would replace 4 curated files, create 14 the README policy says should not exist, and aims core at a directory that does not exist (G0b).

f. TOOLS.TOML. zero-docs reads it through zero-core, in this order: first Row in zero-core gains category, expected_usage, description and depends_on as optional fields (its own change, tested in zero-core); then zero-docs replaces parse_registry (toolgen.rs:128-224) and the count at main.rs:212 with registry::read(); its three private core_root() copies (main.rs:167, registry.rs:37, toolgen.rs:8) become zero_core::paths::core_dir(). No new crates. Why: decision 149, and the line parser reads depends_on from one line only.

## Non-goals
- No new crates. No sudo. Omarchy untouched.
- AGENTS.md is not edited here; anything it needs is proposed to Christian.
- No per-tool contract or manifest design: INT-293 owns that, and ZERO//PROTOCOL stays an idea.
- No change to cicomplete: INT-292 owns the version-bump side.
- No change to the intent ledger: INT-294 and 211/212 own that.
- novashell/README.md keeps its hand-written text; only what G1a rules about a version line changes.

## Dependencies
depends_on: none. Informs 292 (the cure may be that documents stop stating a version) and 293
(the README-owner rule). Decisions 149 (tools.toml has one owner) and 142 (store facts, derive
conclusions) bear on G1.

## Versions
- core 4.4.1 -> 4.4.2 (patch)
- nsh 5.3.4 -> 5.3.5 (patch)
- nsh-test 2.0.6 -> 2.0.7 (patch)
- db-browse 2.1.1 -> 2.1.2 (patch)
- friday-chat 2.1.2 -> 2.1.3 (patch)
- ship 1.0.1 -> 1.0.2 (patch)
- teach 4.0.1 -> 4.0.2 (patch)
- zero-ade 1.0.1 -> 1.0.2 (patch)
- zero-context 1.0.1 -> 1.0.2 (patch)
- zero-core 1.0.1 -> 1.0.2 (patch)
- zero-daemon 4.0.1 -> 4.0.2 (patch)
- zero-deadwood 1.0.1 -> 1.0.2 (patch)
- zero-docs 2.0.1 -> 2.0.2 (patch)
- zero-doctor 0.1.1 -> 0.1.2 (patch)
- zero-gate 0.1.0 -> 0.1.1 (patch)
- zero-gen 1.0.0 -> 1.0.1 (patch)
- zero-git 5.1.3 -> 5.1.4 (patch)
- zero-insightd 0.1.0 -> 0.1.1 (patch)
- zero-sandbox 3.0.1 -> 3.0.2 (patch)

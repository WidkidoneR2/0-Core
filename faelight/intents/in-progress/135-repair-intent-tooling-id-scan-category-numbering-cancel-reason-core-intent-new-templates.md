---
id: 135
date: 2026-07-09
type: future
title: "Repair and consolidate the intent ledger tooling"
status: in-progress
tags: [ledger, intent, tooling, bugfix, consolidation]
---

## Vision
One implementation owns the intent ledger. It counts correctly, it validates itself, and
`doctor` runs that validation every deploy -- so the ledger's integrity is demonstrated
continuously instead of audited by hand.

## The Problem
The ledger has TWO implementations that disagree about what a ledger is.

- `core intent` (engine, `domains/intent/mod.rs`): owns the workflow and analytics --
  focus, start, complete, defer, cancel, override, deps, graph, velocity, predict, story,
  next, brief. Eleven aliases point at it (cistart, cicomplete, cif, cid, cin, cis, ciu,
  civ, cibd, cideps, cibr). Wired into checkpoints and doctor.
- `intent` (`rust-tools/intent`): owns `add` -- the ONLY working way to create an intent --
  plus `timeline`, plus duplicates of list/show/search/stats/validate.

Every rule must be written twice, and the copies have drifted into contradiction:

1. **The two validators enforce opposite theories.** `intent validate` reports 7 issues
   (malformed frontmatter, missing fields) and no ID problems. `core intent validate`
   reports 20 duplicate IDs and no frontmatter problems. Per `decisions/137`, most of those
   20 are FALSE POSITIVES -- record dirs number themselves, so `decisions/001` and
   `incidents/001` are not a collision. But `decisions/002` appears twice and `105` appears
   twice: those are REAL, and the current output buries them in noise.
2. **The ID scans differ.** Engine `active_folders` includes `"decisions"` (contradicting
   137). `rust-tools/intent` excludes record dirs but computes `next_id` from the INTENT
   dirs and then writes into whichever category the user chose -- which created
   `decisions/135` on top of `decisions/135-rio-terminal` on 2026-07-09.
3. **`core intent new` is broken** -- the `templates/` dir was lost in the INT-061 move.
   So creation lives only in the crate that does not own the ledger.
4. **`intent cancel <id> --reason "x"`** stores the literal string `--reason` as the reason.
5. **`docs/POLICIES.md:377`** documents `intent add incident "..."` -- a form the wizard no
   longer supports. The docs describe a tool that drifted out from under them.

Nothing catches any of this until a human notices. `decisions/002` has been duplicated for
months.

## The Solution
Consolidate onto `core intent`, then make the ledger self-checking. The order is forced:
creation must work in the engine BEFORE the tool that owns creation can be retired.

## Success Criteria
- [x] **Gate 1 -- DONE 2026-07-09, gen 327.** Both sites patched (:1283, :2363).
      PROVEN behaviorally: `core intent new future` offered **136**, not 139 (max would be
      decisions/138 if decisions/ were still scanned). Not provable alone -- `active_folders`
      has no caller but new_intent/new_intent_smart, both reached via `core intent new`.
      ORIGINAL: Remove `"decisions"` from
      `active_folders` at `intent/mod.rs:1281` and `:2359`. Both sites; fixing one leaves the
      other drifted (the same failure mode as the `intents/future` path bug). Deploy; prove.
      MUST precede Gate 2 -- a repaired `core intent new` on the old scan hands out 138s.
- [x] **Gate 2 -- DONE 2026-07-09. It already worked.** The `templates/` cause was
      FABRICATED: `grep -rn templates intent/mod.rs` -> no match; `git log --diff-filter=D`
      -> no such dir ever existed. `core intent new` takes the template NAME as an arg and
      generates the stub inline. What broke it was the stale path, fixed 2026-07-08 (ff2b9610).
      The false cause propagated to the parking note, ff2b9610's message, INT-115, and this
      charter. Corrected everywhere but the commit message.

- [x] **Gate 2b -- `core intent new` wrote invalid YAML.** DONE 2026-07-09, gen 328.
      `new_intent` (line 1319) opens `format!(r#"---` -- a RAW string -- then wrote
      `title: \"{}\"`, so a literal backslash-quote landed in every file it created.
      `new_intent_smart` (2395) opens `format!("---` -- a NORMAL string -- where the same
      escape is CORRECT. Same text, opposite correctness, because the enclosing literal
      differs. Patched both on the assumption they matched; the compiler caught it
      (`expected ',' found '{'` at 2400). Reverted 2400, kept 1324.
      PROVEN: created an intent, `xxd` line 5 -> `title: "..."` (0x22, no 0x5c).
      `intent validate` had reported this class correctly for months; `core intent validate`
      never checked frontmatter, so the true signal was buried under its 20 false
      duplicate-ID reports. Still to repair: `complete/064-faelight-logout`.

- [x] **Workflow finding.** `cargo build -p faelight-core` is a NO-OP on an unrelated crate.
      The engine is `-p core`. Three confusable names: `faelight-forest` (flake output),
      `core` (engine crate), `faelight-core` (a rust-tool). Cargo suggests the wrong one on
      a typo. A green `Finished` can mean nothing was built. Only `dep` compiles what runs.
- [x] **Gate 3 -- DONE 2026-07-09, gen 329.** `get_next_id()` took no argument and always
      scanned the intent lifecycle dirs; `main()` then wrote into the chosen category. Now
      `get_next_id(category)`: intent dirs (future/in-progress/complete) share one counter
      because a file MOVES between them; each record dir owns its own sequence.
      PROVEN on the deployed binary, both namespaces in one pass: `intent add` -> decisions
      gave **139** (next after 138-nixos); -> future gave **136** (next after 135).
      Also confirmed: the standalone's raw string writes `title: "{}"` unescaped, which is
      why its files were always valid YAML. Gate 2b's damage really was limited to `064`.
      NOTE: `intent add` accepted `status: complete` for a decision, though real decisions use
      `status: decided` + `verdict:`. It validates nothing it writes. -> Gate 5.
      ORIGINAL: `rust-tools/intent`
      computes the id from the intent dirs regardless of chosen category. Scan the directory
      the file will land in. Demonstrated by creating a decision and confirming it takes the
      next free DECISION number, not the next intent number.
- [x] **Gate 4 -- NOT A BUG. Corrected 2026-07-10.** `rust-tools/intent/src/main.rs:108-111`
      takes the reason POSITIONALLY: `let reason = args.get(3)`. Its own usage text (line 113)
      says `intent cancel <id> [reason]`, example `intent cancel 036 "no longer needed"`.
      Claude invoked it with `core intent`'s clap signature (`--reason "x" <id>`), so it
      faithfully stored the literal string `--reason`. The tool did what it documents.
      No fix needed; the charter's item 4 was a fabricated defect, like the `templates/` dir.
      THE REAL DEFECT, which stands: two commands, one verb, incompatible interfaces --
      `core intent cancel --reason "x" <id>` vs `intent cancel <id> "x"`. That is the
      consolidation problem (Gate 6), not a cancel bug.
- [x] **Gate 5 -- DONE 2026-07-10, gen 330.** Rewrote `core intent validate` (was 24 lines:
      one flat HashMap over nine folders -> 21 duplicates, 1 real). Now two-pass:
      Pass 1 semantic checks on parsed intents; Pass 2 raw-byte frontmatter check, because
      `parse_intent()` returns Option and load_all SILENTLY DROPS malformed files -- which is
      precisely why `complete/105` hid for months.
      Ported from `rust-tools/intent` (the crate Gate 6 retires -- it held the better code):
      four required fields, seven-state status vocabulary, README/`type: index` exemption,
      `starts_with("---")`. Added three: id matches filename prefix; status matches directory
      (LIFECYCLE dirs only -- record dirs legitimately carry several statuses); duplicate ids
      WITHIN a namespace per decisions/137.
      FOUND 6 REAL DEFECTS, zero false positives, all repaired:
        - `philosophy/002` carried `id: "philosophy-002"` (quoted, prefixed; 001 says `id: 001`)
        - `incidents/112` carried `id: 111` -- filename and frontmatter disagreed
        - three date-named incidents (`2026-02-03-...`, `2025-...`) carried ids 007/008/009 but
          were UNREACHABLE by `intent NNN`, which resolves on filename prefix. Renamed.
        - `decisions/002` duplicated. `002-faelight-bar` had `type: future` -- a misfiled
          INTENT from the Hyprland era, not a decision. Renumbered to `decisions/139`.
      All 165 intents valid. `decisions/001` vs `philosophy/001` vs `incidents/001` correctly
      produce NO complaint -- separate namespaces, enforced.
      NOTE for Gate 6: the two validators report different counts (165 vs 160). `intent
      validate` walks eight folders and omits `in-progress` entirely -- it has never validated
      an in-progress intent. Same disease: two walkers, two folder lists, silently different.
      ORIGINAL: `core intent validate` learns
      per-namespace uniqueness (not global) AND the frontmatter checks currently only in
      `intent validate`: `id:` present and matching the filename, `title:`/`status:`/`date:`
      present, frontmatter parses, status consistent with the directory. Its 20 false
      duplicates drop to the real ones. Fix those: `decisions/002` (twice) and `105` (twice,
      one with malformed frontmatter, likely INT-061 damage). Also `cancelled/058`.
- [ ] **Gate 6 -- retire `rust-tools/intent`.** Move the `add` wizard and `timeline` into
      `core intent`. Repoint aliases `intl`, `ints`, `int-active`. Update `docs/POLICIES.md:377`.
      Remove the crate from the flake and `registry/tools.toml`. `fsh-test` and the cheatsheet
      must still pass. Nothing may reference the retired binary.
- [x] **Gate 7 -- DONE 2026-07-10, gen 332. The ledger checks itself.**
      `check_intents` was decoration: hardcoded `Status::Pass` (it could never fail), a THIRD
      divergent folder list -- no `in-progress`, plus a phantom `active/` that has never
      existed -- and `content.contains("status: complete")`, a substring match over whole
      files, so any charter quoting that string counted as complete. Doctor said 161; the CLI
      said 165. Nobody knew what doctor was counting.
      Extracted `validate_issues() -> (usize, Vec<String>)`, context-free, reading
      `faelight_core::paths::intents_dir()` -- the same source doctor already used.
      `validate()` prints it. `check_intents()` reports it. ONE rule, ONE implementation,
      TWO consumers. They cannot disagree.
      Count corrected 165 -> 164: the old `load_all` counted `incidents/00-INDEX.md` as an
      intent. It is `type: index`. 164 is right.
      PROVEN on the deployed binary: doctor read `164 intents, all valid`, matching the CLI.
      Seeded `decisions/121-seeded-duplicate.md` -> check went WARN, named
      `Duplicate id 121 within namespace 'decisions'`, health 90 -> 87, `health_drop` fired.
      Removed it -> Pass, 164 valid.
      KNOWN DEBT: `validate_issues` hand-rolls frontmatter extraction rather than reusing
      `parse_intent`. Its `field()` closure splits on the first `:`, so a title containing a
      colon is truncated. Harmless -- only `id` and `status` are compared, never `title` --
      but it is a THIRD parser in a codebase whose disease is duplicate implementations.
      Fix when Gate 6 lands: walk the dir for the byte-0 `---` check, call `parse_intent`
      for everything else.
      ORIGINAL: This is the point of
      the whole intent: a collision like `decisions/135` is caught the moment it is made, not
      months later. Green when the ledger is sound; warns with the specific file when not.
      Seeded-failure test: introduce a duplicate, confirm doctor catches it, remove it.

## Notes
- Root cause of every bug here is duplication: one rule, two implementations, diverging.
  That is INT-115's disease, and this is its sharpest instance -- the two copies grew
  contradictory COMMENTS (INT-070 vs INT-077), each swearing it was correct.
- `decisions/137` is the governing ruling: intent dirs and record dirs are separate namespaces.
- Realistically two sessions. Gates 1-4 are the repair; 5-7 are the consolidation. Do not
  start Gate 6 until Gate 2 is proven -- retiring the only working creation path first would
  leave no way to file an intent.
- Never bulk-tick. Each gate: fix -> `cargo build` -> DEPLOY -> prove on the deployed binary
  -> commit. A cargo build is not a deploy; that trap has bitten twice this week.

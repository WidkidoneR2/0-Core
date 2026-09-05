---
id: 230
title: "fsh cannot be installed without 0-Core, because the shell and its integration are the same code"
status: in-progress
type: architecture
priority: high
date: 2026-08-23
tags: [fsh, architecture, portability, boundary, int-227]
---

## Vision
fsh owns shell behaviour. 0-Core integration is an optional adapter.

## ✅ RECON FIRST, and it reframed the intent entirely
Measured 2026-08-23. fsh has TWO path dependencies -- `faelight-core` and `faelight-git` -- and is a
workspace member via the `faelight/rust-tools/*` glob.

⭐⭐ **`faelight_core` IS USED AT 68 SITES, AND ALMOST EVERY ONE IS `paths::`:**

    paths::intents_dir()      paths::rust_tools_dir()   paths::state_db()
    paths::registry_dir()     paths::read_health()      paths::core_root_string()
    paths::changelog_file()   paths::runtime_dir()

Across `commands/mod.rs`, `prompt.rs`, `digest.rs`, `session.rs`, `exec.rs`, `main.rs`, and three
TUIs.

★ **SO THE DEPENDENCY IS NOT A LIBRARY -- IT IS 0-CORE'S DIRECTORY LAYOUT.** The prompt shows the
next intent. `digest` reads the ledger. `health_tui` counts active intents. `nl` reads the registry.

⚠️ **WHICH MEANS THE SHELL IS NOT MERELY LOCATED INSIDE 0-CORE, IT IS FUNCTIONALLY PART OF IT.** The
question is not "how do we move a directory" -- it is **what is fsh without 0-Core?**

## THE RULING (2026-08-23)
**STANDALONE fsh MUST WORK WITHOUT 0-CORE. 0-Core integration MAY make fsh richer, but must not
define whether fsh is a viable shell.**

⚠️ TWO ALTERNATIVES WERE CONSIDERED AND REJECTED, with reasons:
- **VENDOR `paths` into fsh** -- rejected. It creates TWO AUTHORITIES over one layout, and they
  drift. This ledger has removed that shape repeatedly: two alias expanders, three selector call
  sites, five observability instruments.
- **A COMPILE-TIME FEATURE FLAG** -- rejected. It PRESERVES the coupling while making the build
  matrix worse, and sprinkles 0-Core awareness through the shell as `cfg`. ⭐ A RUNTIME INTEGRATION
  ADAPTER is architecturally cleaner, and nothing yet justifies compiling two different products.

★ AND THE FRAMING IS PART OF THE RULING. "Extract fsh from 0-Core" is an implementation
prescription. The boundary is the intent:

    fsh
     |-- shell / runtime / history / completion / prompt / jobs / diagnostics
     |
     `-- integration
          `-- faelight-core / 0-Core

The 68 `paths::` calls are EVIDENCE FOR WHERE THAT ADAPTER MUST EMERGE, not a task list.

## ⚠️ DO NOT SOLVE 68 SITES IN ONE PASS
Classify them by CAPABILITY first, because the classification is the finding:

    core shell state          what fsh genuinely needs to operate at all
    0-Core discovery          intents, tools, registry
    0-Core observability      health, digest, changelog
    0-Core UI enrichment      prompt and TUIs showing ecosystem state
    0-Core execution          exec.rs and related behaviour

⭐ THE EXPECTATION, stated so it can be WRONG: surprisingly little of the 68 will turn out to be
fundamental to being a shell. Every census this week shrank on reading -- 42 portability sites
became 2 real defects, 23 capability sites became 5 questions, 14 service probes became 4.

## ⚠️ THIS DOES NOT GATE THE VM WORK, and that is a deliberate sequencing decision
VM validation proceeds against the CURRENT MONOREPO LAYOUT. A VM clones 0-Core whole and builds fsh
in place.

★ THE REASON IS DIAGNOSTIC, NOT CONVENIENCE. Doing both at once combines two experiments --
"does fsh work in a VM?" and "can fsh survive without 0-Core?" -- and if the VM breaks, nothing says
which boundary failed. Prove the shell runs as part of the existing checkout FIRST. Standalone
ownership is a separate architectural migration.

## ⭐ WHY THIS MATTERS: PACKAGING IS DOWNSTREAM OF THIS BOUNDARY (his plan, 2026-08-23)
He arrived at the destination independently and it sharpens the intent: fsh should be installed the
way bash is -- a versioned package, core files under system paths, user customisation confined to
`~/.config/`. His words: *"treat it like real software from the start: package it, version it, own
the core, and give users clean extension points instead of the source itself."*

    void   ->  xbps template, xbps-install fsh
    arch   ->  PKGBUILD / AUR, or a small personal repo
    both   ->  /usr/bin/fsh + /usr/share/fsh, root-owned; user edits ~/.config/fsh only
    always ->  tagged releases (v0.1.0), never a living checkout of main

⭐⭐ AND MOST OF IT IS ALREADY TRUE, which is worth recording so nobody rebuilds it:
- **The user side EXISTS.** `config.rs:81` reads `~/.config/faelight-shell/config.fsh`, and there is
  already `~/.config/faelight-shell/plugins/` (`mod.rs:7014`) and `.../scripts/` (`mod.rs:16079`).
  Config, plugins and scripts -- all three extension points, in the XDG location, today.
- **The lock is STRONGER than the plan assumes.** fsh is a Rust binary, and on NixOS the deployed
  config is a home-manager symlink into `/nix/store`, which `config.rs:67` already records as
  READ-ONLY. Not root-owned-and-editable -- immutable.
- **Versioning EXISTS.** Semver, `cicomplete` prompting for bumps, `bump-versions`, and INT-102 owns
  the tool-version-versus-forest-release architecture. fsh is at 3.8.4.

⚠️⚠️ **SO THE ONLY THING BLOCKING HIS PLAN IS THIS INTENT.** A PKGBUILD that installs
`/usr/bin/fsh` produces a binary that expects `~/0-core/faelight/intents/` to exist, because 68
sites read 0-Core's layout. **You cannot package what cannot install alone.** Packaging is not a
parallel workstream -- it is the thing this boundary unlocks.

★ AND THE SOCIAL HALF IS RIGHT AND COSTS NOTHING TO ADOPT EARLY: clear ownership, an official
install method, versioned releases, and the expectation that users report bugs rather than editing
an installed copy. That part is documentation, and it can be written the day the boundary lands.

## Success Criteria
- [x] G1 THE 68 SITES ARE CLASSIFIED BY CAPABILITY, as a committed artifact produced mechanically so
      it can be re-run and diffed. The classification decides the scope; it is not decoration
<!-- evidence: 9e726f92. census-core-coupling.py + faelight/rust-tools/novashell/CORE-COUPLING.md.
     82 classified paths:: calls across 14 functions -- core shell state 31, 0-Core discovery 40,
     observability 8, execution 3. The unit is stated three ways because faelight_core lines (80),
     classified calls (82) and bare use statements (1) are different numbers and were being confused.
     THE PREDICTION IN THIS INTENT HELD, and the reverse of what the raw count suggested: 38% of the
     coupling needs no adapter at all, because the XDG move and FAELIGHT_STATE_DB already resolved it.
     Re-runnable and diff-gated: exits 2 on an unclassified function, so a new coupling site cannot
     land silently. It did exactly that on its first run -- bin_dir was dropped between the histogram
     and the classification table and the script refused to pass. It has since corrected Claude four
     times: bin_dir, a miscounted six-vs-four migration, and two arithmetic slips. -->
- [x] G2 THE ADAPTER BOUNDARY IS NAMED: one place where fsh asks about 0-Core, and everything else
      reads the answer. No second authority over the layout -- `paths` is not copied
<!-- evidence: fdc174a9, 3c4d7ea6, e6b290e1, 540ff38a, a054f3a2, c061cac6, 84d1b696, 7cf40096.
     THE BOUNDARY IS novashell/src/core_integration.rs. It CALLS faelight_core::paths and never
     copies it, so paths stays the single authority over WHERE and the adapter owns WHETHER.
     Accessors: present, ledger, focus, intents_root, tools_root, tool_manifest, registry_root,
     forest_version, release_name, health.
     MEASURED BY THE CENSUS: 51 unmigrated 0-Core calls at the start, 13 at the end, and none of
     the 13 is work the boundary should own -- 4 are the adapter's own calls, 4 are observability
     inside it, 2 are exec.rs's catastrophic-rm protected paths (SAFETY: an Option returning None
     would silently disarm a protection), 1 is main.rs:3126 which STAYS BY RULING (its comment
     records the 43-vs-40 category scoping kept visible as INT-211's finding), 1 is db.rs:68
     core_root_string which is how the shell finds its OWN database and is core shell state
     misclassified, and 1 is cheatsheet_tui.rs:265, which cannot be fixed by an adapter at all.
     daemon_socket was reclassified to core shell state on measurement: it derives from
     runtime_dir(), XDG since 2026-08-21, and its three callers were already correct.
     ⚠️ THE CENSUS NOW STATES ITS OWN LIMITS in its generated output, because it corrected this
     work five times and will be trusted for that reason: it matches text not syntax, it sees only
     paths:: calls (THIRTY hand-built 0-core paths in this shell are invisible to it, twelve
     pointing at a scripts directory that does not exist -- INT-240), it classifies by function so
     one function asked two questions cannot be split, and it measures path coupling rather than
     the defect class. -->
- [x] G3 A 0-CORE-ABSENT fsh STARTS, ACCEPTS INPUT, AND RUNS A COMMAND. Demonstrated, not argued
<!-- evidence: demonstrated 2026-09-04 on the deployed binary.
     mkdir -p /tmp/g3home; HOME=/tmp/g3home nsh -c "echo SHELL_ALIVE" -> SHELL_ALIVE, exit 0.
     HOME=/tmp/g3home nsh -c "pwd" -> the real launch directory, NOT a phantom -- the startup-cd
     fix from 2026-08-21 is holding. /usr/bin/ls -la /tmp/g3home -> EMPTY: nsh -c manufactures no
     state at all under a fresh HOME, so the shell cannot build a forest on a machine that has none. -->
- [x] G4 EACH 0-CORE FEATURE DEGRADES VISIBLY WHEN 0-CORE IS ABSENT, per INT-227's invariant: an
      unavailable capability must never become a successful-looking empty result. A prompt with no
      intent to show says nothing, it does not show a wrong one
<!-- evidence: 2026-09-05. All 36 consumers of core_integration classified by reading each
     call site plus six lines of context: 21 already correct (11 refusals in the find/fsearch/
     --intent/dev arms, four forest_version and two health sites saying unknown, @rust/@intents
     expanding to themselves, prompt.rs:459 and main.rs:3421 saying nothing on None, the
     recorded zero at mod.rs count_md); nine render nothing on None, which is the gate wording
     (digest, session, nl, exec.rs guard filter); six were the violation, each carrying a comment
     presenting it as a design choice. Six fixed, one boundary per commit, each RED on the
     shipped shell and GREEN on the debug build under HOME=/tmp/g4home, then all six re-read on
     the deployed binary after ship (04:10):
       1 00c7542d banner: 0 active beside ? planned -> ? active
       2 56869b21 health TUI (Ctrl+D): Active intents 0 -> ?
       3 7c908aab tools/tt: No results, exit 0 -> refusal, exit 1
       4 8ee52ff9 pick intent: No intents found, exit 0 -> refusal, exit 1
         (the commit body says silent; the shipped shell printed No intents found -- this
         line is the correction, history left alone)
       5a ac334925 fstats intents: header + 0 intents complete, exit 0 -> refusal, exit 1
       5b a1986969 fstats all: blank where a section refused -> the refusal line shown;
         extract_output now renders Error via Display, aggregate exit unchanged by decision
       6 072aae76 it: full-screen empty ledger -> one line at the prompt, no screen switch;
         limit stated: the arm is above dispatch and does not set $?
     Suite 193/193 on the debug binary before ship.
     THE REDS WERE TAKEN AGAINST deadb373: the shipped nsh was built 00:18 at the cistart
     commit, before e6b290e1 and every adapter consumer, so the daily shell carried none of
     this intent until the 04:10 ship. The version string 3.9.0 did not change across that gap.
     NOT G4, found while proving it, for 230's body or the owners named:
       - exec builtin does not search PATH: exec nsh fails, exec /home/christian/.local/bin/nsh
         works. AGENTS.md instructs exec nsh after every rebuild; that instruction cannot be
         followed today.
       - HOME redirect does not move the state home (XDG_STATE_HOME set independently):
         health, focus.toml, config all stayed present under /tmp/g4home. G5 must redirect
         both, or state what it is not testing.
       - the health TUI under an absent forest wrote forecast rows into the REAL state
         (68% then 55% then 56% across three runs). Not deleted. INT-222/192 territory.
       - health TUI reports Health 100% above ten warnings and Commits today 0; fstats says
         0 commits while the TUI once said 16. INT-222/192.
       - rulings still open: mod.rs release_name fallback The Forest Remembers beside an
         unknown version; the intents rows function yielding [] on absence. -->
- [x] G5 RUNTIME PROOF: a test runs fsh with the 0-Core paths absent or redirected, and asserts the
      shell works and the integrations report their absence. ⚠️ NOT a source-text check
<!-- evidence: 1d538eb6, 2026-09-05. nsh-test case repl_230_absent_forest_refuses_not_empties:
     HOME and XDG_STATE_HOME both pointed at a fresh /tmp directory the case creates before
     spawning (HOME alone hides the ledger; the state home reads XDG_STATE_HOME on its own and
     stayed present under a bare HOME redirect -- measured that night). Four REPL sessions, one
     door each: echo alive, tools, pick intent, fstats intents; each asserts the refusal text.
     WATCHED FAILING TWICE, for two different reasons:
       - first run, 193/194: the harness returns the capture of the LAST line only, so a
         four-line session could not assert the first three. Harness fact, recorded in the case.
       - against the shell that HAD the bug: deadb373 built in a git worktree, NSH_BIN pointed
         at it -> expected "No results." to contain "tools: needs 0-Core, which is not present".
         The case catches what it claims to catch. Worktree removed afterwards.
     GREEN: 194/194 against the 04:10 shipped binary, the case at ~741ms. -->
- [x] G6 NO COMPILE-TIME FEATURE FLAG unless a concrete reason to compile two products is recorded
      here first
<!-- evidence: satisfied BY CONSTRUCTION. No `cfg` appears in core_integration.rs or at any
     migrated call site; absence is a runtime fact throughout. The intent rejected the flag on
     the grounds that it PRESERVES the coupling while worsening the build matrix, and nothing in
     the migration produced a reason to revisit that. Verify with: grep -rn cfg( on the adapter. -->
- [x] G7 each gate carries evidence per INT-158
<!-- evidence: every gate above carries an evidence comment naming a commit, a measurement, or a
     demonstration. G4 lists six hashes with red and green per site; G5 one hash plus the red
     taken against deadb373. The one correction (8ee52ff9's body says silent where the shipped
     shell printed No intents found) is recorded in G4 rather than by rewriting history. -->

## Non-goals
- Moving the repository. ⚠️ THAT IS THE VERSION THIS INTENT REPLACES. A git remote is trivial; the
  coupling is the work, and moving the code without the boundary would produce a repo that still
  cannot build alone.
- `faelight-git`. It is a second path dependency and a smaller question, asked separately once the
  `faelight-core` boundary exists.
- Removing 0-Core features. The machine that has 0-Core should get the richer shell.
- Blocking VM validation. Explicitly out -- see above.


## ⭐ G2 PROGRESS AND WHAT IT FOUND (2026-09-04)

Commits: `fdc174a9` (boundary + four broken readers) - `3c4d7ea6` (rust_tools_dir)
- `e6b290e1` (observability). Census: 0-Core discovery **40 -> 25**.

**THE BOUNDARY IS `novashell/src/core_integration.rs`.** It CALLS
`faelight_core::paths` and never copies it, so `paths` stays the single authority
over WHERE and the adapter owns WHETHER. Runtime only, no `cfg` -- G6 holds.
Accessors so far: `present`, `ledger`, `tools_root`, `tool_manifest`,
`forest_version`, `release_name`, `health`.

### ⚠️⚠️ FOUR SURFACES WERE READING A DIRECTORY THE WORKFLOW EMPTIES
`cistart` MOVES a started intent from `future/` into `in-progress/`, and
`prompt.rs`, `session.rs`, `digest.rs:106` and `health_tui.rs` all scanned
`future/` only. Measured: `future/` held **0** intents with `status: in-progress`,
`in-progress/` held **4**. So the next-intent hint and the banner list were
STRUCTURALLY always empty, and `health_tui` reported **5** by matching the bare
word in prose rather than the frontmatter key. `core` reads the status field and
had the right answer the whole time. **The shell and the engine disagreed and
nothing noticed.** Fixed; the banner now names the real work.

### ⚠️ TWO COPIES OF ONE PATH THAT HAD ALREADY DRIFTED
`dev_cmd` built the same eleven-line manifest expression in its `test` arm and
again in its `watch` arm, byte-identical, forty lines apart -- and `test` checked
the file existed while `watch` did not, so `dev watch nosuchtool` announced cargo
watch on a path that was not there. The existence check now lives INSIDE
`tool_manifest`, where an arm cannot skip it.

Likewise `faelight/meta/VERSION` was read at four sites with **two different
answers for absence**: three said `unknown`, one produced the EMPTY STRING and
printed it as though it were a version.

### ⚠️⚠️ THREE THINGS THE ADAPTER CANNOT FIX, recorded so they are not lost
1. **`cheatsheet_tui.rs:265` reads `rust_tools_dir()/novashell/src/commands/mod.rs`.**
   nsh parses ITS OWN SOURCE at runtime to build the cheatsheet. A packaged
   install has no source tree, so it yields an empty cheatsheet with no error.
   This is a feature that cannot survive the packaging this intent exists to
   enable, and it needs its own answer: generate at build time, ship the parsed
   data, or drop it. **Deliberately not migrated.**
2. **`commands/mod.rs:6776` is `db.health_score().unwrap_or(0)`** -- a FOURTH
   health reader, fabricating a zero, reading the DATABASE rather than a path.
   ⚠️ **The census cannot see it.** G1 measures path coupling, not the defect
   class, and a clean census must never be read as a clean shell.
3. **Two version authorities, a full major apart.** `nsh version` prints
   **1.0.0** from `faelight/meta/VERSION` with a release dated 2026-07-06, while
   `core version` and the banner both say Forest **13.0.0**. Not caused by this
   work and not fixed by it.

### ✅ ONE THING THE AUGUST RECORD HAD WRONG
The note that `prompt.rs` falls back to `"100"` then `unwrap_or(100)` and asserts
PEAK health from a missing file is **STALE**. All three `read_health` callers now
match `Some`/`None` honestly and carry comments recording that the doubled
fallbacks were removed. They were wrapped for the presence check, not repaired.

### ⏭ G2 REMAINING
**25 discovery calls** (21 `intents_dir`, plus `core_root_string`,
`registry_dir`, `tools_registry`, and the cheatsheet), **3 `daemon_socket`**.

### ⏭ G4 HAS A LIST NOW, AND PART OF IT IS SELF-INFLICTED
G4 is NOT satisfied and this pass moved against it in one respect: **five
`.unwrap_or_default()` calls** were introduced to keep the migration mechanical,
each turning an absent forest into an empty `PathBuf`. Named here rather than
discovered later. Add `health_score().unwrap_or(0)` and `health_tui`'s
`unwrap_or(0)`, which survived this pass unchanged.


## ✅ G2 CLOSED, AND WHAT THE MIGRATION FOUND (2026-09-05)

The boundary work found more defects than it created abstractions. Every one is
the same shape -- **an absent or wrong value presented as a real one** -- which
is INT-227's invariant, and is why G4 has been closing site by site rather than
as a gate at the end.

### The defects, all fixed and all proven live

1. **FIVE BLIND INTENT READERS.** `cistart` moves a started intent from
   `future/` to `in-progress/`, and `prompt.rs`, `session.rs`, `digest.rs:106`,
   `health_tui.rs` and `main.rs:3416` all scanned `future/` only. Measured:
   `future/` held **0** intents with `status: in-progress`, `in-progress/` held
   **4**. So the banner list and the focus hint were STRUCTURALLY empty, and
   `health_tui` reported **5** by matching the bare word in prose. `core` had
   the right answer the whole time; the shell and the engine disagreed and
   nothing noticed.

2. **A GUARD THAT BLOCKED EVERY `rm -rf`.** The catastrophic-rm protected list
   tests `expanded.contains(protected)`, and an `unwrap_or_default()` made one
   entry the EMPTY STRING -- which every string contains. On a forestless
   machine it refused every recursive delete while naming `''` as the thing it
   protected. **Introduced by Claude during this intent** and fixed with a
   filter, not a wrap: a protected list holds paths that exist.

3. **`find @rust` SEARCHED AN EMPTY PATH.** Same `unwrap_or_default()`, and
   worse than degradation -- it OVERWROTE a good `core_root` default and handed
   `""` to `fd`. Also Claude's. All four search-root sites now refuse with a
   reason, because each one ends in a tool being pointed at a directory.

4. **FOUR VERSION READERS DISAGREEING ON ABSENCE.** Three said `unknown`, one
   produced the empty string and printed it as though it were a version.

5. **TWO COPIES OF ONE PATH THAT HAD DRIFTED.** `dev_cmd` built the same
   manifest expression in `test` and `watch`, forty lines apart; `test` checked
   the file existed, `watch` did not and announced cargo watch on a path that
   was not there.

6. **A FOCUS WRITE THAT COULD NEVER SUCCEED.** `set_focus_intent` writes
   `shell_state.focus_intent`; `get_focus_intent` reads `focus.toml`. A setter
   and a getter sharing a name and not a storage. And the caller's all-digits
   guard rejected every value the old display produced, so two independent
   failures were stacked, neither visible from the other. Filed as **INT-242**.

7. **A BANNER COMPUTATION THAT WAS NEVER USED AND WAS WRONG.** `_focus` read and
   parsed the tools registry on every render and discarded the result -- and its
   comment claimed "lowest audit score tool" while the code took the first name
   line in file order, reading no score at all.

### Recorded, not fixed here

- **`cheatsheet_tui.rs:265`** reads `rust_tools_dir()/novashell/src/commands/mod.rs`.
  nsh parses ITS OWN SOURCE at runtime, so a packaged install gets an empty
  cheatsheet with no error. ⚠️ **This cannot survive the packaging this intent
  exists to enable** and needs its own answer: generate at build time, ship the
  parsed data, or drop it.
- **THIRTY hand-built `0-core/` paths**, twelve pointing at `~/0-core/scripts`
  which does not exist, plus `completion.rs` offering `cd` to four paths that
  were never right after the tree moved. INT-240, measured.
- **Two frontmatter parsers with different tolerances** -- `intent_tui`'s reads
  a proper `---` fence, the adapter's bounds at twenty lines. Both work today.
  INT-211.
- **`db.health_score().unwrap_or(0)`** at `commands/mod.rs:6776`, a fourth
  health reader fabricating a zero from the database. Invisible to the census.
- **Two version authorities a full major apart**: `nsh version` prints 1.0.0
  from `faelight/meta/VERSION`, `core version` says Forest 13.0.0.

### ⏭ REMAINING ON THIS INTENT

**G4** -- the display-level work. `health_tui`'s `unwrap_or(0)` and
`db.health_score().unwrap_or(0)` remain, and `8144`/`16517` return an empty
success where a refusal may be better. Every `unwrap_or_default()` Claude
introduced has been removed.

**G5** -- a runtime test asserting the shell works and the integrations report
their absence. ⚠️ Not a source-text check.

**G7** -- evidence per gate.

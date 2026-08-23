---
id: 221
date: 2026-08-13
type: future
title: "fsh pick calls sk (skim) which is not installed, so every fuzzy selection fails with "sk not found" while fzf is installed on the same system"
status: complete
tags: [fsh, pick, fuzzy, dependency, int-134]
---

## Vision
A built feature reaches for a backend the system actually has.

## The Problem
MEASURED 2026-08-14 while ruling INT-134 gate 5:

    pick file   ->   x sk not found

`pick` is real and its subcommands are `pick intent`, `pick intent --active`, `pick history` and
`pick file [--core]`. It pipes candidates through an external fuzzy selector. That selector is `sk`
(skim), and **skim is not installed on this system**.

⚠️ AND A WORKING ALTERNATIVE IS ALREADY PRESENT: `fzf` IS installed. So the shell ships a feature
that cannot run, next to the tool that would let it.

Every subcommand is affected, not just files -- they all funnel through the same selector.

## HOW IT WAS FOUND, and why that matters
Nobody reported this. It surfaced because INT-134 gate 5 required a RULING on "fuzzy command
completion", and ruling required running the thing rather than reading about it. The roadmap had
carried a note calling `pick` intents-only, written from a partial subcommand list -- that note was
also wrong, and the same probe corrected both.

## The Solution
⚠️ THERE IS A REAL CHOICE HERE AND IT SHOULD BE MADE DELIBERATELY, not patched over:
  - DECLARE `sk` as a dependency in the Nix config, so the tool the code names is the tool present
  - USE WHICHEVER IS AVAILABLE -- probe for `sk`, fall back to `fzf` -- which tolerates both systems
    but puts two selectors in one code path
  - SWITCH TO `fzf` outright, since it is already installed and is the more widely available tool

⚠️ AND WHATEVER IS CHOSEN, THE FAILURE MESSAGE SHOULD SAY MORE THAN "sk not found". A user reading
that cannot tell whether they typed something wrong, whether the feature exists, or whether a
dependency is missing. Same class as INT-215: an accurate message that leaves the reader guessing.

## Evidence (measured 2026-08-14)
- `pick file` reports "sk not found". `pick` with no argument prints its full usage, so the builtin
  itself is fine -- the failure is at the selector.
- `shutil.which` reports sk absent and fzf PRESENT on this system.
- The subcommands are pick intent, pick intent --active, pick history, pick file [--core].

## Success Criteria
- [x] G1 RED FIRST: the failure is reproduced by a case, not only by hand -- `pick` with a selector
      absent, asserting the message names a missing dependency rather than an unknown command
<!-- demonstrated: fsh-test case `pick_without_fzf_names_the_dependency`. Went RED first (159 cases,
     158 passing), now green. Builds its PATH at RUNTIME by dropping only directories containing an
     fzf executable, so it cannot rot when store hashes change. Uses `pick intent` rather than
     `pick file` so nothing (rg) can fail before the selector is reached. Commit 9ff4c80c. -->
- [x] G2: THE CHOICE IS MADE AND RECORDED with what it gives up -- declare sk as a dependency, probe
      for either, or switch to fzf. A decision, not a default
<!-- RULED: SWITCH TO fzf OUTRIGHT. fzf is the supported fuzzy-selector backend for fsh.
     WHY: it is already installed by the system configuration; recon found NO sk-specific flags
     (--prompt, --height, --reverse, --ansi are common to both), so no behaviour is given up; and
     for a NixOS-controlled shell deterministic dependencies beat runtime backend discovery.
     WHAT IT GIVES UP: an installation that has skim but not fzf is no longer served. That case
     does not exist here and is not worth two selectors in one code path.
     REJECTED: declaring sk a Nix dependency (adds a package to satisfy an implementation detail)
     and probing for either (solves a problem we do not have). Commit 9ff4c80c. -->
- [x] G3: PROVEN: the chosen selector runs on this system and a fuzzy selection completes end to end
<!-- demonstrated live 2026-08-22: `pick history` opened fzf and returned `Selected: <line>`;
     `pick intent` opened and exited silently on Escape; `pick intent --active` listed 2 entries.
     Needs a TTY, so it is a manual demonstration by design. -->
- [x] G4: the failure message names the missing dependency and what to do about it, so a reader can
      tell a missing tool from a typo. Same class as INT-215
<!-- "pick: fzf is required for fuzzy selection but was not found on PATH -- add fzf to the system
     environment". Names the executable, says it is a dependency, says what to do. Asserted by the
     G1 case, which fails if the message stops naming fzf. -->
- [x] G5: every subcommand is checked, not only `pick file` -- they share one selector path, so a fix
      at that path must be shown to serve all four
<!-- STRUCTURAL, not demonstrated: `grep -c 'Command::new("sk")'` -> 0, `grep -c "fuzzy_select("`
     -> 4 (the helper plus three callers). All four subcommands route through one path as a
     countable fact.
     AND THE GATE EARNED ITS KEEP: taking it literally found a SECOND defect. `pick intent --active`
     meant its opposite -- the condition skipped everything that was NOT future, so it listed 51
     future intents and hid the two in progress. Root cause recorded in the code: `status` is the
     DIRECTORY name and there is no `planned` directory, so this picker labelled an intent [future]
     while `intl` labelled the same intent [planned]. Two vocabularies for one state. Fixed in
     commit 73859562; demonstrated 2/2 in-progress where 51 appeared before. -->
- [x] G6: the fsh-test suite stays green, and a case covers the selector-absent path so this cannot
      regress silently
<!-- 159/159 green. The selector-absent case IS the G1 case, so the regression boundary and the
     red-first proof are the same artifact. -->
- [x] G7: each gate carries evidence per INT-158
<!-- this block. -->

## Outcome
Three sites each reached for skim directly and had already drifted apart -- two said "sk not found",
one said "sk not found -- install skim". They are now ONE helper, `fuzzy_select(items, prompt, ansi)`,
returning raw text so each caller keeps its own parsing, because the three genuinely differ:
`pick intent` strips `INT-`, `pick history` drops a timestamp column, `pick file` needs none.

TWO THINGS FOUND ALONG THE WAY, both recorded rather than left silent:
- A SEMANTIC CHANGE: cancellation used to be caught by the selector's EXIT STATUS. fzf exits
  non-zero on Escape and the helper returns its (empty) stdout regardless, so cancellation is now
  caught by the empty check. Same outcome, different route -- noted in the code at each site.
- THE `--active` INVERSION above, which the gate found only because the gate was taken literally.

SCOPE, RESOLVED: `faelight-clipboard/src/main.rs` had ALREADY switched to fzf in P2 with the same
reasoning; only its `.context("sk not found")` message lagged. Corrected, so nothing anywhere calls
sk and the next reader will not find one and wonder what this intent settled.

CARRIED FORWARD, not fixed here:
- The selected row is highlighted in Hakker Green, HARDCODED with a comment saying why: the palette
  exists only as PROSE in ROADMAP.md and a decisions file, so there is nothing to import. When the
  single token source lands, this is one of the sites that should read from it.
- A colour spec is a string the compiler never validates. A missing colon in `hl+#00ff99` BUILT
  CLEANLY and was caught only when fzf rejected it at runtime -- the same class as an unbound SQL
  placeholder. A test that runs the selector's flag set once would close that gap.
- `pick intent` labels states from directory names while `intl` reads frontmatter. One ledger, two
  vocabularies. INT-211 is where that belongs.

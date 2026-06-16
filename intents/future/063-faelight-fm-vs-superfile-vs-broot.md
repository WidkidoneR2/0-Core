---
id: 063
date: 2026-06-16
type: feature
title: \"Faelight-FM vs Superfile vs Broot\"
status: planned
tags: [file-manager, tui, faelight-fm, superfile, broot, survey, decision, nixos]
version: TBD
priority: low
---

## Why
Faelight Forest already ships faelight-fm, our own Rust TUI file manager. But
the temptation to try the newest shiny file manager keeps recurring, and each
detour costs focus. This intent closes the question once, on purpose: run a
single structured comparison of faelight-fm against the two strongest external
contenders, then DECIDE -- adopt one, fold its best ideas back into faelight-fm,
or confirm faelight-fm and stop wandering.

The deliverable is a decision with a recorded rationale, not a new tool. "Done"
means we never have to casually re-litigate the file-manager question again.

## What Already Exists
- faelight-fm -- our own Rust TUI FM, already wired into the forest: INT-033
  neon-candy theme, fsh integration, the vocabulary/semantic model, Friday. The
  home team; the baseline everything else is measured against.
- superfile (spf) -- Go + Bubble Tea TUI. GUI-like: multi-panel layout, a
  sidebar (XDG folders, pinned dirs, mounted disks), file + image preview, fuzzy
  search, bulk ops, a community plugin system (e.g. git status). On nixpkgs.
  Strength: refined, modern UI. Not Rust.
- Broot -- Rust TUI. Tree-view navigation + fuzzy matching, file manipulation
  via a command syntax, and the `br` shell function that cd's you to the chosen
  directory on exit. Strength: fast keyboard/tree navigation.
- (optional 4th -- see Open Question) yazi -- the most feature-packed Rust FM,
  async I/O, rich previews. Worth ruling in/out if the goal is to close this for
  good.

## Vision
A filled scoring matrix and a written decision, reached from real daily-driving
-- not first impressions. Each candidate installed declaratively, used for a
fixed set of real tasks, scored against shared criteria, then judged.

## Approach
1. Lock the evaluation criteria below BEFORE testing, so the decision is not a
   post-hoc rationalization.
2. Install each candidate declaratively (nix / home-manager) -- no imperative
   installs, consistent with the forest's declarative rule.
3. Daily-drive each on a FIXED task set (see Phases) so they are compared on the
   same work, not different work.
4. Score each criterion, record the matrix in this charter, then decide.

Evaluation criteria:
- Declarative install: clean to add and pin via nix?
- Keyboard model + fsh / vocabulary fit: matches how the forest already works,
  or fights it?
- Preview / image support: text, image, metadata.
- Speed + footprint: responsiveness and RSS on large directories.
- Integration potential: can it talk to Friday / theme / the semantic model, or
  is it a sealed box?
- Dependency + trust surface: timely after the Atomic Arch AUR supply-chain
  attack -- superfile's Go module tree and plugin system vs. the two Rust
  options. Not a Rust-purity test; an honest look at what each pulls in and runs.

## Phases
Phase 1 -- declarative install of all candidates
  Add faelight-fm (present), superfile, Broot (+ yazi if included) to the nix
  config so each launches from a clean rebuild.
  Gate: each candidate installs declaratively and launches

Phase 2 -- fixed-task daily-drive
  Run the SAME task set through each: navigate a deep tree; copy/move/rename;
  bulk-select + delete; fuzzy-jump to a known path; preview a text file and an
  image; return-to-shell-in-that-dir.
  Gate: the fixed task set is run end-to-end in every candidate

Phase 3 -- score the matrix
  Fill the criteria-by-candidate matrix in this charter from the Phase 2 runs.
  Gate: scoring matrix completed and recorded in this charter

Phase 4 -- decide and close
  Record the decision and rationale: adopt one / fold ideas into faelight-fm /
  confirm faelight-fm. If "fold ideas," list the specific ideas as follow-ups.
  If "adopt," the winner is added declaratively; if "reject all," the
  faelight-fm gaps the contenders exposed are logged for faelight-fm work.
  Gate: decision + rationale recorded in this charter
  Gate: chosen outcome actioned (winner declarative, or gaps/ideas logged) so
        the question is genuinely closed

## Gates
- [ ] each candidate installs declaratively and launches (Phase 1)
- [ ] fixed task set run end-to-end in every candidate (Phase 2)
- [ ] scoring matrix completed and recorded in this charter (Phase 3)
- [ ] decision + rationale recorded in this charter (Phase 4)
- [ ] chosen outcome actioned so the question is closed (Phase 4)

## Depends On
  none (self-contained survey; nothing else is blocked on it)

## Open Question
  Include yazi as a 4th contender? Default: keep to the three named unless
  decided otherwise before Phase 1.

## The Rule
"Try them once, on purpose -- then choose, and stop wandering.
 The forest keeps one path to its files." 🌲

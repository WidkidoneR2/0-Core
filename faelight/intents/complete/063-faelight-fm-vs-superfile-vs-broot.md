---
id: 063
date: 2026-06-16
type: feature
title: Faelight-FM vs Superfile vs Broot
status: complete
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
- superfile -- Go + Bubble Tea TUI. GUI-like: multi-panel layout, a
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


## Scoring Matrix
Recorded from real daily-driving (2026-06-16 -> 06-18), not first impressions.
Criteria locked before testing (see Approach). Marks are qualitative from use,
not micro-benchmarks; the decision did not hinge on RSS numbers but on the one
criterion that cleanly separated the field (editor handoff, below).

| Criterion                   | faelight-fm     | superfile             | broot          | yazi                |
|-----------------------------|-----------------|-----------------------|----------------|---------------------|
| Declarative install (nix)   | yes (home team) | yes (nixpkgs)         | yes (nixpkgs)  | yes (nixpkgs)       |
| Keyboard / fsh-vocab fit    | native, thin    | GUI / mouse-leaning   | command-syntax | vim-like, clean     |
| Preview / image             | basic (WIP)     | rich                  | tree only      | richest (async)     |
| Speed + footprint           | light (Rust)    | heavier (Go)          | light (Rust)   | light (Rust)        |
| Integration potential       | total (ours)    | plugins, sealed       | sealed         | scriptable          |
| Dependency / trust surface  | minimal (ours)  | Go + plugins (largest)| Rust, modest   | Rust, modest        |
| Editor handoff (DECISIVE)   | WIP, n/a yet    | no (opens in GUI)     | no (opens GUI) | YES (file -> helix) |

## Decision
ADOPT yazi as the forest's working file manager; ELIMINATE broot and superfile;
RETURN faelight-fm to WIP (not abandoned -- a future rebuild may match or beat
yazi; "like superfile" means a better ratatui TUI, not a GTK rewrite).

Rationale: criteria were locked before testing, but in daily-driving one
criterion proved decisive -- a file manager must drop you straight into the
editor (helix). yazi opens a file directly in helix and returns cleanly; broot
and superfile both punt files to a GUI / LibreOffice handler (superfile's opener
is likely configurable, but that was moot once yazi already met the bar). broot
also lost on a theming clash and a confusing command-syntax model. On the locked
criteria yazi is otherwise at least even with the field (Rust, async, richest
previews, scriptable, modest trust surface). yazi was the 4th contender from the
Open Question -- included, and it won.

Actioned (so the question is genuinely closed):
- yazi: retained, declarative, SUPER+e opens it (config.conf) -- the keeper.
- broot: removed -- package, SUPER+e repoint, doctor check (commit d1197382).
- superfile: removed -- package (commit d1197382).
- faelight-fm: logged as WIP for future improvement. INT-058 (decommission yazi)
  is now obsolete -- its premise (broot as sole FM) is inverted -- and is cancelled.

Follow-up (optional, not a gate): add a yazi doctor check to replace the removed
broot check, so the forest still verifies its file manager.

## Gates
- [x] each candidate installs declaratively and launches (Phase 1)
- [x] fixed task set run end-to-end in every candidate (Phase 2)
- [x] scoring matrix completed and recorded in this charter (Phase 3)
- [x] decision + rationale recorded in this charter (Phase 4)
- [x] chosen outcome actioned so the question is closed (Phase 4)

## Depends On
  none (self-contained survey; nothing else is blocked on it)

## Open Question
  Include yazi as a 4th contender? Default: keep to the three named unless
  decided otherwise before Phase 1.

## The Rule
"Try them once, on purpose -- then choose, and stop wandering.
 The forest keeps one path to its files." 🌲

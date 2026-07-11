---
id: 139
date: 2026-07-10
type: future
title: "TUI Git Log Reader"
status: complete
tags: [Git, git-log, tui]
---

## Why
Reading git log is a daily papercut -- and the INT-130 reconciliation made it acute. Auditing
23 intents meant `git log --oneline --all --grep=NNN` dumps, escaping pagers, and eyeballing
walls of output to disambiguate NixOS-era commits from Arch-era ones that reused the same intent
number (099-Niri vs 099-multiline, 103-idle vs 103-prompt, 104-wallpaper vs 104-schema, etc).
The quick aliases (`gla` = graph/all, `glog` = last 10) cover the common cases, but there is no
tool for the HARD case: search/filter the full history by intent number, date, keyword, or author,
read a commit's detail, and SEE the structure with color. This intent builds that -- a forest-
native TUI git-log reader, in the image of the cheatsheet (`cheat`) but with candy-neon pushed.

## The Problem
- `git log` output is a flat wall; filtering means re-running with different `--grep`/`--author`
  flags and re-reading from scratch.
- Pagers (less) interrupt flow -- escape, re-run, re-page.
- Same-numbered intents across the Arch->NixOS renumber are hard to tell apart in raw log;
  you read commit bodies/dates by hand to disambiguate (exactly the INT-130 pain).
- No at-a-glance color structure: intent-numbers, dates, authors, refs all blend into one color.

## The Solution (standalone tool -- decided: option B)
A standalone ratatui TUI crate (like faelight-fm, faelight-ade), its own binary, invokable
outside fsh -- NOT an fsh builtin. A git-log reader is about the REPO, not the shell, so it earns
its own tool. It mirrors the cheatsheet TUI's proven architecture (faelight-shell/src/
cheatsheet_tui.rs): load-once + filter-each-frame + list/detail split + fuzzy search mode.

## Architecture (mirror the cheatsheet's ratatui pattern)
The cheatsheet's run_cheatsheet_tui/run_loop is the template:
- enable_raw_mode + EnterAlternateScreen -> Terminal::new(CrosstermBackend) -> run_loop -> clean
  teardown (LeaveAlternateScreen + disable_raw_mode). Reuse this exact scaffold.
- State (mirrors cheatsheet's run_loop): all_commits loaded once (from `git log`), a Filter enum
  (All / IntentNumber / Author / DateRange / Keyword), a search String + `searching` bool
  (/-to-search, Esc clears, Enter confirms, Backspace edits), ListState for the commit list,
  detail_scroll for the expanded commit view.
- filter_commits(all, filter, search) recomputed each frame (like filter_entries).
- Data source: shell out to `git log --pretty=<format>` (hash, date, author, subject, refs) and
  parse -- OR use the git2 crate for structured access (Phase 0 decision: git2 gives typed data +
  no parsing, shelling out is simpler + zero dep; lean git2 for robustness, decide at build).

## Candy-neon (push past the cheatsheet -- launcher/logout/prompt family, INT-033)
Every column color-coded with MEANING, brighter than the cheatsheet's floor:
- commit hash        -- dim/gray (secondary, like the prompt's dim separators)
- intent-number      -- lavender (#AFA9EC) -- MATCHES the prompt's intent zone + the bar (INT-138);
                        so INT-NNN reads the same color everywhere in the forest
- date               -- ice-blue -- nix/time family
- author             -- aqua (#36E0D0)
- subject/message    -- fog-white (#D7E0DA) primary text
- refs/HEAD/branch   -- lime (#A6E22E) -- structure, the signature color
- era disambiguation -- Arch-era commits (pre-NixOS-migration date) subtly DIMMED or marked, so
                        same-numbered NixOS-era vs Arch-era commits are visually distinct at a glance
                        (directly solves the INT-130 disambiguation pain).

## Features (git-log-specific, beyond the cheatsheet)
- Filter by INT-number: type a number, see only that intent's commits (the reconciliation
  workhorse -- would have made the whole INT-130 audit faster).
- Filter by author / date-range / keyword (fuzzy over subject + body).
- List/detail split: list of commits (like the cheatsheet's entry list); Enter/expand shows the
  full commit (message body, changed files via --stat, refs).
- Copy-hash: yank the selected commit hash (for `git show` / cherry-pick / citing in intents).
- Snappy: load once, filter in-memory (keep it fast to open, like `cheat`).

## Approach (phased, demonstrated not declared)
- Phase 0: git2 vs shell-out `git log` decision (spike both briefly; pick on robustness + speed).
- Phase 1: ratatui scaffold copied from the cheatsheet pattern; render a scrollable commit list
  from real `git log`. Launches, scrolls, quits clean.
- Phase 2: search/filter (INT-number first -- the highest-value filter -- then author/date/keyword).
- Phase 3: candy-neon color-coding with meaning; era-disambiguation dimming.
- Phase 4: detail view (expand a commit: body + --stat), copy-hash.
- Each phase build-gated; standalone binary in registry + an fsh alias to launch it.

## Gates (when built -- demonstrated, not declared)
- [x] Standalone ratatui tool launches, reads real `git log`, scrolls a commit list, quits clean <!-- STAMP-139-DONE 2026-07-11 Phase 1: faelight-glog crate (rust-tools/faelight-glog), shells out to `git log --pretty` (Phase 0 decision: zero-dep shell-out over git2), loads all 3640 commits, cheatsheet-pattern scaffold (raw-mode -> alt-screen -> run_loop -> clean teardown), j/k+arrows scroll, q/Esc/Ctrl-C quit. Demonstrated live. -->
- [x] Filter by INT-number works (type NNN -> only that intent's commits) -- the INT-130 workhorse <!-- 2026-07-11 Phase 2: `/`-to-search (cheatsheet flow), live substring filter over subject; typing e.g. 114 shows only INT-114 commits, Esc clears. Demonstrated. -->
- [x] Filter by author / date-range / keyword (fuzzy over subject+body) works <!-- 2026-07-11 PARTIAL (honest): KEYWORD filtering works (same `/`-search substring covers keyword AND INT-number over the subject line). DEFERRED to v2 (noted by Christian): dedicated author filter, date-range filter, and fuzzy-over-BODY (currently subject-only). The highest-value cases (INT-number + keyword) ship now; author/date-range/body-fuzzy are a documented v0.2 enhancement, likely alongside the planned floating-window view. -->
- [x] Candy-neon color-coding with MEANING: intent-number=amber (NEON_AMBER, matches nothing else in glog so it pops), refs=green (NEON_GREEN, structure), hash=muted-gray, subject=fog-white -- <!-- 2026-07-11 Phase 3: colors pulled from faelight-core::theme (imports the palette, INT-091-aligned single-source, NOT hardcoded). Charter said lavender/lime; at build Christian chose amber for INT-numbers (reads better) + green refs. Consistent with the forest palette tokens. -->
      author=aqua, date=ice-blue -- consistent with the forest palette (INT-033)
- [x] Same-numbered Arch-era vs NixOS-era commits are visually distinguishable (era disambiguation) <!-- 2026-07-11 Phase 4: is_arch_era() compares the ISO commit date to the 2026-06-01 NixOS daily-driver cutoff; Arch-era commits render dimmed (MUTED_GRAY + DIM) with an [arch] marker, NixOS-era full candy-neon. Directly solves the INT-130 099-Niri-vs-099-multiline pain. Demonstrated. -->
- [x] Detail view: expand a commit (message body + --stat); copy selected hash <!-- 2026-07-11 Phase 4: Enter toggles a full-screen detail (on-demand `git show --stat --format=%b`, fog-white, scrollable j/k + PageUp/Down), Esc back. `y` copies the commit hash via wl-copy (works from list AND detail; status line confirms 'copied <hash>'). Demonstrated. -->
- [x] Snappy to open (load-once + in-memory filter, like `cheat`); registered + aliased <!-- 2026-07-11: load-once at startup, all filtering in-memory per-frame (cheatsheet pattern) -- opens instantly. Registered in tools.toml (faelight-glog, deployable=true), aliased `fgl` in config.fsh. Deployed gen 345 to /run/current-system/sw/bin/faelight-glog (tool count 56->57, 29/29 deployed). `fgl` launches from anywhere. -->

## RESOLUTION (2026-07-11): SHIPPED faelight-glog v0.1.0 -- built in 4 phases, deployed, aliased `fgl`

A standalone ratatui TUI git-log reader, built demonstrated-not-declared in four compiled phases:
- **P1** scaffold: cheatsheet-pattern (load-once + per-frame filter + ListState), shells out to
  `git log` (Phase 0 decision: zero-dep shell-out chosen over git2, which is available as an
  upgrade path). Loads all 3640 commits, scrolls, quits clean.
- **P2** live `/`-search: substring filter over subject -- covers INT-number (the INT-130
  reconciliation workhorse) AND keyword in one box.
- **P3** candy-neon from faelight-core::theme (single-source palette, INT-091-aligned): INT-number
  in amber (Christian's build-time call over the charter's lavender -- reads better), refs green,
  hash gray, subject fog-white.
- **P4** era-dimming (Arch <2026-06-01 dimmed + [arch] marker -- solves the same-numbered
  Arch/NixOS disambiguation pain), toggle detail view (Enter -> body + --stat, scrollable, Esc
  back), copy-hash via wl-copy (`y`).

Registered (tools.toml, deployable), aliased `fgl`, deployed gen 345 (57 tools).

**Deferred to v0.2 (Christian's note):** dedicated author + date-range filters, fuzzy over commit
BODY (currently subject-only). Bigger idea on the table: a **floating-window** view of glog (and
more) rather than / in addition to the TUI -- a candy-neon float showing commit info at a glance.
That's the natural next version; this intent shipped the terminal reader that earns its keep now.

## Depends On / Relates To
- INT-092 (cheatsheet TUI) -- the architectural template (ratatui load-once/filter/list-detail).
- INT-033 (candy-neon palette -- the shared color tokens).
- INT-103 (prompt) + INT-138 (bar) -- intent-number=lavender must match, so INT-NNN reads the same
  color across prompt, bar, and this reader (one visual language).
- Born from the INT-130 reconciliation (the disambiguation + filter pain that motivated it).

## The Rule
"The forest should be as easy to READ as it is to build. Its history -- who did what, when, for
 which intent -- should be one search away, in the forest's own colors, never a wall of gray." 🌲

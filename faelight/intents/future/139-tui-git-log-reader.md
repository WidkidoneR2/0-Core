---
id: 139
date: 2026-07-10
type: future
title: "TUI Git Log Reader"
status: planned
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
- [ ] Standalone ratatui tool launches, reads real `git log`, scrolls a commit list, quits clean
- [ ] Filter by INT-number works (type NNN -> only that intent's commits) -- the INT-130 workhorse
- [ ] Filter by author / date-range / keyword (fuzzy over subject+body) works
- [ ] Candy-neon color-coding with MEANING: intent-number=lavender (matches prompt/bar), refs=lime,
      author=aqua, date=ice-blue -- consistent with the forest palette (INT-033)
- [ ] Same-numbered Arch-era vs NixOS-era commits are visually distinguishable (era disambiguation)
- [ ] Detail view: expand a commit (message body + --stat); copy selected hash
- [ ] Snappy to open (load-once + in-memory filter, like `cheat`); registered + aliased

## Depends On / Relates To
- INT-092 (cheatsheet TUI) -- the architectural template (ratatui load-once/filter/list-detail).
- INT-033 (candy-neon palette -- the shared color tokens).
- INT-103 (prompt) + INT-138 (bar) -- intent-number=lavender must match, so INT-NNN reads the same
  color across prompt, bar, and this reader (one visual language).
- Born from the INT-130 reconciliation (the disambiguation + filter pain that motivated it).

## The Rule
"The forest should be as easy to READ as it is to build. Its history -- who did what, when, for
 which intent -- should be one search away, in the forest's own colors, never a wall of gray." 🌲

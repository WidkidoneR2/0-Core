---
id: 071
date: 2026-06-20
type: future
title: "Friday: restore Nix-era parity (commit-to-intent recording, then learning)"
status: in-progress
tags: [friday, learning, commit-recording, intent-commits, parity, migration, nixos]
---

## Why
Friday's commit->intent recording has been dark since the Arch->NixOS migration: the
intent_commits table froze at an Arch-era commit, so Friday no longer learns from new
commits, and INT-034 (generation + commit + intent triad tracking) is blocked behind it.
More broadly, Friday kept doing its job but "not the Nix way." This intent restores
Friday's Arch-era parity on NixOS.

## What
Confirmed mechanism (Phase 0, 2026-06-22): the intent_commits recorder -- the
INSERT OR IGNORE INTO intent_commits -- exists ONLY in faelight-git's `fg done`
(commands/done.rs, INT-312). The NixOS daily commit path is interactive `fg sync`
(commands/sync.rs), which has NO such write. On Arch commits went through `fg done`; on
NixOS the workflow moved to `fg sync`, orphaning the recorder. Commit RECORDING severed
at the migration while commit DETECTION survived -- the separate git_commit signal still
fires the "commit detected" notification. The table froze at row 2972 (INT-328,
committed_at = 2026-06-01, the cutover); ~250+ commits since are unrecorded.
Work:
- Restore recording on the path actually used (`fg sync`) WITHOUT duplicating the
  recorder: extract one shared helper and call it from both `done` and `sync`.
- Fill columns the recorder never set (gate_hint, author); stop hard-coding intent_status.
- Restore Friday learning from commit history once rows flow again.
- Later: revert gen-diff's "git is source of truth" workaround to read the table again.
Scope boundary: PARITY RESTORATION (recover what the migration broke), distinct from
INT-039 (friday-daemon) and INT-041 (shell-context), which are NEW Friday features.

## Approach
Phase 0 (done): traced the break in code -- recorder in done.rs only, sync.rs lacks it,
git_commit detection is a separate live signal; freeze pinned to 2026-06-01. Phase 1:
extract the recorder into one shared record_commit(hash, message) (commands/mod.rs) and
call it from BOTH done.rs and sync.rs -- one recorder, two callers, no drift (anti-
proliferation); fill the long-blank columns and set intent_status honestly. Phase 2:
confirm Friday consumes the now-flowing rows (learning, not just storage). Phase 3: close
or formally defer the remaining Phase-0 gaps -- notably reverting the gen-diff workaround.

## Phases
Phase 0 -- parity audit: record the Arch->Nix gap list here.
Phase 1 -- repair commit->intent recording (intent_commits frozen since migration).
Phase 2 -- learning resumes: Friday consumes recent commit history again.
Phase 3 -- close remaining parity gaps (or formally defer).

## Phase 0 Findings (2026-06-22) -- parity-gap list
1. SEVERED WRITE (headline): the intent_commits recorder lives only in `fg done`
   (done.rs, INT-312); `fg sync`, the NixOS daily commit path, never writes it. Frozen at
   row 2972 / INT-328 / 2026-06-01. ~250+ commits since are unrecorded. Detection
   (git_commit signal) still fires; only the genealogy INSERT is absent on the used path.
2. NEVER-FILLED COLUMNS: even working Arch-era rows left gate_hint and author blank and
   hard-coded intent_status = 'in-progress'. The table was designed richer than filled.
3. WORKAROUND TO UNWIND: gen-diff (main.rs:91) routes around the stale table ("git is the
   source of truth") instead of reading it -- revert once recording is restored (Phase 3).
4. committed_at is the real epoch time column (not "timestamp").
Headline = gap #1; #2-#4 fold into Phases 1 and 3.

## Phase 1 Results (2026-06-22)
Phase 1 uncovered THREE migration casualties, not one:
1. SEVERED WRITE (the headline): the intent_commits recorder lived only in fg done; fg sync
   (the NixOS daily path) never called it. Fix: extracted get_active_intent + record_commit
   into commands/mod.rs as ONE shared recorder; both fg done and fg sync call it. Recorder now
   also fills author and an honest intent_status (none when no active intent). gate_hint -> Phase 3.
   Proof: row 2972 (INT-328, 2026-06-01) -> row 2973 (today) -- 21-day gap closed.
2. ATTRIBUTION: record_commit first read the lowest-numbered active intent (got 005 with five
   in-progress). Fix: parse the leading INT-NNN from the commit message (ground truth), fall
   back to the active scan. Proof: row 2974 recorded intent_id=71 correctly.
3. FOCUS-READ DRIFT: friday-chat get_active_intent read the stale shell_state focus_intent row
   instead of focus.toml (the source cistart writes, that faelight-shell trusts). Fix: read the
   toml first, shell_state as fallback -- matching faelight-shell. Proof: /intent now shows
   Active intent: INT-071 and lists rows 2973/2974 (Friday reading the revived table).
Gate 2 closed (recording resumed AND correctly attributed). Gate 3 closed (Friday cites the
restored commits live -- demonstrated, not wired).

## Gates
- [x] Phase 0: Friday Arch->Nix parity-gap list recorded in this charter
- [x] commit->intent recording repaired: a new commit records an intent_commits row past the Arch-era freeze
- [x] Friday learns from recent commits again (demonstrated, not just wired)
- [ ] remaining Phase-0 parity gaps resolved or formally deferred

## Notes
- Unblocks INT-034 (triad tracking needs live commit-recording).
- Distinct from INT-039 (daemon) and INT-041 (shell-context): new features, not parity.
- Your framing: "doing its job, just not the Nix way" -- this is the recovery.

## The Rule
"The forest remembers -- including why each commit was made." 🌲

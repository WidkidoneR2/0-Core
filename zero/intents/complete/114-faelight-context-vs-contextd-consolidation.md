---
id: 114
date: 2026-07-02
type: future
title: "faelight-context vs contextd consolidation"
status: complete
tags: [context, cleanup, naming.]
---

## Vision
Resolve the faelight-context vs faelight-contextd naming/function overlap so
there's one clear tool (or two clearly-distinct ones), eliminating confusion.

## The Problem
Two similarly-named tools coexist in the workspace:
- faelight-context (v1.0.0)
- faelight-contextd (v0.1.0)
The `-d` suffix usually means "daemon," implying one is a CLI and one a background
service -- but the naming is close enough to cause confusion. Unclear if they're
(a) a proper CLI+daemon pair (keep both, maybe clarify names), (b) one superseding
the other (retire the loser), or (c) accidental duplication (merge).

## Recon needed (before deciding)
- What does faelight-context DO? (CLI? one-shot context query?)
- What does faelight-contextd DO? (daemon? persistent context service?)
- Do they share code / a state source? Does one call the other?
- Is contextd (v0.1.0, early) an in-progress replacement for context, or a
  companion daemon?

## Decision space
- KEEP BOTH as a clear CLI + daemon pair (possibly rename for clarity).
- MERGE if duplicative.
- RETIRE one if superseded (get-version/profile pattern).

## RESOLUTION (2026-07-11): KEEP BOTH + rename contextd->insightd + REVIVE the daemon

**Two distinct tools, kept both.** faelight-context (CLI, one-shot codebase analysis) and the
daemon are a real CLI+daemon pair -- not duplicates. The problem was the one-letter name
collision (context / contextd) that risked cd-ing into the wrong directory and doing damage.

**Renamed the daemon: faelight-contextd -> faelight-insightd** (Christian's instinct; matches
its own "surfaces insights" description; collides with neither `context` nor the planned
`fridayd`). Touched: crate dir + Cargo.toml + bin, root Cargo.toml member, the systemd unit,
7 live code refs (engine main + predict/strategy/engines domains + fsh main -- including the
FUNCTIONAL `systemctl is-active` health check), and the registry. Kept faelight-context as-is.

**Registry fixes (the concrete confusion 114 was filed for):** the contextd entry was malformed
(`[[tools]]` plural, missing `type`, `deployed` instead of `deployable`) -- corrected + renamed.
And faelight-context was DUPLICATED (two [[tool]] entries) -- removed the stale one.

**BONUS -- revived a dead subsystem (the real magic).** The daemon had NEVER run. Its health
factor ("Nervous System +5", strategy/mod.rs:1581) had always scored false, because: (a) the
systemd unit was an orphaned loose dotfile -- no home-manager config ever referenced it, so it
was never deployed (`is-enabled` = not-found), and (b) even if loaded, its ExecStart pointed at
a dead Arch-era path (`/home/christian/0-core/scripts/faelight-contextd`). Fixed by wiring it as
a proper home-manager `systemd.user.services.faelight-insightd` block (modeled on faelight-wsd,
INT-053), ExecStart -> the real Nix binary path, WantedBy faelight-session.target. Result: the
daemon is now `active (running)`, enabled, polling state.db every 30s, and the engine sees it --
the forest's nervous system is online for the first time.

**Forward note (fridayd):** insightd already does Friday-adjacent signal work (state.db watching,
failure-loop detection). When fridayd is built (strictly for Friday), decide then whether fridayd
coexists with insightd or absorbs its signal-watching -- do NOT rename insightd toward friday.

## Gates (when built)
- [x] Function of each tool documented <!-- STAMP-114-DONE 2026-07-11: faelight-context (v1.0.0, INT-159) = one-shot CLI, Deep Codebase Understanding Engine (scan/map/patterns/summary/decisions); run-and-exit. faelight-insightd (was faelight-contextd v0.1.0) = persistent daemon, 30s poll of runtime/state.db, processes signals (failure-loop detection at main.rs:101), surfaces insights; the forest's nervous system. Verified from source (main.rs headers, loop{} at :300) not memory. -->
- [x] Relationship (pair / duplicate / supersede) determined <!-- 2026-07-11: DISTINCT CLI + DAEMON PAIR, not duplicates. Different jobs: context analyzes code STRUCTURE on demand; insightd watches runtime SIGNALS continuously. They share only the (former) name stem -- no shared code path, no supersession. The one-letter collision context/contextd was the only real problem. -->
- [x] Decision executed: keep-both / merge / retire, with clear naming <!-- 2026-07-11: KEEP BOTH + RENAME + REVIVE. (1) Renamed faelight-contextd -> faelight-insightd (crate, bin, unit, 7 code refs, registry) to end the context/contextd collision that risked wrong-directory damage; kept faelight-context unchanged. (2) Fixed registry: malformed [[tools]] entry -> [[tool]] w/ correct schema; removed a DUPLICATE faelight-context entry. (3) BONUS REVIVE: the daemon had never actually run -- its systemd unit was an orphaned loose dotfile (never referenced by any home-manager config, so never deployed) AND its ExecStart pointed at a dead Arch-era path (scripts/faelight-contextd). Rewired as a proper home-manager systemd.user.services.faelight-insightd block (modeled on faelight-wsd), fixed ExecStart to /run/current-system/sw/bin. Now: active(running), enabled, 30s poll, WantedBy faelight-session.target. `systemctl --user is-active faelight-insightd` = active, so the engine's Nervous System (+5) health factor (strategy/mod.rs:1581) scores TRUE for the first time. -->

---

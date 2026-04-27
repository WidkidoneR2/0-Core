---
id: 252
date: 2026-04-27
type: future
title: "fsh connection unification"
status: in-progress
tags: [faelight, shell, refactor, stabilization]
version: TBD
---
Eliminate every rogue `Connection::open` call in faelight-shell. All SQLite access flows through the single connection owned by `ForestDb`. The visible payoff is the disappearance of the "attempt to write a readonly database" warning that fires every prompt. The structural payoff is that fsh stops fighting itself for WAL locks, and future builtins have one obvious way to talk to state.db.
This is gate zero of fsh stabilization week. The read-only warning is hitting Christian on every prompt right now, and yesterday's failed audit traced the root cause to multi-handle contention on state.db. Every other shell bug (heredoc, fsearch narrowing, rspatch escapes, pipeline issues) sits on top of this foundation. Fix the foundation first.
Mechanical refactor in three phases, one site at a time, build between each.
**Phase A -- Safe sites (6).** Functions in commands/mod.rs and completion.rs that already have &ForestDb in scope or trivially can. No prompt hot path involvement. Lowest risk, fastest wins.
**Phase B -- Hot path (6).** Six sites in main.rs that run during prompt rendering. Need to confirm how ForestDb flows through the REPL loop before touching. Highest user-visible impact.
**Phase C -- session.rs (3).** Session lifecycle module. Confirm whether session.rs needs its own connection lifecycle or can borrow from ForestDb. Likely the latter.
**Tooling rules for this intent:** Use the edit builtin for changes, never string replacement. Build and verify after every single site. No batching. If a site fails to compile, stop and diagnose -- do not move on.
- [ ] All 16 rogue Connection::open sites replaced with &db.conn
- [ ] db.rs:18 remains the only legitimate Connection::open call in the shell
- [ ] cargo build clean, no new warnings introduced
- [ ] deploy faelight-shell succeeds
- [ ] No "attempt to write a readonly database" warning across one full interactive session
- [ ] Friday session continuity unaffected (session.rs refactor preserves behavior)
Phase A -- Safe sites
  G1  commands/mod.rs:1683  last builtin
  G2  commands/mod.rs:1710  save builtin
  G3  commands/mod.rs:1744  recall slot list
  G4  commands/mod.rs:1767  recall value fetch
  G5  commands/mod.rs:6314  fsh_identity_cmd
  G6  completion.rs:504     alias completion
Phase B -- Hot path
  G7  main.rs:546           shell_persist init
  G8  main.rs:1298          friday signal
  G9  main.rs:2234          insight query
  G10 main.rs:2264          pattern query
  G11 main.rs:2490          quote/prompt
  G12 main.rs:2644          alignment readout
Phase C -- session.rs
  G13 session.rs:25         session_state init
  G14 session.rs:94         session create
  G15 session.rs:206        session query
Phase D -- Verification
  G16 cargo build clean
  G17 deploy faelight-shell
  G18 one full session, zero read-only warnings
---

## Phase A Notes (2026-04-27)

Closed: G1 last, G2 save, G3+G4 recall, G5 fsh_identity_cmd
Deferred: G6 completion.rs alias completion

Why G6 deferred:
ForestHelper is held by rustyline Editor. Tried Option 2 (passing &ForestDb 
via lifetime parameter ForestHelper<'a>), reverted per one-shot rule when 
body restructure introduced brace mismatches. Architectural finding: G6 
likely requires Arc<ForestDb> wrapper across call sites. Defer to Phase B/C 
where main.rs is touched.

Phase A user-visible win: read-only WAL warning eliminated from interactive 
sessions. 5 of 16 connection sites unified. 5 clean commits.
*The forest grows with intention.*

---
id: 185
date: 2026-07-20
type: fix
title: "Friday's knowledge engine matches fsh's status string, not command stderr -- signature lessons can never legitimately fire"
status: in-progress
tags: [fsh, friday, knowledge, stderr, int-233, int-183]
---

## Vision
Friday's knowledge engine should do what it was built for (INT-233): when a command fails with a
known error, recognize the ERROR and offer the stored fix. Today it structurally cannot, because
it never sees the real error output. Fix the input, and the engine Christian already built starts
actually helping.

## The Problem
Discovered during INT-183 (2026-07-20). The post-failure lesson lookup (exec.rs, INT-233 block)
matches against `error_msg`, which for an EXTERNAL command is fsh's OWN CommandResult::Error status
string -- e.g. "exited 1 -- general error", "misuse of shell builtin". It is NOT the command's
stderr. `grep` confirms exec.rs never captures external stderr into CommandResult::Error.

Consequence: the knowledge_entries whose `error_signature` is a real error fingerprint
(error[E0716], error[E0277], "nixos-rebuild error: collision") can NEVER legitimately match an
external command, because the string they'd match against ("exited 1 -- general error") never
contains the fingerprint. Proof: those lessons' success_count -- rust_e0716_temporary=0,
rust_e0277_missing_debug=0. They have never fired correctly. Before INT-183 they fired only as
FALSE positives (the generic word "error" in "exited 1 -- general error" substring-matched
"error[E0716]"). INT-183 correctly silenced those false fires -- which means the signature lessons
are now correctly dormant, but STILL cannot do their intended job until this is fixed.

The deeper cost (why this matters down the road): the knowledge engine is designed to LEARN --
you solve an error once, Friday remembers the fix. As more lessons accumulate, a working engine
gets more helpful; a blind one just accumulates more never-matching (or mis-matching) entries. The
feature Christian built is architecturally prevented from paying off, and the problem grows with
the lesson count, not shrinks.

## The Solution
Capture external command stderr and make it available to the knowledge lookup, so a real
"error[E0716]" (or any signature) actually reaches the matcher.
- Recon FIRST: how external commands are spawned (exec.rs) and where their output goes. Determine
  whether stderr is currently streamed straight to the terminal (likely) and how to also capture
  it for the failure path WITHOUT breaking live output (the user must still see errors in real
  time -- tee, not swallow).
- Feed the captured stderr (not just the exit-status string) into the INT-233 lookup's error text,
  so INT-183's Branch 1 (signature-present) can match real fingerprints.
- Watch scope: capturing all stderr for every command has cost/complexity. Likely only need it on
  the FAILURE path, and possibly only for a bounded tail of stderr.
- This UNBLOCKS INT-183's Branch 1: once real stderr reaches the matcher, a genuine error[E0716]
  from `cargo build` will fire rust_e0716_temporary -- the live positive control INT-183 could not
  demonstrate.

## Success Criteria
- [x] Recon: how external commands run in exec.rs, where stdout/stderr go, whether stderr is
      captured anywhere. Documented -- confirm the current "error_msg = status string, not stderr"
      finding at the code level.
      <!-- DONE. The generic external path is run_external (commands/mod.rs:7849): spawned
      `sh -c line` with .stderr(Stdio::inherit()) -- child stderr went straight to the terminal,
      fsh never saw it, so postexec built error_msg from the exit code ("exited N"). exec.rs
      execute_with_context (:562) runs postexec synchronously right after dispatch, so last_stderr
      is unambiguously this command's. Confirmed at code level. -->
- [x] External command stderr is captured on the failure path AND still shown to the user live
      (tee semantics -- no swallowing, no double-printing). Demonstrated.
      <!-- DONE (commit pending). run_external now spawns with .stderr(Stdio::piped()) + a THREADED
      TEE: a thread reads child stderr in a 4096-byte loop, writing each chunk to the real
      std::io::stderr() LIVE (flushed) AND into a capture buffer; child.wait() runs concurrently
      (no pipe-fill deadlock). stdin/stdout stay inherited (normal output + interactive programs
      unaffected). NO double-print: CommandResult::Error keeps its short "exited N" display string
      (main.rs prints that once); the captured stderr rides a SEPARATE channel (shell_state
      last_stderr), not the display payload. Proven on metal gen 410: errors show live. -->
- [x] The captured stderr feeds the INT-233 knowledge lookup's error text.
      <!-- DONE. run_external ALWAYS writes shell_state.last_stderr (empty on success, so never
      stale for an external cmd) via INSERT OR REPLACE. postexec's INT-233 error_msg now prefers
      last_stderr (when non-empty) over the CommandResult::Error status string, falling back to the
      payload for builtins. So the two-branch matcher finally sees REAL stderr. -->
- [x] LIVE POSITIVE CONTROL (the one INT-183 could not do): a real `error[E0716]` fires
      rust_e0716_temporary on the deployed binary. Demonstrated on metal.
      <!-- DONE, deployed gen 410. By hand at the real REPL:
      `sh -c 'echo "error[E0716]: temporary value dropped while borrowed" >&2; exit 1'`
      -> "Friday knows this (99% confidence): Assign temporary to named let binding..."
      (rust_e0716_temporary). This FIRED -- the exact positive control 183 could not demonstrate
      (183's gate 4 was ticked with the honest caveat that the stderr gap blocked it; 185 closes
      it). Fires via branch 1 (signature INSTR finds error[e0716] in the now-real stderr). NOTE:
      used the reliable echo-to-stderr fingerprint rather than hand-crafting a real rustc E0716
      (which fought us) -- same code path, deterministic. -->
- [x] No regression to INT-183's relevance gating: the false specimens STILL stay silent.
      <!-- DONE, deployed gen 410, by hand at the real REPL: `core intent cancel 999` and
      `sqlite3 /nonexistent/nope.db "SELECT 1;"` both SILENT (no knowledge hint).
      ★ THIS GATE EXPOSED A REAL LATENT BUG that 185 caused and had to fix (recorded honestly):
      feeding REAL stderr surfaced richer text than 183's thin status strings ever contained. The
      clap error's resolved path /run/current-system/sw/bin/core tokenized into run/current/system/bin;
      the token "current" then matched INSIDE "concurrent" in statedb_wal_mode's description via
      branch 2's substring `descr.contains(token)` -- giving a false 2nd hit ("current"+"required")
      and firing a 99% WAL hint at a clap arg error. Instrumented the real debug binary to catch it
      (search_tokens trace showed WAL hits=2). FIX A (word-boundary): branch 2 now splits descr into
      whole words (same split rule as the token builder) and requires exact word membership, not
      substring. "current" != "concurrent" -> WAL drops to 1 hit -> silent. Verified all 13
      signature-less descriptions: none relied on substring matching, so no true positive lost.
      185 is inseparably the tee + this fix -- real stderr is only safe once matching is word-exact. -->
- [x] fsh still boots/deploys; fsh-test green on the deployed binary.
      <!-- fsh boots + deployed gen 410 (reload picked up new binary, banner shown). fsh-test
      97/97 PASSED on the deployed binary. -->
- [x] Each gate carries evidence per INT-158.
      <!-- Evidence in each comment above: run_external tee + last_stderr stash, postexec error_msg
      change, Fix A word-boundary; deployed gen 410; by-hand positive control + regression proofs. -->

## The Rule
"A knowledge engine that cannot see the error cannot know the fix. INT-183 stopped it lying;
INT-185 lets it finally tell the truth -- feed it the real stderr and the lessons Christian wrote
start earning their keep." 🌲

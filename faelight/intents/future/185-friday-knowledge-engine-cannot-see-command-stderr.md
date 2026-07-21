---
id: 185
date: 2026-07-20
type: fix
title: "Friday's knowledge engine matches fsh's status string, not command stderr -- signature lessons can never legitimately fire"
status: planned
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
- [ ] Recon: how external commands run in exec.rs, where stdout/stderr go, whether stderr is
      captured anywhere. Documented -- confirm the current "error_msg = status string, not stderr"
      finding at the code level.
- [ ] External command stderr is captured on the failure path AND still shown to the user live
      (tee semantics -- no swallowing, no double-printing). Demonstrated.
- [ ] The captured stderr feeds the INT-233 knowledge lookup's error text.
- [ ] LIVE POSITIVE CONTROL (the one INT-183 could not do): a real `error[E0716]` from an actual
      failing `cargo build` fires rust_e0716_temporary on the deployed binary. Demonstrated on metal.
- [ ] No regression to INT-183's relevance gating: the false specimens (sqlite bad-path, clap
      cancel) STILL stay silent -- real stderr must not reintroduce the loose-match noise.
- [ ] fsh still boots/deploys; fsh-test green on the deployed binary.
- [ ] Each gate carries evidence per INT-158.

## The Rule
"A knowledge engine that cannot see the error cannot know the fix. INT-183 stopped it lying;
INT-185 lets it finally tell the truth -- feed it the real stderr and the lessons Christian wrote
start earning their keep." 🌲

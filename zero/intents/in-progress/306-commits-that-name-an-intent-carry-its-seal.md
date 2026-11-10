---
id: 306
date: 2026-11-10
type: future
title: "Commits that name an intent carry its seal"
status: in-progress
tags: [seals, intent, git]
---

## Vision
A commit whose message names an intent also names the reviewed plan that produced
it, and the plan text is still there to re-check. The aim is a guard that refuses
a message carrying `Intent: INT-xxx` with no `Seal:` of 16 hex, then, once the
commit exists, checks that a note under refs/notes/seals hashes to that seal.
Where the guard lives, how it sees the note, and which commits are exempt are
rulings in the gates. Nothing is built before those rulings.

## The Problem
INT-295 shipped commits that name the intent and carry no seal. F-0117
(zero/intents/findings/F-0117.md, found 2026-11-09, severity low) says a36327cc
through 200f5cbd carry `Intent:` and no `Seal:` trailer and no note under
refs/notes/seals. The G7 ruling on that intent was to seal G7, file the finding,
and not rewrite history. The close after that range is the other shape.
d69ff1ec, subject "INT-295: complete; nsh 5.3.12, nsh-test 2.0.15, zero-doctor
0.1.4", carries `Intent: INT-295` and no `Seal:`. That is the cicomplete version
bump. Nothing on the commit path looks at either trailer.

.githooks holds pre-commit, pre-push, lib/nsh-test-gate.sh and README.md. There
is no commit-msg hook and no post-commit hook. pre-commit and pre-push each exec
zero-gate, from target/debug, then target/release, then PATH, and exit 1 if none
is there (.githooks/pre-commit, .githooks/pre-push). An absent gate is not a
pass. core.hooksPath is .githooks; .git/hooks is not used.

zero-gate (zero/tools/zero-gate/src/main.rs) answers pre-commit, pre-push and
all. pre-commit runs secrets, gitleaks, formatting and retired spawns. pre-push
runs .githooks/lib/nsh-test-gate.sh, then cargo test -p core. No stage reads an
Intent or Seal trailer. The crate's only dependency is libc. Cargo.toml says it
deliberately does not depend on zero-core: a gate that needs an intent ledger to
run cargo fmt is carrying a house to hold a door open. .githooks/README.md still
says pre-commit is "secrets, risk tiers, formatting" and pre-push is "the shell
test suite". The binary's --help is the list above. Risk tiers are not a stage.

core intent trace re-checks a seal in one place. seal_digest
(zero/engine/src/domains/intent/trace.rs:227) is the first 16 hex of sha256 over
the plan bytes. check_seal (trace.rs:253) answers Intact when the note hashes to
the Seal trailer, Broken when it hashes to something else, NoNote when the
trailer is present and the note is absent, Legacy when there is no Seal and a
Fingerprint trailer is present, and NoSeal when neither trailer is present.
check_commit_seal (trace.rs:442) reads the Seal and Fingerprint trailers and the
blob from `git notes --ref=seals list`, then calls check_seal. trace_commit
(trace.rs:501) prints that verdict. A note is stored against a commit object, so
it cannot be added until that object exists. commit-msg runs on the message file
before the object exists, so it can read the trailer and cannot read the note.
post-commit runs after the object exists; git does not abort the commit when
that hook exits non-zero, so it can warn and cannot refuse. pre-push can refuse
the push.

The stored note has to be the plan bytes. INT-266 lines 67-68 record
`git notes --ref=seals add --no-stripspace -F`. F-0023 and F-0063 record that
`git notes add -F` runs stripspace unless that is turned off, so the stored note
hashes to a different value and trace reports Broken. Later commits store the
blob with hash-object -w and notes add -C.

F-0116 (found by INT-303 on 2026-11-07, severity low, at trace.rs:532) is the
other open hole. trace_seal asks seal_commits (trace.rs:467) which commits carry
that Seal. seal_commits feeds
`git log --format=%h%x09%(trailers:key=Seal,valueonly)` through parse_fixes
(trace.rs:79). parse_fixes keeps only values that is_finding_id accepts
(trace.rs:74): F- and exactly four digits. A 16-hex seal is not a finding id, so
it is dropped, seal_commits returns nothing, and trace_seal says the value is no
seal on HEAD and traces it as a commit. trace_commit does not take that path, so
the same seal reads intact when named by its commit. Whether this intent fixes
that is G4.

## The Solution
No home and no exemption are chosen here. The behaviour under discussion is the
one F-0117 names: refuse a commit whose message carries `Intent: INT-xxx` and
does not carry `Seal:` as 16 hex, then check, once the commit exists, that a
note under refs/notes/seals hashes to that seal. Each gate is a question or a
step for Christian to rule. A ruling is a RULED line in this file, dated, in his
words. Declining a gate, with the reason, closes it. No code is written before
G1, G2 and G3 have those lines.

## Success Criteria
- [x] G0 The recon in The Problem is read against the tree and either stands or
      is corrected here before any other gate. Re-read: .githooks has no
      commit-msg and no post-commit; zero-gate's stages are pre-commit, pre-push
      and all, and none reads a trailer; check_seal is the one verdict; a note
      cannot be added until the commit object exists; post-commit cannot refuse;
      d69ff1ec carries Intent and no Seal; F-0116's lookup goes through
      parse_fixes and drops a 16-hex value. Proof: a dated line in this file
      saying the recon stands, or the correction.
<!-- evidence: 2026-11-10. Re-read against 8691e49f. The recon stands after one citation. .githooks holds pre-commit, pre-push, lib/nsh-test-gate.sh and README.md; no commit-msg and no post-commit. pre-commit and pre-push each exec an executable at target/debug/zero-gate, then target/release/zero-gate, then zero-gate on PATH, and exit 1 if none is there. zero-gate answers pre-commit (secrets, gitleaks, formatting, retired spawns), pre-push (.githooks/lib/nsh-test-gate.sh, then cargo test -p core) and all; no stage reads an Intent or Seal trailer. Cargo.toml depends on libc only. --help is that list. README.md still says pre-commit is secrets, risk tiers, formatting and pre-push is the shell test suite. check_seal is trace.rs:253. seal_digest is the function at trace.rs:227; the filed text said 226, which is the doc comment, and that citation is corrected above. check_commit_seal is 442, trace_commit 501, trace_seal 532, seal_commits 467, parse_fixes 79, is_finding_id 74. parse_fixes returns only values is_finding_id accepts, F- and four digits, so a 16-hex seal is dropped. d69ff1ec carries Intent: INT-295 and no Seal. INT-266 line 67 records git notes --ref=seals add --no-stripspace -F. A note is stored on the commit object, so it cannot be added until that object exists, and post-commit cannot refuse. core.hooksPath is not a file in the tree; .githooks/README.md says to set it to .githooks. -->
- [x] G1 Where the guard lives. Christian rules one home, or a split, and why.
      The options left open: a script at .githooks/commit-msg; a new zero-gate
      stage, knowing the crate does not depend on zero-core; a core intent
      subcommand that a hook calls; another place he names. The ruling says what
      that home can see when it runs, and what it cannot. No code before this
      line exists. Proof: the RULED line.
RULED 2026-11-10 (Christian, accepted Jarvis's recommendation: "i accept"): the trailer check lives in a new .githooks/commit-msg script that refuses a message carrying Intent: INT-xxx without Seal: of 16 hex; the note check lives in core intent (a subcommand the hook calls), so zero-gate stays free of the intent ledger. commit-msg sees the message only; core intent sees the commit and refs/notes/seals.
- [x] G2 How the note is checked. One check cannot both refuse the message and
      hash the note, because the note can be added only after the commit exists.
      Christian rules the split. One shape to accept, change or reject:
      commit-msg refuses a message that carries Intent: and no Seal: of 16 hex,
      and a later check requires the note under refs/notes/seals and requires
      its sha256 to match the seal. post-commit can only warn. pre-push can
      refuse. The ruling names which of those, or another, does the hash, and
      names the command that stores the note. stripspace makes a true plan read
      Broken (F-0023, F-0063). The forms already in use are --no-stripspace -F,
      and hash-object -w then notes add -C. The hash stays seal_digest; the
      later check asks that verdict (check_seal, as core intent trace and
      core intent seals already do) rather than a second implementation.
      Proof: the RULED line.
RULED 2026-11-10 (Christian, accepted Jarvis's recommendation: "i accept"): pre-push does the hash, calling the core intent check that uses check_seal (no second implementation); a missing or broken note refuses the push. Notes are stored with `git notes --ref=seals add --no-stripspace -F <plan> <commit>`.
- [x] G3 The escape hatch for a release commit. d69ff1ec carries Intent: and no
      Seal:, and it is the cicomplete version bump. Christian rules whether that
      class is exempt, and how an exemption is named so a skipped check cannot
      look like a passed one. If nothing is exempt, he rules what cicomplete
      must start putting on the bump commit. a36327cc through 200f5cbd and
      d69ff1ec stay as they are either way. Proof: the RULED line.
RULED 2026-11-10 (Christian, accepted Jarvis's recommendation: "i accept"): a release commit is exempt only when it carries a `Release:` trailer naming the version bump; cicomplete adds it to its bump commit; the exemption is printed by the check so a skip never reads as a pass. Earlier commits stay as they are.
- [ ] G4 Whether to also fix F-0116 (trace by seal value). The cause is in The
      Problem. Christian rules: fix it in this intent, or leave it open. If it
      is fixed here, a failing test comes first, under its own plan Seal, and
      the commit that turns it green carries Fixes: F-0116. If it is left, this
      gate says so and F-0116 stays open. Proof: the RULED line, and, if the fix
      landed, core intent trace of a seal that trace by commit already reads as
      intact.
RULED 2026-11-10 (Christian, accepted Jarvis's recommendation: "i accept"): fix F-0116 in this intent, failing test first under its own Seal, the green commit carries Fixes: F-0116.
- [x] G5 Red, then the guard, only after G1, G2 and G3 are ruled. Shown failing
      first: a message that carries Intent: and no Seal:; a Seal whose note is
      missing; a note that hashes to something else. Then the ruled exemption
      passes, and a message of the same shape that is not exempt fails. The
      suite is the one the ruled home already uses: cargo test -p zero-gate,
      cargo test -p core, or an nsh-test row. Proof: the red output, then the
      green output, each under its plan Seal.
<!-- evidence: 2026-11-10. Red commit 117770c6, Seal 9aed05fabf20e79d, the note under refs/notes/seals hashes to it. cargo test -p core g5_ -- --ignored --test-threads=1 exited 101: 1 passed, 10 failed, 0.06s. The pass is g5_commit_message_gate_leaves_a_message_with_no_intent, because the stub answers untouched. The ten failures are the refusal, the 16-hex seal, a seal that is not 16 lowercase hex, the Release exemption, the exempt (release) line, the range check (Intact, NoSeal, NoNote, Broken, Release-exempt, no-Intent ignored), a missing range, the commit-msg script (REFUSE: commit-msg is not an executable hook), and pre-push (does not call core intent seals check). cargo test -p core on that commit: 254 passed, 0 failed, 11 ignored. Green, this commit: the ignore is gone. cargo test -p core: 265 passed, 0 failed, 0 ignored. cargo test -p zero-gate: 4 passed. Scratch commit, hooks path holding only commit-msg: Intent: INT-306 and no Seal refused, exit 1, "refused: this message carries Intent: INT-306 and no Seal: of 16 lowercase hex." and the lines that say to add Seal: or Release:. The same repo with Release: nsh 1.0.0 printed "commit-msg: release exemption used (Release: nsh 1.0.0)" and exited 0. Before this commit, core intent seals check 8691e49f..HEAD exited 0: 43ecc66e intact, 117770c6 intact. core intent seals check a36327cc^..200f5cbd exited 1, NoSeal: a1389f57 4ac3356c 9be9d059 6b8722bc 305c8245 7ed2fc90 0fa00a33 9531c1ee 250a8e35 200f5cbd. cicomplete does not commit; it prints the suggested message with Intent: and Release:. pre-push reads the ref lines before zero-gate and runs the gate with stdin from /dev/null. .githooks/test-pre-push.sh gives the refs to a gate that reads stdin; the gate sees nothing and the seal check still sees both refs. -->
- [ ] G6 Doors, after any ship. cargo test -p novashell passes. nsh-test passes
      after `cargo build -p novashell -p nsh-test`. core intent validate reports
      every intent valid. d reports 0 failed. A commit this intent makes from a
      reviewed plan carries Intent: INT-306 and Seal:, and the note under
      refs/notes/seals hashes to that seal. core intent trace of one such commit
      reads the seal intact. Proof: the door output.
- [ ] G7 Optional. Christian floated tying seals to cistart. Today cistart is
      core intent start (zero/engine/src/domains/intent/mod.rs, start, around
      line 905): it moves the file from future/ to in-progress/, sets focus, and
      prints the id, the title and that the intent is focused. It does not
      mention Seal. He rules whether cistart also prints a reminder that commits
      for this intent need a Seal. Declining it, with the reason, closes the
      gate. If he wants the line, the wording is his and it is shown before it
      is added. Proof: the RULED line, and the printed reminder if one was added.
RULED 2026-11-10 (Christian, accepted Jarvis's recommendation: "i accept"): cistart prints a one-line seal reminder; the wording is Christian's and is shown to him before it is added.

## Out of scope
- Rewriting a36327cc through 200f5cbd, d69ff1ec, or any other commit on main.
- A second seal hash. seal_digest stays the one definition.
- Editing AGENTS.md. The Seal rule is already written there (INT-266). Any new
  sentence waits on its own ruling and a reviewed plan.
- A guard that writes the plan. It checks a trailer and a note.

## Open for Christian
- G7, the wording of the one-line seal reminder cistart prints. It is Christian's and is shown to him before it is added.

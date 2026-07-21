---
id: 184
date: 2026-07-20
type: improvement
title: "git commit: auto-retry after rustfmt reformats (the commit-doesn't-land trap)"
status: planned
tags: [fsh, git, workflow, rustfmt, tooling]
---

## Vision
Committing Rust changes lands on the first try. The rustfmt pre-commit hook reformatting a file
should not silently cost you the commit.

## The Problem
OBSERVED four times 2026-07-20: `git commit` on a .rs change triggers the rustfmt pre-commit
hook, which REFORMATS the file and then FAILS the commit ("files were modified by this hook").
The commit does NOT land. If you then proceed (e.g. `dep`), you build/deploy from an UNCOMMITTED
tree -- a dirty-tree advisory, and work that looks committed but is not. The fix each time was
manual: re-add the now-reformatted file and commit again. It works, but it is a silent trap that
bit us repeatedly, and it is exactly the kind of papercut that wastes attention on every Rust
commit.

## The Solution
Make the commit flow rustfmt-aware. When a commit fails specifically because rustfmt reformatted
tracked files (hook id rustfmt, "files were modified by this hook"), automatically re-stage the
reformatted files and re-run the commit ONCE, then report the result. Options (choose after
recon):
- A wrapper/alias around commit (the `gp`/commit path) that detects the rustfmt-modified failure
  and retries.
- Or configure the hook to stage its own reformats (if the pre-commit framework supports it).
Guardrails: retry ONCE only (avoid loops); only auto-retry for the rustfmt-reformatted case, not
for real failures (ripsecrets/risk-gate failures must still stop the commit and surface loudly);
never auto-commit something the human did not already ask to commit.

## Success Criteria
- [ ] Recon: how the commit path and the pre-commit hooks are wired (the rustfmt hook, the
      commit alias/flow). Documented.
- [ ] The rustfmt-reformatted-and-failed case is detected specifically (not confused with real
      hook failures like ripsecrets/risk-gate).
- [ ] On that case only, the reformatted files are re-staged and the commit retried ONCE; the
      commit lands. Demonstrated on a real .rs change that rustfmt reformats.
- [ ] Real failures (secrets, risk-gate) still STOP the commit and surface -- proven, no silent
      swallow.
- [ ] No loops (retry is once), no committing of unrequested content.
- [ ] Each gate carries evidence per INT-158.

## The Rule
"A formatter fixing your whitespace should never cost you the commit. Detect the one benign
failure, re-add, retry once -- and let every real failure still shout." 🌲

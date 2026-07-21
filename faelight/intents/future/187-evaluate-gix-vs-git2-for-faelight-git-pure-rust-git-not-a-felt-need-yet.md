---
id: 187
date: 2026-07-21
type: arch
title: "evaluate gix vs git2 for faelight-git -- pure-Rust git, NOT a felt need yet"
status: planned
tags: [fsh, faelight-git, gix, git2, libgit2, evaluate]
---

## Vision
Evaluate migrating faelight-git from git2 (libgit2 C bindings) to gix (gitoxide, pure-Rust git). Pure Rust
means no libgit2 C dependency -- simpler builds, no C toolchain surface, potentially better performance
and a more Rust-native API.

## HONEST FRAMING -- this is NOT a felt need
git2 0.18 WORKS TODAY. faelight-git uses it (repo.rs, the libgit2 commit path that BYPASSES pre-commit
hooks -- discovered in INT-184). Nothing is broken. This intent is a "nicer / more-Rust-native" want, not
a problem being solved. It is filed as a tracked destination (from Christian's crate-stack vision,
2026-07-21), not as urgent work. It may correctly sit in future/ for a long time, or resolve to "keep git2."

## THE TRAP TO AVOID (INT-169's rule applies here too)
NO BIG-BANG REWRITE OF WORKING CODE. faelight-git is LOAD-BEARING -- it is Christian's commit path (fg
commit, gp). A migration that half-works leaves the commit workflow broken. If this proceeds, it is
incremental, command by command, with git2 live until each gix path passes the same behavior.

## Gate zero -- the honest question
Name what gix GIVES that git2 does not, concretely: pure-Rust build simplicity? a specific feature? measured
performance? If the only answer is "it is more Rust-native," that is a real-but-LOW-priority reason -- valid
to file, not valid to prioritize over felt-need work. If nothing concrete, CANCEL / keep git2.

## Success Criteria
- [ ] Verify-first: inventory every git2 use in faelight-git (repo.rs commit path, status, log, diff, etc.)
      -- the real migration surface.
- [ ] Gate zero: what does gix concretely give? Documented, or cancel.
- [ ] gix maturity check: does gix cover every operation faelight-git needs at production quality? (gitoxide
      is not 100% feature-complete vs libgit2 -- verify the specific ops are supported before committing.)
- [ ] If it proceeds: incremental, one operation at a time, git2 live until each gix path passes the same
      behavior. faelight-git commit/push must work at EVERY step.
- [ ] Each gate carries evidence per INT-158.

## The Rule
"git2 works. gix is nicer. 'Nicer' is a real reason and a low priority -- this waits behind everything with
a felt need, and it may correctly never move." 🌲

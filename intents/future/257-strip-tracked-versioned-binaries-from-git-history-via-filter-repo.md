---
id: 257
date: 2026-04-28
type: fix
title: "Strip Tracked Versioned Binaries from Git History via filter-repo"
status: planned
tags: [fix, git, hygiene, repo-size, filter-repo, history-rewrite, infrastructure]
version: TBD
---

## Vision

The 0-core repository carries 100 versioned binaries in its tracked
history (~388 MB packed) from before `bin/*@*` was added to .gitignore.
Commit `94ee3770` (2026-04-28) untracked them going forward, but the
history still contains every binary blob.

This intent rewrites git history to remove all `bin/*@*` blobs from past
commits, dramatically shrinking the repository.

The work itself is low-risk-but-disruptive: history rewrite means every
commit hash changes upstream of the rewrite point. Coordination matters
more than the technical execution.

## Why Now (vs. immediately)

Deferred from the 2026-04-28 cleanup session for two reasons:

1. **Disruption cost.** History rewrite invalidates every commit hash.
   Anyone with a clone (currently just one person) needs to re-clone or
   force-pull. Tools that reference commit hashes (deploy versioned
   binaries, release tags, intent links) need re-verification.

2. **Scoped focus.** That session was shipping daily-driver friction
   fixes. Doing both was scope creep risk.

The right time is during a cleanup window, not in the middle of feature
work.

## Approach

### Phase 1: Verify scope
- Run `git filter-repo --analyze` to understand current repo composition
- Confirm the candidate paths to remove (`bin/*@*` only, NOT `bin/`
  symlinks or other contents)
- Estimate post-rewrite repo size

### Phase 2: Pre-rewrite backup
- Tag current HEAD as `pre-filter-repo-2026-XX-XX`
- Push tag to GitHub for emergency restore reference
- Clone the repo to a separate location as a frozen snapshot

### Phase 3: Execute rewrite
- Use `git filter-repo --path-glob 'bin/*@*' --invert-paths`
- Verify resulting tree on a fresh clone
- Confirm no surprise removals (only versioned binaries gone, everything
  else intact)
- Verify recent commits still reachable by their NEW hashes

### Phase 4: Force-push and announce
- `git push --force-with-lease origin main`
- Update any external references (if any) to new commit hashes
- Re-clone local working copy from rewritten remote

### Phase 5: Verify health
- Run `core doctor` — all 22+ checks pass on fresh clone
- Verify deploy still functions (versioned binaries on disk are local;
  their git tracking is gone)
- Confirm intent ledger references are consistent
- Update VERSION + CHANGELOG noting the history rewrite

## Hard Dependencies

- `git filter-repo` installed (pacman: `git-filter-repo` or paru)
- Clean working tree before starting
- No active CI/CD that would race the force-push
- Confirmed single-clone environment (no team coordination needed beyond self)

## Success Criteria

- [ ] Repo size reduced significantly (target: <100 MB packed, from 388 MB)
- [ ] Pre-rewrite tag pushed and accessible
- [ ] All recent commits accessible at new hashes
- [ ] No source files lost (only versioned binaries removed)
- [ ] `core doctor` 100% on freshly cloned repo
- [ ] Deploy works (binaries still on disk, just not tracked)
- [ ] Intent ledger entries verified intact
- [ ] At least one production deploy run after rewrite to verify pipeline
- [ ] CHANGELOG updated with rewrite note

## Risks and Mitigations

### Risk 1: Force-push catastrophe
**Mitigation**: Pre-rewrite tag pushed first. Snapshot clone in safe
location. Force-push uses `--force-with-lease` not `--force`.

### Risk 2: Deploy breakage
**Mitigation**: Versioned binaries stay on local disk. Deploy reads from
disk, not git. Verify with a small deploy after rewrite.

### Risk 3: Intent reference drift
**Mitigation**: Intent files reference commit hashes only in narrative
text (not enforced). Update prominently-referenced hashes after rewrite.

### Risk 4: Surprise data loss
**Mitigation**: Phase 1 analysis explicitly checks what will be removed.
Phase 2 backup is the safety net.

## Scope

### In scope
- Removing `bin/*@*` blobs from all historical commits
- Repo size reduction
- History rewrite via git filter-repo
- Coordinated force-push

### Out of scope
- Removing other large historical artifacts (separate analysis)
- Migrating to LFS for any large files (no current need)
- Switching to a different git host
- Compressing existing source code or asset history

## Gate Check
⬜ Not started

---

*"The repository should be lean.
The history should be honest.
Cleanup is its own kind of work."* 🌲

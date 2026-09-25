---
id: 111
date: 2026-07-02
type: future
title: "bump-versions lightweight per-tool versioning"
status: complete
tags: [versioning, release, cicomplete]
---

## Why
`bump-versions` is SUGGESTION THEATER. It reads each tool's Cargo.toml, prints what a
patch/minor bump WOULD be -- but has NO write path (verified: bump_versions_cmd in
faelight-shell only builds a display String; even the `apply` branch just prints usage).
cicomplete (engine intent/mod.rs) does the same: reads versions, prints "suggested",
says "Run: bump-versions to apply" -- pointing at a command that cannot apply.
So faelight-shell sat at 2.5.0 through INT-100/101 (and 104/082/116 this session) not
because bumping is heavy -- because BUMPING WAS NEVER IMPLEMENTED. The gap is a missing
writer + a closed loop, not a new tool.

## The axiom that resolves the confusion (gen vs version)
TWO ORTHOGONAL AXES -- never linked in the data, only felt linked because nothing bumped:
- TOOL VERSION = "this artifact's code changed" -> lives in the tool's Cargo.toml,
  bumped at cicomplete when its code changed. ZERO connection to generation count.
- FOREST GENERATION = "system rebuild count" (gen 300, 999...) -> disposable counter,
  resets on fresh install, means nothing about tool maturity.
Gen 999 is a non-event. A tool bumps ONCE per meaningful code change, no matter how many
generations were burned testing it. The bump attaches to the CHANGE, not the testing.

## Architecture -- who owns what (settled by recon, no new tool)
1. bump-versions (faelight-shell, commands/mod.rs:12432) = THE WRITER.
   Already reads Cargo.toml + does semver math. ADD the missing write path:
   `bump-versions patch|minor|major <tool>` edits the tool's Cargo.toml version line in
   place (count-asserted). Turn the liar into a doer.
2. cicomplete (engine domains/intent/mod.rs:~960) = THE TRIGGER.
   Already detects which tools an intent's commits touched. Replace the dead
   "Run: bump-versions to apply" print with an interactive prompt (see Interaction).
3. faelight-docs = THE CHANGELOG owner.
   Already generates version tables + Keep-a-Changelog sections from version metadata
   (toolgen.rs). Once versions MOVE, its changelogs become accurate automatically.
   (Optional stretch: bump appends a per-intent changelog line from the intent title.)
4. faelight-update = EXCLUDED. Wrong axis -- it updates the RUNNING SYSTEM, not dev-time
   source versions. Do not involve it.

## Semver rules (type -> proposed level)
Intent `type:` proposes; human confirms/overrides at the prompt.
- bugfix / polish / infrastructure  -> PATCH   (x.y.Z+1)
- feature                            -> MINOR   (x.Y+1.0)
- breaking change                    -> MAJOR   (X+1.0.0)  [type has no "breaking";
                                                            reached only by human override]

## Interaction (prompt at cicomplete -- Christian's design)
At cicomplete, for each touched tool, PROPOSE from type and ASK:
  "bump faelight-shell 2.5.0 -> 2.5.1 (patch)?  [patch / minor / major / skip]"
- Level pre-filled from the intent type mapping (common case = one keypress to accept).
- Human can escalate to MAJOR (the only path to major -- covers breaking changes the
  type field cannot express) or downgrade / skip.
- NOTHING bumps silently. Demonstrated, human-authorized, forest-aligned.
- Fires at cicomplete because that is when the change is fresh (fix vs feature).

## Phases
Phase 0 -- Recon (done): source-of-truth = each tool Cargo.toml `version` line. The
  8-tool list already lives in bump_versions_cmd; decide keep-hardcoded vs derive from
  Cargo workspace members.
Phase 1 -- Write path: `bump-versions patch|minor|major <tool>` edits the version line
  in place, count-asserted. Verify a real read-modify-write bump.
Phase 2 -- type->semver mapping function.
Phase 3 -- Wire cicomplete: detect touched tools -> propose from type -> interactive
  prompt -> call writer. Close the loop.
Phase 4 -- Verify faelight-docs changelog/version tables reflect moved versions.
Phase 5 -- Retroactive first use: bump faelight-shell / core / faelight-git for THIS
  session (INT-104 schema, INT-082 de-Arch, INT-116 Arch sweep) -> clean 1.0.0 baseline.

## Gates
- [x] bump-versions has a real WRITE path; demonstrated live (db-browse 1.0.0->1.0.1 round-trip on deployed binary)
- [x] type -> patch/minor mapping implemented (semver_level_for_type); major reachable via prompt override
- [~] cicomplete prompts per touched tool + applies -- demonstrates itself AT cicomplete 111 (the completion IS the proof)
- [x] faelight-docs reflects moved versions (READMEs regenerated with 3.0.0/3.2.0/4.2.0)
- [x] retroactive bumps applied: shell 3.0.0, core 3.2.0, git 4.2.0 (committed)
- [x] gen-vs-version axes documented (this intent body)

## CRITICAL LESSON (learned live 2026-07-06) -- version bump REQUIRES lock regen
A version bump changes a workspace member's Cargo.toml, but Cargo.lock still pins the OLD
version. The LOCKED devShell (`cargo check --workspace --locked`) then REFUSES to build:
  "error: cannot update the lock file ... because --locked was passed to prevent this"
This is the build system correctly catching manifest/lock DRIFT. If missed, it breaks the
DEPLOY (nixos-rebuild) -- worst possible timing (mid-release).

MANDATORY STEP after ANY version bump, BEFORE deploy:
  cd ~/0-core && cargo check -p <bumped-tool>     # BARE cargo, NOT via the locked devShell
This regenerates Cargo.lock to match the new version(s). Then the locked devShell + deploy
accept it (manifest and lock agree again). This is the SAME rule as "new crates need bare
cargo check before the locked devShell accepts them" -- applied to CHANGED VERSIONS too.

FOLLOW-UP (make the bump flow do this automatically -- future enhancement):
  bump-versions (and the cicomplete prompt) SHOULD, after writing a version, run the bare
  `cargo check -p <tool>` to sync the lock -- so a bump is never left in a deploy-breaking
  half-state. Until that is wired, the manual `cargo check` step is REQUIRED after bumps.
  Add a gate: "bump-versions regenerates Cargo.lock (or reminds to) after a write."

## Relationship
- PREREQUISITE for the Faelight OS 1.0.0 release: cannot cut 1.0.0 with versioning that
  is pure suggestion-theater. Do 111 -> then 1.0.0.
- Excludes faelight-update. Touches faelight-shell (writer), engine/intent (trigger),
  faelight-docs (changelog).

i---
id: 105
date: 2026-07-01
type: future
title: "fix paths.rs drift: realign canonical path module to NixOS-era flat structure"
status: complete
tags: [paths, drift, faelight-core, refactor]
---

## Why
paths.rs (rust-tools/faelight-core/src/paths.rs) is a leftover STRUCTURAL
AUTHORITY from the Arch-era "numbered gravity" design: it hardcodes 00-meta/,
01-registry/, 02-rules/, 04-runtime/. Those directories NO LONGER EXIST -- the
NixOS migration flattened them to meta/, registry/, policy/, runtime/. So paths.rs
now describes a repo that is gone, and every tool calling it (get-version, profile,
intent, faelight-git, faelight-hooks, doctor, ~24 files via
`faelight-core = { path = "../faelight-core" }`) is pointed at phantom paths.

PROVEN LIVE BUG: `get-version` prints "❌ Could not read system version" because
`paths::version_file()` -> 00-meta/VERSION -> does not exist.

Deeper problem: there are TWO competing structural authorities --
  (1) paths.rs's stale numbered map, and
  (2) the actual flat dirs on disk.
They disagree, and tools break in the gap. The fix is NOT "make paths.rs agree
with disk" as a one-off patch -- it is to stop paths.rs being a SECOND authority
at all, and move toward the v2 Nix tree (INT-061) as the SINGLE structural truth.

## Philosophy -- keep the 0-Core mythology, apply the Nix way
0-Core's founding idea was structural integrity made VISIBLE -- the structure
teaches itself, orderly, legible ("if you cannot point at the layer in the tree,
it is not 0-Core"). That principle stays. What retires is the Arch-era
IMPLEMENTATION (numbered dirs + a Rust module as the map). On NixOS the structural
authority should be DECLARATIVE and SINGLE-SOURCE:
  - The repo's shape (INT-061 v2 tree: nix/ + faelight/) is canonical.
  - Structure is defined ONCE; code READS that structure, never re-declares it.
  - No gap between "where files are" and "where code looks" -> fewer errors.
This is 0-Core's integrity thesis, expressed the Nix way.

## Scope of THIS intent (the honest first step)
Retire the Arch-era numbered-gravity thinking FROM THE CODE and make paths.rs a
FAITHFUL reflection of the real (NixOS-era) structure -- unbreaking the ~24
consumer tools. This is the foundation that makes INT-061's tree moves cheap
(once paths.rs is the single correct map, moving a dir is a one-line change there).

Confirmed real dirs on disk (numbered gravity ALREADY gone): meta/, registry/,
policy/, config/, runtime/, schema/, intents/, docs/, engine/, rust-tools/,
hosts/, modules/, profiles/, users/, pkgs/, labs/, tests/.

Old (stale) -> New (real) mapping to correct in paths.rs:
  meta_dir()      00-meta   -> meta
  registry_dir()  01-registry -> registry
  rules_dir()     02-rules  -> policy   (CONFIRM contents first; may split/remove)
  interfaces_dir() config    -> config  (already correct)
  runtime_dir()   04-runtime -> runtime
  target_dir()    04-runtime/target -> target (root-level now)
Plus fix the two #[cfg(test)] tests (they ASSERT the old numbered paths -- update
to assert the new real paths, or the suite fails).

## Explicitly NOT in this intent (separate, later)
- The 40+ files that HARDCODE paths as string literals (bypassing paths.rs)
  entirely -- routing those THROUGH paths.rs is a later consolidation.
- The v2 tree directory MOVES themselves -- that is INT-061.
- Nix-declared structure (structure defined in Nix, Rust reads it) -- the horizon
  vision; its own future intent once this foundation is clean.

## Approach (incremental, rebuild-after-each -- no big bang)
1. cistart 105. `d` before.
2. Read meta/, registry/, policy/, config/ contents to finalise the mapping
   (esp. 02-rules -> policy? and where hooks/security landed).
3. Rewrite paths.rs functions to the real dirs; retire numbered-gravity naming +
   comments. Update the two tests to assert real paths.
4. Build faelight-core alone: `cargo build -p faelight-core`.
5. Build the workspace so all ~24 consumers recompile against the fixed module.
6. PROVE: re-run get-version (must succeed now), profile, intent, doctor -- each
   reads REAL files, no "not found".
7. Deploy, commit, `d` after, cicomplete.

## Success criteria
- [ ] paths.rs contains NO numbered-gravity names (00-/01-/02-/04-).
- [ ] Every paths.rs function points at a directory that EXISTS on disk.
- [ ] `get-version` succeeds (was the proven failure).
- [ ] The ~24 consumer tools build + run against the corrected module.
- [ ] Tests updated to assert real paths; `cargo test -p faelight-core` green.
- [ ] Charter notes the bridge to INT-061 (v2 tree as single authority) + the
      later "route hardcoded strings through paths.rs" consolidation.

## Relationship to INT-061
105 is the FOUNDATION: it makes paths.rs the single CORRECT map. Then 061's v2
tree moves become cheap (move a dir = one-line change in paths.rs). Long-term, the
authority migrates from paths.rs into the Nix structure itself -- 0-Core integrity,
Nix-native.

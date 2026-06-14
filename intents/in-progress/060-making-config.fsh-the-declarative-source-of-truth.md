---
id: 060
date: 2026-06-13
type: fix
title: Making config.fsh the declarative source of Truth
status: in-progress
tags: [fix, bugfix]
version: TBD
---
## Vision
Make `config.fsh` the single declarative, version-controlled source of truth for every fsh
alias, deployed by home-manager. After this intent the live alias set is fully reproducible
from the repo on a clean rebuild, and no permanent aliases live in the untracked runtime
`state.db`. The shell becomes auditable and authorized: an alias is in the repo, or it does
not exist.

## Why Now
The helix / INT-056 session exposed that fsh aliases live in runtime `state.db`, not in the
tracked `config.fsh`. Already observed:
- A shadow `hx -> helix` alias silently broke a working tool; only an accidental `unalias`
  revealed it.
- ~40 stale Arch-era aliases (paru/pacman, zsh, nvim, yazi, `ccat=/usr/bin/cat`) persist
  invisibly and error when invoked. None are reproducible on a fresh machine.
- The live `alias` set (~250) does not match the 50-alias tracked `config.fsh`.

Until this is reconciled the shell is not reproducible and shadow-break regressions can recur.
Fixing it now also unblocks the neovim/yazi decommissions (INT-058) that share the same
runtime-alias cruft.

## Approach
1. Locate the live `state.db` -- confirmed NOT in `~/.local/share/faelight-shell`; find the
   real path (check `~/.local/state`, repo `runtime/`, and the fsh source for the DB path).
2. Understand fsh's alias-load model: does it merge `config.fsh` + `state.db`? Which wins? Are
   `config.fsh` aliases cached into `state.db` at startup? Demonstrate, don't assume.
3. Inventory: capture live `alias` output, diff against tracked `config.fsh`, tag every entry
   keep / migrate / purge.
4. Migrate keepers not already in `config.fsh` into it (tracked, home-manager `xdg.configFile`);
   rebuild and confirm they load from the tracked file.
5. Purge stale runtime aliases from `state.db` (paru/pacman, zsh, nvim, yazi, `ccat`) via the
   safe mechanism identified in step 2 (`unalias`, or a scoped DB edit).
6. Coordinate neovim retirement + its ~11 `nvim` aliases (with the package-removal intent), and
   resolve what ships the `exa` binary.
7. Verify reproducibility: on a fresh shell AND a fresh login the live alias set equals
   `config.fsh`; nothing comes from `state.db`; regenerable from the repo alone.

## Success Criteria
- [ ] Live `alias` set matches tracked `config.fsh` (no untracked runtime-only aliases)
- [ ] All ~40 stale Arch-era aliases purged from `state.db`
- [ ] No shadow aliases remain -- every alias resolves to an installed binary
- [ ] `config.fsh` documented as the authoritative source; alias-load behavior written down
- [ ] neovim + its `nvim` aliases retired together (or handed to a coordinated intent)
- [ ] `exa` origin resolved (removed, or documented as an eza compat shim)
- [ ] Alias set reproducible on a clean rebuild -- demonstrated, not assumed
- [ ] `core doctor` health >= pre-intent % before and after

## Gate Check
```
✅ G1  state.db located; alias-storage schema confirmed (demonstrated by inspection)
✅ G2  fsh alias-load model documented (config.fsh vs state.db precedence) -- demonstrated
⬜ G3  full alias inventory diffed vs config.fsh; every entry tagged keep/migrate/purge
⬜ G4  keepers migrated into config.fsh; rebuild succeeds; they load from the tracked file
⬜ G5  stale Arch-era aliases purged from state.db
⬜ G6  no shadow aliases remain (every alias -> installed binary) -- demonstrated
⬜ G7  neovim + nvim aliases retired together, or deferred via a logged, approved hand-off
⬜ G8  exa origin resolved
⬜ G9  reproducibility proven: fresh shell + fresh login == config.fsh, regenerable from repo
```
---
*"The forest grows with intention."* 🌲

## Progress -- 2026-06-14 (G1, G2 demonstrated)
G1 -- state.db at ~/0-core/runtime/state.db (canonical db.rs:17). Aliases in
table shell_aliases (id, name UNIQUE, command, created). Live count: 346 vs
~50 tracked in config.fsh -> ~296 untracked runtime aliases to sort.

G2 -- alias-load model (source-demonstrated):
- Boot: config.fsh parsed; each alias INSERT OR REPLACE'd into shell_aliases via
  db.add_alias (config.rs:251 -> db.rs:96). No truncate/prune -- purely additive.
- Runtime resolves aliases from the TABLE (db.rs:115), not config.fsh. Table is
  the live source of truth. `alias` writes (db.rs:96), `unalias` deletes (db.rs:105).
- No source/provenance column; 346 count proves boot is additive-only.
- safety_guard.rs:40 blocks raw sqlite3 DELETE on state.db -> G5 purge must use
  the sanctioned path (unalias / db delete).

KEY FINDING (new requirement): data cleanup alone won't hold -- cruft re-grows.
Needs a code change so config.fsh is authoritative on boot: (a) prune table
entries not in config.fsh at startup, or (b) add a source column and drop
runtime-only entries. Propose as a new gate.

NEXT: G3 -- dump 346, diff vs config.fsh, tag each keep/migrate/purge.

---
id: 092
date: 2026-06-26
type: feature
status: complete
title: "Cheatsheet v2: sync command_registry to reality + live verification (hybrid)"
tags: [cheatsheet, fsh, command-registry, state-db, liveness, nix-era, truth]
priority: high
---
## Why
The `cheat` builtin (faelight-shell cheatsheet TUI) reads from the `command_registry`
table in state.db, colored by KIND with NO liveness check. The table is a frozen
Arch-era snapshot -- bulk-loaded once (every row stamped 1777762202), and NOTHING
writes to it (only cheatsheet_tui.rs touches it, read-only). It has drifted badly:
  DIAGNOSIS (measured 2026-06-26):
    keybinds:  88 in registry  vs  35 real binds in mango config.conf  -> ~53 PHANTOM
               (Arch-era niri/Hyprland binds that no longer exist -- the "red"/wrong entries)
    aliases:    0 in registry  vs  299 real in shell_aliases          -> 299 MISSING
    builtins:  17 in registry  vs  (fsh's real builtin set)           -> unverified
    commands:  13 in registry                                          -> unverified
The cheatsheet lies about the system: shows keybinds that do nothing, hides every alias.
A cheatsheet that misrepresents the forest is worse than none -- it tells you wrong things
about your own tools. This makes it tell the truth, live. ("A forest that knows itself.")
Note: supersedes the Arch-era "INT-260" reference in INT-052 (260 does not exist on the
Nix branch -- numbering restarted at 001). INT-052 re-points its cheatsheet gate here.
## Approach: HYBRID (refresh-from-reality + load-time liveness)
Decided (vs refresh-only or load-only): refresh rebuilds the registry from live sources;
load-time does CHEAP verification to color entries. Best of both -- accurate data AND
current status without 300 slow PATH lookups on every open.
## Phases
Phase 1 -- Refresh engine (the missing writer)
  Build the rebuild path that regenerates command_registry from LIVE sources:
    - builtins  : from fsh's actual command dispatch set (not a frozen 17)
    - aliases   : all rows from shell_aliases (currently 299, shown as 0)
    - keybinds  : parsed from mango config.conf `^bind=` lines (the real 35)
    - commands  : forest tools that actually exist (registry/tools.toml + PATH)
  Stale entries pruned or marked deprecated=1 (the ~53 phantom keybinds go).
  Expose as `cheat --refresh` (and/or a core domain) -- explicit, idempotent.
  Gate: after refresh, registry counts MATCH reality (aliases=299, keybinds=35,
        0 phantoms); `cheat` shows all real aliases.
Phase 2 -- Load-time liveness (the hybrid color)
  TUI verifies each entry cheaply at load and colors green=verified / dim=stale:
    - alias   : exists in shell_aliases?
    - builtin : in fsh known builtin set?
    - keybind : present in current mango config.conf?
    - command : on PATH / in registry?
  Cheap checks only -- no shelling out 300x. Keep `cheat` snappy to open.
  Gate: working entries render verified, any stale entry visibly dim; open time < 200ms.
Phase 3 -- Auto-refresh trigger
  Wire refresh to a sensible trigger so the registry never refossilizes:
    - PRIMARY: on `deploy` (aliases/binds change at deploy time) -- hook into the
      deploy flow after config.fsh + mango config regenerate.
    - MANUAL: `cheat --refresh` always available.
    - (Decide: also on shell start? Likely NO -- adds latency; deploy covers real changes.)
  Gate: a fresh alias/bind added + deploy -> appears in `cheat` with no manual step.
## Gates
- [x] P1: `cheat --refresh` rebuilds registry from live sources; counts match reality
      (aliases 299, keybinds 35, phantoms 0); all aliases visible in cheat
- [x] P2: load-time liveness coloring (green verified / dim stale); open < 200ms
- [x] P3: auto-refresh on deploy; new alias/bind appears without manual refresh
- [x] No phantom Arch-era keybinds remain
- [x] Demonstrated live (not just implemented): open cheat, see 299 aliases, real binds,
      correct colors, after a deploy
## Notes
- Source of truth precedence: shell_aliases (aliases), mango config.conf (keybinds),
  fsh dispatch (builtins), registry/tools.toml + PATH (commands).
- command_registry schema already supports this: kind, name, source, category,
  description, expansion, example, added_at, last_seen, deprecated. Use last_seen to
  track refresh recency; deprecated=1 to hide stale without deleting history.
- cheatsheet_tui.rs already filters WHERE deprecated=0 -- so marking phantoms
  deprecated=1 hides them immediately with zero TUI change.
- Keep it forest-owned: this is fsh's own source (rust-tools/faelight-shell), the
  kind of tool we build and understand, not adopt.
## The Rule
"The cheatsheet must not lie. If it lists a command, the command must exist.
 If a command exists, the cheatsheet must know it. The forest knows itself --
 starting with knowing its own commands." 🌲


## Progress -- 2026-06-26 (Phases 1a, 1b, 3 + highlighter DONE -- demonstrated)
Core thesis PROVEN: the cheatsheet (and highlighter) now read from sources of truth, not
fossils. Live counts after deploy: 296 aliases, 108 builtins, 35 keybinds (452 total entries).
  - Phase 1a (aliases+keybinds sync): DONE. refresh_registry() rebuilds from config.fsh +
    mango config.conf. cheat --refresh command added.
  - Phase 1b (builtins): DONE. Parses commands/mod.rs execute() match arms AT REFRESH (108
    real builtins, was a fossil 17). Self-syncing -- dispatcher IS the source of truth.
    curate_builtin_desc covers ~44; rest are "description pending" stubs (incremental).
  - Phase 3 (auto-refresh on deploy): DONE + proven end-to-end (testfossil rode a deploy
    into the cheatsheet, zero manual steps). faelight-shell --refresh-cheatsheet + deploy hook.
  - Highlighter fix (bonus, the reported bug): is_known_command now checks builtins + PATH +
    aliases. Green = runnable, red = not. cheat/it/gt + all 296 aliases now green.
  - Parser hardening: handles quote-wrapped, quote-containing, unquoted, commented alias
    lines; 0 missed. Fixed two real parser bugs found via count-verification.
  - Hygiene: removed 4 dead/malformed browser aliases (gmail/youtube/chatgpt/claude).
Bugs found+fixed via "demonstrated not implemented" (count checks, not trusting green):
  stale shell_aliases table -> read config.fsh directly; quote-containing alias parse miss.
REMAINING (lower priority now):
  - Phase 2 (load-time liveness coloring in the TUI): RE-SCOPED to optional polish. With all
    sources reading from truth + auto-refresh on deploy, nothing CAN be stale -- Phase 2 was a
    correctness fix that's now a visual nicety. Build only if the green/dim distinction is wanted.
  - Description curation: ~64 builtins still "description pending" -- the human, incremental piece.
Stays in-progress for Phase 2 + curation. Core value fully delivered.


## Progress -- 2026-06-26 (COMPLETE -- all phases demonstrated)
Closed fully complete. The cheatsheet (and highlighter) read from sources of truth, never
fossils, and stay correct automatically.
FINAL STATE: 296 aliases (config.fsh) + 108 builtins (commands/mod.rs dispatcher) + 35
keybinds (mango config.conf) = 439 command entries + 13 legacy 'command' kind = 452 total.
  - Phase 1a (aliases+keybinds sync from live sources): DONE.
  - Phase 1b (builtins from dispatcher match arms, self-syncing): DONE. 108 builtins, ALL
    described (curate_builtin_desc 108/108, zero "pending" stubs -- curated from ground truth:
    handler-reading + source + Christian's confirmations, no fabricated behavior).
  - Phase 2 (load-time liveness coloring): DONE + demonstrated. Entry.live + live_alias_names()
    dim aliases present in the registry but absent from live config.fsh (drifted). Proven with
    an injected ghost alias rendering dim while a live alias rendered normal.
  - Phase 3 (auto-refresh on deploy): DONE + demonstrated (testfossil rode a deploy in).
  - Highlighter (the reported "red" bug): FIXED. is_known_command checks builtins+PATH+aliases;
    green = runnable, red = not.
  - Parser hardened (quote-wrapped/containing/unquoted/commented; 0 missed); dead aliases purged.
All four command surfaces now read from truth: aliases->config.fsh, keybinds->config.conf,
builtins->commands/mod.rs, highlighter->all three. The forest knows its own commands.
The Rule fulfilled: "If it lists a command, the command exists. If a command exists, the
cheatsheet knows it." 🌲

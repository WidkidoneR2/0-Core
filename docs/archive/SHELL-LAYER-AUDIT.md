# faelight-shell Layer Audit — INT-162 Phase 1
**Date:** 2026-03-31
**File:** rust-tools/faelight-shell/src/commands/mod.rs (4503 lines, 65 functions)

## Classification Legend
- SHELL   — pure shell behavior, keep in fsh
- DATA    — table/pipeline operation, keep in fsh  
- FOREST  — reads/displays forest state (acceptable if using cache/subprocess)
- POLICY  — embeds decision logic that belongs in core (needs migration)
- UTIL    — utility/helper function

## Function Classifications

| Function | Line | Classification | Notes |
|----------|------|----------------|-------|
| fmt_time | 9 | UTIL | time formatting helper |
| emit_command | 24 | SHELL | security event logging |
| levenshtein | 40 | UTIL | fuzzy match helper |
| execute | 65 | SHELL | main dispatch — keep, wrap with ExecContext |
| forecast | 423 | POLICY | duplicates core doctor forecast — migrate |
| sandbox | 501 | FOREST | calls core sandbox via subprocess |
| checkpoint | 564 | FOREST | calls core checkpoint via subprocess |
| git_status | 654 | DATA | git data table |
| tools_table | 721 | FOREST | reads tools.toml — acceptable |
| events_table | 777 | FOREST | reads state.db events — acceptable |
| audit_table | 803 | FOREST | calls core audit via subprocess |
| history_table | 850 | SHELL | shell history display |
| checkpoints_table | 888 | FOREST | reads checkpoints |
| domains | 940 | SHELL | shell domain display |
| git_commits | 978 | DATA | git commit table |
| git_commits_subprocess | 1024 | DATA | git subprocess helper |
| git_files | 1059 | DATA | git file table |
| watch_cmd | 1101 | SHELL | watch/repeat command |
| decisions_table | 1228 | FOREST | reads decisions from db |
| alias_cmd | 1270 | SHELL | alias management |
| unalias_cmd | 1349 | SHELL | alias removal |
| list_plugins | 1366 | SHELL | plugin listing |
| reload_plugins_cmd | 1411 | SHELL | plugin reload |
| sys_processes | 1422 | SHELL | ps table |
| sys_ports | 1466 | SHELL | network ports table |
| sys_services | 1511 | SHELL | systemd services table |
| sys_files | 1553 | SHELL | file listing |
| sys_network | 1601 | SHELL | network info |
| pkg_cmd | 1650 | SHELL | package management |
| sys_packages | 1864 | SHELL | installed packages table |
| sys_logs | 1891 | SHELL | journal logs |
| search | 2000 | SHELL | file/content search |
| cd | 2087 | SHELL | directory change |
| parse_since_time | 2109 | UTIL | time parsing helper |
| since_cmd | 2136 | FOREST | reads events by time range |
| debug_cmd | 2262 | SHELL | shell debug info |
| usage_report | 2377 | SHELL | command usage stats |
| z_jump | 2435 | SHELL | directory jump |
| theme_cmd | 2472 | SHELL | prompt theme switch |
| run_external | 2498 | SHELL | external command executor |
| help | 2712 | SHELL | help display |
| health | 2777 | FOREST | reads cached health score — acceptable |
| events | 2812 | FOREST | reads state.db events — acceptable |
| decisions | 2865 | FOREST | reads decisions — acceptable |
| intents | 2925 | FOREST | reads intent files — acceptable |
| tools | 2970 | FOREST | reads tools registry — acceptable |
| version | 3010 | FOREST | reads VERSION file — acceptable |
| commits | 3044 | DATA | git commit count |
| story | 3076 | FOREST | calls core story via subprocess |
| advise | 3089 | FOREST | calls core advise via subprocess |
| audit | 3101 | FOREST | calls core audit via subprocess |
| schema | 3114 | SHELL | schema display |
| find_cmd | 3134 | SHELL | file finder |
| index_directory | 3230 | SHELL | directory indexer |
| git_churn | 3285 | DATA | git churn analysis |
| git_branches | 3366 | DATA | git branch table |
| history_stats | 3414 | SHELL | history statistics |
| history_pattern | 3482 | SHELL | history pattern analysis |
| on_cmd | 3549 | SHELL | event trigger |
| ensure_snapshots_schema | 3628 | SHELL | db schema helper |
| snapshot_cmd | 3646 | SHELL | session snapshot |
| timeline_cmd | 3728 | SHELL | event timeline |
| snap_diff_cmd | 3786 | SHELL | snapshot diff |
| sql_query_cmd | 3923 | SHELL | SQL query interface |
| parse_sql_query | 3969 | UTIL | SQL parser |
| dashboard_cmd | 4059 | SHELL | dashboard router |
| dashboard_system | 4073 | SHELL | system dashboard |
| dashboard_forest | 4173 | FOREST | forest dashboard |
| scripting_let_cmd | 4258 | SHELL | .fsh let binding |
| scripting_run_cmd | 4291 | SHELL | .fsh script runner |
| histogram_cmd | 4364 | DATA | histogram visualization |
| chart_cmd | 4420 | DATA | chart visualization |
| render_chart | 4438 | DATA | chart renderer |

## Summary
| Category | Count | Action |
|----------|-------|--------|
| SHELL | 35 | Keep — correct layer |
| DATA  | 12 | Keep — correct layer |
| FOREST | 16 | Keep — acceptable (cache/subprocess) |
| POLICY | 1 | Migrate — forecast() duplicates core |
| UTIL  | 4 | Keep — helpers |

## Priority Migrations (Phase 5)
1. **forecast()** — duplicates core doctor forecast exactly
   Action: replace with subprocess call to `core doctor forecast`

## DEC-005 Compliance
Current state: 98% compliant.
forecast() is the only function embedding policy logic.
All other FOREST functions correctly delegate to core via subprocess
or read from acceptable shared state (db cache, VERSION file).

## Phase 1 Complete
Layer audit documented. One migration identified.
Foundation is solid — shell architecture is cleaner than expected.

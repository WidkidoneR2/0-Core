# 0-Core v2 Migration Log

## System State
- v1.0.0-stable tagged: ✅
- engine/ scaffold: ✅

## Phase Status
| Phase | Status | Notes |
|---|---|---|
| 0 — Freeze & Tag | ✅ complete | v1.0.0-stable tagged |
| 1 — engine/ scaffold | ✅ complete | core version, core doctor working |
| 2 — Wrappers | 🔄 in-progress | link domain complete |
| 3 — Domain migration | ⬜ pending | |
| 4 — Runtime isolation | ⬜ pending | |
| 5 — Remove script layer | ⬜ pending | |
| 6 — Capability enforcement | ⬜ pending | |

## Domain Migration Status
| Domain | Status | doctor score |
|---|---|---|
| link | ✅ complete | 20/21 |
| zone | ✅ complete | 20/21 |
| intent | ⬜ pending | — |
| profile | ⬜ pending | — |
| security | ⬜ pending | — |
| sandbox | ⬜ pending | — |
| update | ⬜ pending | — |
| doctor | ⬜ pending | — |
| fetch | ⬜ pending | — |
| git | ⬜ pending | — |
| workspace | ⬜ pending | — |
| release | ⬜ pending | — |
| notify | ⬜ pending | — |
| lock | ⬜ pending | — |
| launcher | ⬜ pending | — |

## Session Log
- 2026-02-20: Phase 0 complete, v1.0.0-stable tagged
- 2026-02-20: Phase 1 complete, engine/ scaffold builds, core version + core doctor working
- 2026-02-20: Phase 2 started, link domain migrated, faelight-link wrapper delegates to core link
- 2026-02-20: zone domain migrated, faelight-zone wrapper delegates to core zone

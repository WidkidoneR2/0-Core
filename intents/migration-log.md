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
| intent | ✅ complete | 20/21 |
| profile | ✅ complete | 20/21 |
| security | ✅ native | 20/21 |
| sandbox | ✅ complete | 20/21 |
| update | ✅ complete | 20/21 |
| doctor | ✅ complete | 20/21 |
| fetch | ✅ complete | 20/21 |
| git | ✅ complete | 20/21 |
| workspace | ✅ complete | 20/21 |
| release | ✅ complete | 20/21 |
| notify | ✅ complete | 20/21 |
| lock | ✅ complete | 20/21 |
| launcher | ✅ complete | 20/21 |

## Session Log
- 2026-02-20: Phase 0 complete, v1.0.0-stable tagged
- 2026-02-20: Phase 1 complete, engine/ scaffold builds, core version + core doctor working
- 2026-02-20: Phase 2 started, link domain migrated, faelight-link wrapper delegates to core link
- 2026-02-20: zone domain migrated, faelight-zone wrapper delegates to core zone
- 2026-02-20: intent domain migrated, intent wrapper delegates to core intent
- 2026-02-20: profile domain migrated, profile wrapper delegates to core profile
- 2026-02-20: security domain migrated, security-audit wrapper delegates to core security
- 2026-02-20: sandbox domain migrated, faelight-sandbox wrapper delegates to core sandbox
- 2026-02-20: fetch domain migrated, faelight-fetch wrapper delegates to core fetch
- 2026-02-20: git domain migrated, faelight-git wrapper delegates to core git
- 2026-02-20: workspace domain migrated, wrappers for workspace-view, recent-files, faelight-fm
- 2026-02-20: release domain migrated, get-version reads natively, bump tools delegate to v1
- 2026-02-20: core alias updated — core now invokes v2 binary, 0core navigates to root
- 2026-02-20: notify domain migrated, faelight-notify wrapper delegates to core notify
- 2026-02-20: lock domain migrated, faelight-lock wrapper delegates to core lock
- 2026-02-20: launcher domain migrated, palette/dmenu/launcher wrappers delegate to core launcher
- 2026-02-20: update domain migrated, faelight-update and safe-update delegate to core update
- 2026-02-20: doctor domain migrated — ALL 15/15 DOMAINS COMPLETE

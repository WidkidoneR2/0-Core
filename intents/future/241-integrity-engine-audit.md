---
id: 241
title: "Integrity Engine Audit -- No Phantom Fixes, No Silent Failures"
status: in-progress
date: 2026-04-18
tags: [integrity, doctor, audit, fixes, phantom, reliability]
---
The integrity engine auto-fixes 1 issue on every single run.
This is wrong. An auto-fix that fires every run is not a fix -- it is noise.
The jarvis_log_freshness check was the immediate offender -- fixed.
But the audit needs to go deeper.
Every auto-fix in the integrity engine must be verified:
1. Does the fix actually fix something?
2. Does the problem recur after the fix?
3. Is the check threshold appropriate?
4. Is the check still relevant to the current system?
- registry_version_drift -- does this fire spuriously?
- jarvis_log_freshness -- FIXED (30 day threshold, real insert)
- db_wal_mode -- does WAL mode get unset somehow?
- Any new checks added since last audit
- Are proposals actually actionable?
- Do they recur after being applied?
- Are alerts accurate?
- Any false positives?
Every integrity check must satisfy:
- Fires only when genuinely broken
- Fix actually resolves the issue
- Does not re-fire within the same session after fix
- Threshold is based on real system behavior, not arbitrary values
⬜ All AutoFix checks audited -- each fires only when genuinely broken
⬜ registry_version_drift verified -- does not fire spuriously
⬜ db_wal_mode verified -- WAL stays set across deploys
⬜ jarvis_log_freshness verified -- does not fire within 30 days of fix
⬜ All Propose checks reviewed for accuracy
⬜ All Alert checks reviewed for false positives
⬜ d shows no auto-fix on clean system
⬜ integrity engine audit documented in decisions/
"An integrity engine that cries wolf
is worse than no integrity engine at all." 🌲

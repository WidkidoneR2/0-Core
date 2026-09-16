---
id: 141
date: 2026-07-11
type: future
title: "faelight-glog v0.2: author/date filters + floating-window view"
status: cancelled
tags: [glog, git, tui, filters, floating-window]
priority: low
---

## Vision
Extend faelight-glog (v0.1 shipped in INT-139) with the filters deliberately deferred at v0.1,
and explore a candy-neon floating-window presentation of glog + forest info.

## Deferred from INT-139 (v0.1 shipped with keyword + INT-number filtering only)
v0.1 gate 3 ("filter by author / date-range / keyword fuzzy over subject+body") was ticked
PARTIAL: keyword + INT-number substring over the SUBJECT line works. These remain for v0.2:
- Dedicated AUTHOR filter (glog already parses %an; add a filter mode / prefix).
- DATE-RANGE filter (glog already parses %aI ISO dates; add from/to filtering -- reuse the
  is_arch_era() date-compare pattern already in the code).
- Fuzzy search over the commit BODY, not just the subject (v0.1 loads subject only via
  `git log --pretty`; body needs an on-demand or bulk fetch -- consider git2, already a
  workspace dep in faelight-shell).

## The floating-window idea (Christian, 2026-07-11 -- exploratory, note the overlap)
A candy-neon FLOATING WINDOW view of glog (and possibly more forest info at a glance) rather
than / in addition to the ratatui TUI. This would use the GTK4 + gtk4-layer-shell recipe the
forest already owns (faelight-launcher / faelight-logout / faelight-bar): a glassy, summonable
panel showing recent commits, maybe active intent + health + Friday signal.

OVERLAP FLAG: this heavily overlaps INT-014 (faelight-dashboard v2 -- "forest health, active
intents, Friday status, system resources, recent commits in one view"). The float idea may
BE part of 014, or feed it, rather than living in glog. DECIDE the home before building:
- If it's "a git-log float" -> could be glog v0.2.
- If it's "a forest-info float (git + intents + health + Friday)" -> that's INT-014.
Do not let glog sprawl into a dashboard by accident. This intent CAPTURES the idea; it does
not commit glog to becoming a floating window.

## Success Criteria (draft -- refine at build)
⏸ Author filter: narrow the log to a chosen author -- deferred: the tool was retired 2026-09-15 (INT-247 Layer 2) -- approved by: christian 2026-09-15
⏸ Date-range filter: narrow to a from/to window (reuse ISO date-compare) -- deferred: the tool was retired 2026-09-15 (INT-247 Layer 2) -- approved by: christian 2026-09-15
⏸ Fuzzy search covers the commit BODY, not just the subject -- deferred: the tool was retired 2026-09-15 (INT-247 Layer 2) -- approved by: christian 2026-09-15
⏸ Floating-window direction DECIDED (glog v0.2 vs fold into INT-014) before any float build -- deferred: the tool was retired 2026-09-15 (INT-247 Layer 2) -- approved by: christian 2026-09-15
⏸ Each new filter demonstrated live on the real repo (demonstrated-not-declared) -- deferred: the tool was retired 2026-09-15 (INT-247 Layer 2) -- approved by: christian 2026-09-15

## Depends On / Relates To
- INT-139 (faelight-glog v0.1 -- shipped; this extends it)
- INT-014 (faelight-dashboard v2 -- the floating-window idea overlaps; resolve ownership)
- git2 (workspace dep in faelight-shell -- the likely path for body-fuzzy + richer queries)

---

## CANCELLED 2026-09-15 -- THE TOOL IS GONE

`faelight-glog` was retired (INT-247 Layer 2, evidence from the INT-249 census): 0
invocations through `fgl` since the migration. This intent is v0.2 of a crate that
no longer exists, so the author/date/body filters go with it.

⭐ AND `glog` -- the alias that looked like evidence this tool was alive -- was
`git log --oneline -10` all along. It never pointed at this crate. The tool had been
replaced without being retired, and only counting ALIASES showed which was which.

### ⚠️ THE FLOATING-WINDOW IDEA HAS NO HOME, AND THAT IS THE HONEST RECORD

This intent said the float would belong to INT-014 if it were a forest-info panel
rather than a git-log one. That branch is also closed: **INT-014 is CANCELLED** -- it
was "full NixOS replacement, ratatui, forest-native", and the machine has not been
NixOS since 2026-08-26.

So the idea is not being handed on. It is being RECORDED AS UNHOMED: a candy-neon
summonable panel showing recent commits, active intent, health and Friday signal.

Written down because the idea outlived both intents that tried to hold it, which is
evidence it is real -- but Omarchy already has Waybar and walker, and the honest first
question next time is whether this is a NEW panel or a Waybar module. Do not reopen it
as "glog v0.2".

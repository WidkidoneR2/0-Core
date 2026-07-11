---
id: 141
date: 2026-07-11
type: future
title: "faelight-glog v0.2: author/date filters + floating-window view"
status: planned
tags: [glog, git, tui, filters, floating-window]
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
- [ ] Author filter: narrow the log to a chosen author
- [ ] Date-range filter: narrow to a from/to window (reuse ISO date-compare)
- [ ] Fuzzy search covers the commit BODY, not just the subject
- [ ] Floating-window direction DECIDED (glog v0.2 vs fold into INT-014) before any float build
- [ ] Each new filter demonstrated live on the real repo (demonstrated-not-declared)

## Depends On / Relates To
- INT-139 (faelight-glog v0.1 -- shipped; this extends it)
- INT-014 (faelight-dashboard v2 -- the floating-window idea overlaps; resolve ownership)
- git2 (workspace dep in faelight-shell -- the likely path for body-fuzzy + richer queries)

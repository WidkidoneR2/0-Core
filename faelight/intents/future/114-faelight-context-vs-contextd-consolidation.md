---
id: 114
date: 2026-07-02
type: future
title: "faelight-context vs contextd consolidation"
status: planned
tags: [context, cleanup, naming.]
---

## Vision
Resolve the faelight-context vs faelight-contextd naming/function overlap so
there's one clear tool (or two clearly-distinct ones), eliminating confusion.

## The Problem
Two similarly-named tools coexist in the workspace:
- faelight-context (v1.0.0)
- faelight-contextd (v0.1.0)
The `-d` suffix usually means "daemon," implying one is a CLI and one a background
service -- but the naming is close enough to cause confusion. Unclear if they're
(a) a proper CLI+daemon pair (keep both, maybe clarify names), (b) one superseding
the other (retire the loser), or (c) accidental duplication (merge).

## Recon needed (before deciding)
- What does faelight-context DO? (CLI? one-shot context query?)
- What does faelight-contextd DO? (daemon? persistent context service?)
- Do they share code / a state source? Does one call the other?
- Is contextd (v0.1.0, early) an in-progress replacement for context, or a
  companion daemon?

## Decision space
- KEEP BOTH as a clear CLI + daemon pair (possibly rename for clarity).
- MERGE if duplicative.
- RETIRE one if superseded (get-version/profile pattern).

## Gates (when built)
- [ ] Function of each tool documented
- [ ] Relationship (pair / duplicate / supersede) determined
- [ ] Decision executed: keep-both / merge / retire, with clear naming

---

---
id: 262
date: 2026-05-01
type: arch
title: \"faelight-term Dead Code Audit and Renderer Decision\"
status: complete
tags: [architecture, rust, design]
version: TBD
---

## Vision

faelight-term v2 has dead and misleading code. The renderer module is a stub. The Config struct has unread fields. There are stale pty backup files in the source tree. wgpu is a dependency but the actual rendering happens software-rasterized in main directly. This intent audits the dead code, decides whether the GPU path lives or dies, and removes anything that pretends to do work it does not actually do.

## Why Now

Discovered while diagnosing INT-232 brightness, glyph fallback, and selection issues. Surface reading suggested a config-driven GPU pipeline. Real reading found software rendering in main with everything else as scaffolding. Wasted real diagnostic time today and will keep wasting time on every future term issue. Created planned, blocked on INT-232.

## Approach

Three passes. First, inventory every dead or stub file and decide delete or wire-up. Second, decide GPU or software as the official rendering path and remove the losing dependencies. Third, either wire Config into the actual render path or delete it. Recommendation leaning software-only since that is what the binary already does and it is fast.

## Success Criteria

- [ ] Every file in src does real work or is deleted
- [ ] Stale pty backup files removed from source tree
- [ ] Renderer architecture decision made and documented
- [ ] Config either fully wired or fully removed
- [ ] No dead_code annotations remain unjustified
- [ ] cargo build release clean with no warnings
- [ ] System health 100 percent after audit

## Gate Check

Not started. Blocked on INT-232 daily-driver readiness.

---
id: 124
date: 2026-07-06
type: future
title: "Health freshness: refresh doctor event on session-start-if-stale + after deploy (splash never shows stale health)"
status: in-progress
tags: [health, doctor, splash, deploy]
---

## Vision
The splash never lies about health. A self-knowing forest should report its
current state, not a stale snapshot -- the number you see on terminal open should
always match `d`. Health refreshes automatically at the moments it changes (boot,
deploy), with zero cost on the common path.

## Problem

The terminal splash "system health" line reads the latest `doctor` event from
state.db (db.rs:385, `detail.health`). After a reboot (which clears generation
drift) or any health change, the recorded event is stale until `d` / `core doctor
run` is run manually — so the splash can display a wrong number.

Observed 2026-07-07: splash showed 93% (ADVISORY) while live `d` showed 100%
(HEALTHY, 32/32). Root cause: the last recorded doctor event pre-dated the reboot
(00:09 = 93% with 2 drift warnings); the reboot cleared drift but nothing
re-ran doctor, so the splash kept reading the stale 93% event until `d` was run.
Data is correct; the splash reads a pre-change snapshot.

## Fix — two triggers

### A. Session-start refresh-if-stale (fsh main.rs, before session::render ~3492)
- Before rendering the splash, check if the latest doctor event is STALE:
  older than the current boot time, OR older than a TTL (e.g. 30 min).
- If stale → run `core doctor run` once to refresh events + cache.
- If fresh → skip (no doctor run). MUST be cheap on the common path — a
  timestamp compare, never a doctor run when a recent event exists. Full doctor
  is ~500ms; running it on every terminal open is unacceptable.

### B. Post-deploy refresh (deploy flow)
- Deploy currently READS ~/.cache/faelight/health-status without refreshing it
  (deploy/mod.rs:26). Ensure the full 32-check `core doctor run` (writes both the
  events row and the health-status cache) runs as part of deploy, so both
  surfaces are fresh after every deploy.

## Gates (demonstrated, not declared)

- [ ] Reboot → open terminal → splash shows CURRENT health without manually running `d`
- [ ] Recent event (within TTL) → session start does NOT re-run doctor (verify no startup penalty via timing)
- [ ] After `deploy` → recorded doctor event reflects post-deploy health
- [ ] Splash number always matches `d` number (no divergence)

## Design guardrails

- Stale-gate = timestamp compare on the common path, NOT a doctor run.
- Boot-time detection: compare event ts against system boot (/proc/stat btime),
  so "stale = older than this boot".
- Splash and `d` must read the SAME source (events table `detail.health`) —
  already verified as the correct shape.
- Three health surfaces exist (events table = splash, ~/.cache/faelight/health-status
  = deploy gate, live = `d`); keep them consistent, don't add a fourth.

---

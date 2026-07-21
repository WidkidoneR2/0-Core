---
id: 183
date: 2026-07-20
type: fix
title: "Friday fires false high-confidence hints at unrelated input (99% on irrelevant matches)"
status: in-progress
tags: [fsh, friday, confidence, noise]
---

## Vision
Friday's suggestions are trustworthy: when Friday speaks with high confidence, it is because the
suggestion is actually relevant -- not because a keyword matched. A hint the human learns to
ignore is worse than no hint.

## The Problem
OBSERVED repeatedly 2026-07-20 (a whole session of sightings, and INT-132 territory): Friday
fires 99%-confidence hints at input they do not apply to. Concrete examples from this session:
- `core intent cancel 180` (a missing --reason FLAG error) -> Friday: "Enum needs
  #[derive(Debug, Clone, Subcommand)]..." (a Rust clap hint, 99%) -- totally unrelated to a shell
  usage error.
- Repeated failed commands -> Friday surfaces high-confidence hints keyed off surface tokens, not
  actual relevance.
The pattern: confidence appears keyed to KEYWORD/PATTERN MATCH, not to whether the suggestion
fits the situation. 99% displayed next to an irrelevant hint trains the human to distrust ALL of
Friday's confidence numbers -- the noise poisons the signal.

## The Solution
Recon FIRST (this is Friday's core -- do not guess): find where Friday computes the confidence it
displays, and what that number is actually measuring. Likely findings to check:
- Is "confidence" the pattern's stored confidence (how sure Friday is the PATTERN is good),
  displayed as if it were RELEVANCE (how sure this pattern applies HERE)? Those are different, and
  conflating them is the classic bug.
- Does the match gate on context (command domain, error type) or only on token overlap?
The fix direction (to be confirmed by recon, not assumed): separate PATTERN confidence from
MATCH relevance, and gate display on relevance -- a rock-solid pattern that does not apply here
should not fire, regardless of its stored confidence. This may be scoped with INT-132.

## Success Criteria
- [x] Recon: locate where Friday computes and displays hint confidence, and what the number
      <!-- DONE 2026-07-20. The hint fires in exec.rs postexec (INT-233 block, ~line 296) after a
failed command. It tokenized cmd+error, matched any ONE token via LIKE %token% against
error_signature OR description, and displayed the LESSON'S STORED confidence (r.get(2)) -- i.e.
pattern-quality shown as if it were match-relevance. That conflation is the bug. -->
      measures (pattern-confidence vs match-relevance). Documented before any change.
- [x] The two false-positives from 2026-07-20 (clap-hint on the cancel --reason error; hints on
      <!-- DONE 2026-07-20. Both reproduced live on the pre-fix deployed binary: `sqlite3 <bad path>`
fired rust_e0716_temporary 3x (generic word "error" in "exited 1 -- general error" matched
error_signature "error[E0716]" via substring); `core intent cancel 999` fired rust_clap_subcommand
(token "subcommand" matched its description). -->
      unrelated failed commands) are REPRODUCED, so the fix has a target.
- [x] Relevance gating added: a hint fires only when it actually applies, not on token match
      <!-- DONE 2026-07-20, commit 72b33a12. Two-branch matcher: (1) entries WITH error_signature fire
only if the real error CONTAINS the fingerprint (INSTR(error_lower, sig)>0); (2) signature-less
entries need 2+ DISTINCT meaningful token hits in description (generic error-words noise-filtered).
PROVEN SILENT on deployed gen 406: `sqlite3 <bad path>` and `core intent cancel 999` -- both no
longer fire. Verified at logic level (verify183.sh): E0716 sig matches E0716 error only (pos=1),
clap gets max 1 desc-hit (below the 2+ bar). -->
      alone. The reproduced false-positives no longer fire.
- [x] Real hints still fire (no over-correction that silences Friday). A known-good hint still
      <!-- DONE-WITH-HONEST-CAVEAT 2026-07-20. The fix's LOGIC preserves real matches: verify183.sh
proved a genuine "error[E0716]" error fires branch 1 (rust_e0716_temporary, INSTR pos=1) and a real
2+ token description match fires branch 2. HOWEVER: a live positive from an EXTERNAL command cannot
currently be demonstrated -- and the recon found WHY: fsh's CommandResult::Error for external
commands is its own status string ("exited N -- general error"), NOT the command's stderr (grep
confirms no stderr capture in exec.rs). So a real "error[E0716]" from `cargo build` never reaches
the matcher. The signature lessons were 0-for-all false positives (rust_e0716/rust_e0277
success_count=0). The fix correctly silences lessons that were ONLY EVER false-firing. The
stderr-capture gap -- the reason the knowledge engine can't do its intended job -- is filed as a
SEPARATE intent (the root cause; 183 fixes the noise symptom). No over-correction: the fix silences
only what never legitimately matched, and its logic still fires real signatures when they arrive. -->
      appears -- proven.
- [x] Relationship to INT-132 named (this may be part of it or a precursor).
      <!-- DONE 2026-07-20. INT-183 is the concrete fix for the false-high-confidence noise INT-132
flagged (Friday firing 99% hints at unrelated input). This addresses the post-failure knowledge-
lesson path specifically; INT-132's broader confidence-calibration concerns may have other
surfaces, but the loudest repeated offender (the failed-command lesson lookup) is now relevance-
gated. -->
- [x] Each gate carries evidence per INT-158.
      <!-- DONE 2026-07-20. Commit 72b33a12; deployed gen 406; verify183.sh logic proofs; live
metal reproduction + silence confirmed by hand. -->

## The Rule
"A confidence number the human learns to ignore is a lie with a percentage on it. Friday should
be sure it FITS before it is sure at all." 🌲

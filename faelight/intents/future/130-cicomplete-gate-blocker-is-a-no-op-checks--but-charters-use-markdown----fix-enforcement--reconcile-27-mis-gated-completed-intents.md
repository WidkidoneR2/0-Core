---
id: 130
date: 2026-07-07
type: future
title: "cicomplete gate-blocker is a no-op: checks ⬜ but charters use markdown [ ] — fix enforcement + reconcile 27 mis-gated completed intents"
status: planned
tags: [cicomplete, integrity, gates, int-332, critical, ledger]
---

## PRIORITY: CRITICAL -- ledger integrity
This is not a feature. cicomplete's gate-enforcement (INT-332) has NEVER worked.
Intents have been marked `complete` with open gates since the mechanism was written.
"Complete" in the ledger cannot be trusted until this is fixed and the affected
intents are reconciled.

## The Bug (proven 2026-07-07)
engine/src/domains/intent/mod.rs, the INT-332 open-gate check (~line 928):

    let open_gates: Vec<&str> = file_content.lines()
        .filter(|l| {
            let trimmed = l.trim();
            trimmed.starts_with('U+2B1C') && !l.contains('U+23F8')   // looks for the WHITE-SQUARE emoji
        })

It counts a gate as OPEN only if the line starts with the U+2B1C white-square emoji.
But EVERY charter writes gates as markdown checkboxes: `- [ ]`. The blocker never
checks for `- [ ]`. So it matches nothing, the open_gates vec is always empty, and
cicomplete NEVER blocks. It has been a silent no-op for all markdown-gated intents.

PROOF:
  grep -rl 'U+2B1C' intents/complete/   -> EMPTY (no charter uses the emoji)
  grep -c '^- \[ \]' intents/complete/117-*.md -> 6 (uses markdown)
So the blocker scans for a character present in ZERO charters while charters use a
format it never checks. The deferral/malformed-deferral logic above it likely has the
same emoji-vs-markdown mismatch.

## The Fix
1. Rewrite the open-gate detection to parse MARKDOWN checkboxes:
   - OPEN  = line matching `- [ ]` (unchecked)
   - DONE  = line matching `- [x]` or `- [X]`
   - Optionally ALSO accept the U+2B1C/U+2705 emoji forms for back-compat.
2. Keep the deferral escape hatch, but make it recognize a markdown deferral marker
   too (not only the U+23F8 emoji).
3. TEST IT: attempt to cicomplete an intent that has an open `- [ ]` gate and confirm
   it now BLOCKS with the gate listed. Then check the box and confirm it completes.
   (Demonstrated-not-declared: the fix must be shown to actually block.)

## Reconciliation -- 27 completed intents with unchecked gates
These were marked complete while the blocker was a no-op. Each must be audited:
work verifiably done -> tick the boxes (with a note); work NOT done -> REOPEN.
Do NOT bulk-tick blindly -- that repeats the original sin (marking complete without
verification). Audit per intent.

  - 023-replace-wallpaper-idle-with-nix-services
  - 028-forest-dev-tooling-additions
  - 032-faelight-fm-v4-nix-explorer
  - 064-faelight-logout:-candy-neon-wayland-power-menu
  - 065-faelight-notify-managed-systemd-user-service
  - 091-evaluate-stylix-declarative-theming
  - 097-fsh-neeeds-a-clean-nixshell-operator-path
  - 098-forest-hygiene-pass-registry-reconciliation--deadwood-orphan-cleanup
  - 099-fsh-handle-multi-line-command-blocks-per-line-execution--abbreviation-expansion
  - 100-fsh-variable-assignment-and-varexpansiontgvar-expansion-tg
  - 101-db-cwd-column-fix
  - 103-improving-fsh-prompts
  - 104-shell-snapshots-schema-intent
  - 105-fix-pathsrs-drift-realign-canonical-path-module-to-nixos-era-flat-structure
  - 106-pathsrs-consolidation-follow-ups-rename-rulesdir-fix-hardcoded-font-route-hardcoded-paths-through-the-module
  - 107-decommission-arch-era-stowlink-subsystem
  - 108-profile-profile-mechanism
  - 116-final-arch-sweep-retire-safe-update-de-arch-fsh-pkg-command-purge-pacmanaur-remnants-for-true-nixos-native-100
  - 117-friday-arch-language-cleanup-de-arch-knowledge-facts-suggestion-strings-teach-descriptions
  - 119-git-hooksnix-evalution
  - 120-abort-message-quality--check-other-errors-for-better-message-quality
  - 122-evaluate-nixcats-vs-current-nixvim
  - 123-release-changelog-polish-cap-what-shipped--strip-int-numbers-from-notable-changes
  - 124-health-freshness-refresh-doctor-event-on-session-start-if-stale--after-deploy-splash-never-shows-stale-health
  - 125-cicomplete-auto-syncs-cargolock-after-version-bumps
  - 126-fsearch-teach-extension-allow-list-nix--forest-config-type-forest-configs
  - 127-rewire-vm-verbs-vm-updownstatus----unwired-by-int-027-faelight-vm-undeployed

NOTE on this session's intents (117, 125, 126, 127): their gate WORK was demonstrated
live this session (fsearch .nix, vm restore, lock-sync, de-Arch) -- for these,
reconciliation is ticking boxes for work already proven. The older ones (023-116)
need genuine per-intent audit -- work status unknown, must be verified not assumed.

## Gates
- [ ] Root cause confirmed in code (emoji-vs-markdown mismatch at intent/mod.rs ~928)
- [ ] Blocker rewritten to detect `- [ ]` markdown gates (+ back-compat emoji)
- [ ] Deferral logic updated to match markdown too
- [ ] DEMONSTRATED: cicomplete BLOCKS on an open `- [ ]` gate (shown live), and
      completes once checked
- [ ] Reconciliation pass: every affected intent audited (tick if done / reopen if not)
- [ ] core builds clean, deployed, verified on the running binary

## Relationship
- Fixes INT-332 (which introduced the never-working blocker).
- Trust-critical: the ledger's "complete" is only meaningful once this lands.
- Related INT-125 (also cicomplete): the version-bump path works; the GATE path doesn't.

## The Rule
"A guard that checks for a door no one uses has never once stopped anyone.
 A ledger that says 'done' without proof is a story, not a record.
 Make the guard watch the real door. Make 'complete' mean complete." 🌲

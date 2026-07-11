---
id: 130
date: 2026-07-07
type: future
title: "cicomplete gate-blocker is a no-op: checks ⬜ but charters use markdown [ ] — fix enforcement + reconcile 27 mis-gated completed intents"
status: in-progress
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


## Progress (2026-07-07)
- ROOT CAUSE fixed: gate-blocker now detects markdown `- [ ]` / `- [~]` (was ⬜-emoji-only
  no-op). Committed 872159c8. PROVEN LIVE both ways with throwaway INT-131: open `- [ ]`
  -> cicomplete BLOCKED; `- [x]` -> completed. 131 cancelled.
- Confirmed the gate-block does NOT interfere with the version-bump path: they are
  sequential + independent (block runs first and returns early; bump runs later only if
  the block passed). 131 completed with no bump because it touched no crates -- correct.
- TIER 1 reconciled (committed a3274ae2): 117/125/126/127 -- gates ticked because their
  work was DEMONSTRATED LIVE in this session. Notes added inline on the 3 non-obvious
  gates (125 crane-mismatch = build-succeeded-on-deploy not isolated test; 126 .conf =
  met-by-inspection, dotdir-skip not extension; 117 config.fsh = actually config.rs:104).
  126 junk stub line deleted.
- LEDGER AUDIT (2026-07-07): the ONLY anomaly is complete/ intents with open gates.
  in-progress (056, 130) and future/* open gates are NORMAL. No status/folder mismatches
  (grep 'status: complete' in non-complete folders = empty). Damage is bounded + mapped.

## Reconciliation Method (for the remaining 23 -- do NOT bulk-tick)
Bulk-ticking to make the ledger look clean would REPEAT the original sin. Each intent
gets a real look. Per intent:
1. Read its gates AND its charter body (## Status / completion notes -- many older
   charters document what was demonstrated, e.g. 116/024/077).
2. Decide per gate:
   - Work VERIFIABLE (charter documents it done, or it's trivially checkable now)
     -> tick `- [x]`, add an inline `<!-- -->` note citing the evidence.
   - Work NOT evidenced anywhere -> do NOT tick. Either REOPEN (move to in-progress)
     or FORMALLY DEFER: `⏸ gate -- deferred: [reason] -- approved by: christian [date]`.
3. Stamp the charter: `<!-- Gates reconciled per INT-130, [date]: [tick/defer/reopen summary] -->`.
Reopening is not failure -- it is the ledger telling the truth. Better an honest
"in-progress" than a false "complete."

## Progress (2026-07-10g) -- 100 + 101 reconciled -- HALFWAY (10/23)
Two more, both charter-repairs like 099:
- 100 (fsh VAR=$(...) value-truncation): 4bf16cb6. Malformed charter (stub + prose gates);
  removed stub, added 5 [x] gates. Was REOPENED once (diagnosis mistaken for fix) -- so
  VERIFIED LIVE: X=$(echo one two three) captured whole, not truncated. Initial "lowercasing"
  theory was a misdiagnosis; real bug was split_whitespace truncation.
- 101 (fsh fresh-db schema ordering, shell_history cwd): abf69ef1. Charter repair; 3 [x]
  gates. VERIFIED IN SOURCE -- db.rs orders CREATE (line 53) before ALTERs (73-76); root
  cause (ALTER-before-CREATE) corrected. Source-verified, not fresh-db-spiked.

TALLY: 10 genuinely reconciled (023, 028, 064, 065, 091, 097, 098, 099, 100, 101) + 1
false-complete corrected (032). REMAINING 10: 103, 104, 105, 106, 107, 108, 116, 119, 120,
122, 123, 124. NEXT UP: 103.

NOTE: the fsh cluster (099/100/101) all shared the SAME malformed-charter shape -- a dead
template stub (Vision/Problem/Solution placeholders + junk "- [ ] ...") above the real
content, with real gates as prose not checkboxes. Repair = strip stub + convert/insert real
[x] gates. Watch for more of these in the 103-108 range (same era). Stamp marker changed to
STAMP-NNN-DONE (distinct from headers) to avoid false "already stamped" aborts.

## Progress (2026-07-10f) -- 097, 098, 099 reconciled
Three more GENUINE reconciles, all verified not assumed:
- 097 (fsh clean Nix/Shell operator path): 2df51b9c. 5/7 gates already ticked w/ commits;
  2 acceptance gates closed on attestation (fsh daily-driver 12+ days, no systemic bash drops;
  features deferred to INT-134).
- 098 (forest hygiene pass): 7b9e3268. 5/6 already ticked; Phase 2 (register 11 tools)
  VERIFIED LIVE -- all 11 present in registry/tools.toml.
- 099 (fsh multi-line command blocks): 0ecd1267. GENUINE + CHARTER REPAIR -- charter was
  malformed (dead template stub + gates as prose not checkboxes). Removed stub, converted
  4 gates to [x]. for-loop gate verified LIVE. NOTE: log has a DIFFERENT Arch-era INT-099
  (Niri migration) -- disambiguated by date; our 099 is the NixOS-era one (commits 3c170e2a/
  3fcdde34).

TALLY: 8 genuinely reconciled (023, 028, 064, 065, 091, 097, 098, 099) + 1 false-complete
corrected (032). REMAINING 12: 100, 101, 103, 104, 105, 106, 107, 108, 116, 119, 120, 122,
123, 124. NEXT UP: 100.

WATCH-FORS accumulated this run: (1) false-completes (032). (2) malformed charters -- stub
templates + prose-gates (099). (3) Arch-era vs NixOS-era number collisions -- disambiguate
by date (099). (4) cosmetic frontmatter drift (099 type:future/status:complete) -- note,
don't fix mid-reconcile. ALWAYS: check git log + records + live state before ticking.

## Progress (2026-07-10e) -- 065 + 091 reconciled
Post-break run continues. Two more GENUINE reconciles:
- 065 (faelight-notify systemd service): commit e9fe550b. Verified mostly LIVE -- systemctl
  enabled+active, WantedBy=faelight-session.target; restart seatbelt DEMONSTRATED (killed
  PID 2745 -> respawned 3966 in <5s). Reboot/rebuild survival structural (Nix-store unit).
- 091 (Stylix evaluation): commit 6d102c49. Work was DONE + documented in decisions/091
  (status:decided, HYBRID-NARROW) -- checkboxes just never ticked. VM-trial gate honestly
  marked NOT-performed-but-not-required (documented in the decision record).

TALLY: 5 genuinely reconciled (023, 028, 064, 065, 091) + 1 false-complete corrected (032).
REMAINING 15: 097, 098, 099, 100, 101, 103, 104, 105, 106, 107, 108, 116, 119, 120, 122,
123, 124. NEXT UP: 097.

CONFIRMED PATTERN (Christian's insight): "if it's in the git log / a decision record, the
work happened -- the intents just never got their boxes ticked because the blocker was a
no-op." So per intent: CHECK GIT LOG + decision records FIRST, then verify live where
possible, tick to match. 091 nearly got mis-reclassified as a false-complete until the log
corrected us -- always check before reclassifying.

REMINDER (unchanged): before cicomplete on 130, DEDUPE its own duplicated gate block
(## Gates + ## 130's own gates, byte-identical). Left as-is until audit done.

## Progress (2026-07-10d) -- 064 reconciled; session pause
064 (faelight-logout) reconciled -- GENUINE this time (built + deployed, unlike 032). All 7
gates [x] on honest evidence (commit 132a8fdf): overlay/Esc/styling verified LIVE; keybind+
deploy verified (PATH + mango config.conf:107); Shutdown/Reboot/Lock ATTESTED by author in
daily use (poweroff untestable mid-session); Phase 4 scope guard VERIFIED -- no greetd/tuigreet
refs, only loginctl (no lockout risk).

TALLY: 3 genuinely reconciled (023, 028, 064) + 1 false-complete corrected (032).
REMAINING 17: 065, 091, 097, 098, 099, 100, 101, 103, 104, 105, 106, 107, 108, 116, 119,
120, 122, 123, 124. NEXT UP: 065.

REMINDER for closing 130 later: (1) audit each remaining intent for BOTH under-ticked-reconcile
AND false-complete (032 lesson). (2) Before cicomplete, DEDUPE 130's own charter -- its 6 gates
appear TWICE (## Gates ~L84 and ## 130's own gates ~L190); left as-is deliberately to avoid
touching the blocker read-surface mid-session, but must be one clean block before 130 can close.

## Progress (2026-07-10c) -- 032 was a false-complete, not a reconcile
032 (faelight-fm v4) audited: it was NEVER built (running FM is v3, INT-015) and had 0/5
gates ever met, yet sat in complete/ as status:complete. Reclassified complete/ -> future/,
status -> planned (commit 5bb0eb76) -- an honest planned v4, sibling to INT-136. This is a
DIFFERENT outcome than 023/028: not a clean reconcile-close, but a MISFILING correction.
The audit caught a false-complete that predates the ledger rebuild -- exactly what 130 is for.

Reconciliation tally: 023 + 028 genuinely closed (2). 032 removed from the 23 (it was never
a real complete). So the remaining list is now 20: 064, 065, 091, 097, 098, 099, 100, 101,
103, 104, 105, 106, 107, 108, 116, 119, 120, 122, 123, 124. LESSON: some of these 20 may
also be false-completes, not just under-ticked reconciles. Audit each for BOTH.

## Progress (2026-07-10b) -- update
Supersedes the partial status above. Reconciliation now 2 of 23 COMPLETE:
- 023: closed (62dc9e47).
- 028: FULLY closed (f21c0463). All 4 gates honest [x]. Gate 4 (nextest faster-than-cargo-test)
  was measured live, NOT deferred: cargo test ~432ms vs nextest ~561ms at N=26 -> nextest not
  faster at this scale; criterion retired as inapplicable, real numbers in 028's charter.
- INT-137: COMPLETE (cbb845f1 tick, 987712b5 move). devShell was missing SEVEN system libs
  (udev, libxkbcommon, seatd, libdisplay-info, pam, libinput, libgbm); all added; whole
  workspace incl. faelight-compositor now compiles+links in-shell; bacon green. 137 closed via
  cicomplete -- 130's own fixed blocker passed it on zero open gates (the fix, proven in use).

REMAINING: 21 intents -- 032, 064, 065, 091, 097, 098, 099, 100, 101, 103, 104, 105, 106,
107, 108, 116, 119, 120, 122, 123, 124. Next up: 032.

## Progress (2026-07-10)
Reconciliation resumed -- Tier 2/3 audit begun. Now 2 of 23 done:
- 023 (replace-wallpaper-idle): CLOSED. G1/G2 deferred under the pre-approved KEEP-as-Rust
  decision (christian 2026-06-04); G3/G5 verified live (niri uninstalled, wallpaper showing);
  G4 voided by KEEP. Committed 62dc9e47.
- 028 (forest-dev-tooling): PARTIAL, does NOT close yet.
    - nix-tree: [x] present + ran live (built 1665-path tree). Commit b3257eed/ (g1).
    - nvd diffs cleanly: [x] demonstrated 340->341 diff live. Commit b3257eed.
    - bacon watches+rebuilds: [x] demonstrated (watch loop drove a build cycle).
    - cargo-nextest faster-than-cargo-test: [~] NOT YET MET -- tool present + functional
      (0.9.136) but the "faster" claim is UNMEASURED: workspace won't compile in-shell
      until udev + xkbcommon land. Commit a9f005b1.
- INT-137 filed + amended: friday-dev devShell missing udev AND xkbcommon (both smithay
  build-deps). This is what blocks 028 G4. Surfaced by bacon + cargo nextest list this session.

DEPENDENCY: 130's reconciliation gate cannot close until ALL 23 are honestly resolved.
028 cannot fully resolve until 137 lands. Therefore 130 is transitively blocked on 137
via 028. Order to close 130: finish 137 -> close 028 G4 with a real timed comparison ->
reconcile the remaining 21 (032, 064, 065, 091, 097-108, 116, 119-124) -> then G5 closes.

NOTE: charter has the gate list twice (## Gates and ## 130's own gates), byte-identical.
Left as-is deliberately -- cicomplete's gate-scanner reads this file, and the dedupe is
not worth risking the enforcement surface until we've watched a real block read it. Cosmetic,
makes nothing false (both show the same honest open state). Dedupe is its own later task.

## The remaining 23 (Tier 2/3 -- fresh-session audit)
023, 028, 032, 064, 065, 091, 097, 098, 099, 100, 101, 103, 104, 105, 106, 107, 108,
116, 119, 120, 122, 123, 124.
(117, 125, 126, 127 done in Tier 1.)

## 130's own gates
- [ ] Root cause confirmed in code (emoji-vs-markdown mismatch at intent/mod.rs ~928)
- [ ] Blocker rewritten to detect `- [ ]` markdown gates (+ back-compat emoji)
- [ ] Deferral logic updated to match markdown too
- [ ] DEMONSTRATED: cicomplete BLOCKS on an open `- [ ]` gate (shown live), completes once checked
- [ ] Reconciliation pass: every affected intent audited (tick if done / reopen if not)
- [ ] core builds clean, deployed, verified on the running binary
NOTE: several of 130's own gates ARE already met (root cause confirmed, blocker rewritten,
demonstrated live, core built+deployed+verified). They are left unticked deliberately until
the 23-audit gate is also done -- 130 completes as ONE honest unit, and (fittingly) its own
fixed blocker will now enforce that.

INT-332 predates the Arch->NixOS migration (June 1 2026). The gate-blocker has therefore
been a no-op for the ENTIRE NixOS era and likely its whole existence back to Arch -- not
weeks. This is why the 27 affected intents span a wide range (023 onward) rather than
clustering recent: nothing was ever enforced, the whole time.

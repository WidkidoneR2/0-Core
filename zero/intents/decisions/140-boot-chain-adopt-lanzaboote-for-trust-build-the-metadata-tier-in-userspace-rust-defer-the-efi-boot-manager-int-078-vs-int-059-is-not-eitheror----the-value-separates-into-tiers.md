---
id: 140
date: 2026-07-15
type: decision
title: "Boot chain: adopt Lanzaboote for trust, build the metadata tier in userspace Rust, defer the EFI boot manager. INT-078 vs INT-059 is not either/or -- the value separates into tiers."
status: planned
tags: [boot, secureboot, lanzaboote, everglow, decision, 059, 078, 049]
---

## Decision
Split the boot chain into THREE TIERS by what each actually requires. INT-078 (build our own
generation-aware Rust EFI boot manager) and INT-059 (adopt Lanzaboote) were framed as competing
-- 078's own text says it "may replace" 059. They are not competing. The value separates:

1. METADATA TIER -- userspace Rust, build EARLY (weekend-scale).
   Boot notes (what changed), git commit awareness, flake revision, build provenance, timeline,
   labels. ALL of this is text written into Boot Loader Spec Type #1 entry files at rebuild time.
   It renders in ANY boot menu, including the systemd-boot running today. No EFI code. No new
   boot manager. Testable in userspace without rebooting. Works fine ON TOP of Lanzaboote.
2. TRUST TIER -- adopt Lanzaboote (INT-059). Weeks, not months. Proven, maintained, and it is
   the cryptographic chain of trust -- the part where being wrong means a locked-out laptop.
3. EFI APP TIER -- INT-078, DEFERRED (months, post-1.0). Beautiful boot UI, search, diff,
   diagnostics page, recovery-without-booting, theme engine. Genuinely wanted, genuinely months,
   and it sits ON TOP of a trust chain Lanzaboote already solves. Decide later with better
   information -- adopting Lanzaboote does not foreclose it.

Christian's OWN crate split (from the 2026-07-14/15 braindump) already conceded this line without
naming it: faelight-generation / faelight-uki / faelight-secureboot / faelight-cli are ALL
USERSPACE crates; only faelight-boot is EFI. Plus his note: "test most of the logic in normal
Linux userspace without rebooting a VM." The architecture knew before the decision did.

## Evidence (verified 2026-07-15, not assumed)
- The real ESP entry, /boot/loader/entries/nixos-generation-355.conf:
      title NixOS
      sort-key nixos
      version Generation 355 NixOS Yarara 26.05... (Linux 6.18.35), built on 2026-07-12
      linux /EFI/nixos/...-bzImage.efi
  Plain-text BLS Type #1. `title` is JUST A STRING. The whole metadata tier is writing different
  text here.
- `boot.loader.systemd-boot.extraInstallCommands` EXISTS (verified in the option tree) -- the
  hook that runs after entries are installed. That is the seam for rewriting titles.
- BOOT COUNTING DOES NOT EXIST as a NixOS option. Verified: no attr matching count/tries/fallback
  under boot.loader.systemd-boot OR boot.loader. systemd-boot the PROGRAM supports +tries
  suffixes (this morning's guest `bootctl status` listed "Boot counting" among its features), but
  NixOS's generator never emits them. So automatic rollback is NOT free -- it needs real work
  whichever path is taken. (An earlier "one option away" guess was WRONG; the option tree said no.)
- `boot.loader.limine` is also in the tree -- the other Secure Boot route 059 wants to compare.

## Already built -- do NOT build these twice
- Boot profiles (#8) = NixOS specialisations. Exists.
- Custom labels (#18) = the `title` string in the entry file. Exists, just unused.
- Filesystem snapshots (#9) = btrfs, already the disk layout (@root/@home/@nix/@log).

## Rejected: the EFI plugin architecture (#19)
A plugin system inside a TRUSTED boot component is self-contradicting. Either plugins are in the
verification path (so the trusted core is not small -- the boundary moved, it did not shrink), or
they load unsigned code before the OS, which is exactly what Secure Boot exists to prevent. The
idea's own stated goal ("keep the trusted core extremely small") argues against it. Userspace
modules: yes. EFI-loaded modules: no.

## Why now, why this order
Friday starts in ~3 weeks with a full week blocked for it. An EFI boot manager is months. The
metadata tier is a weekend. INT-059 is ~1-2 weeks out and has a MANDATORY VM-rehearsal gate --
and as of 2026-07-15 the VM (INT-027) boots the real chain (OVMF -> systemd-boot -> generation ->
kernel), so that rehearsal is finally possible. Remaining VM prerequisites for 059, found by
booting it: `Secure Boot: disabled (unsupported)` needs the OVMF_CODE.secboot.fd variant, and
`TPM2 Support: no` needs swtpm.

## Consequences
- INT-059 proceeds as the trust chain. Not blocked on 078.
- INT-078 stays filed, DEFERRED, post-1.0 -- and shrinks to the EFI-app tier only. Its userspace
  crates (generation/uki/secureboot/cli) may be built early and independently.
- A metadata-tier intent should be filed separately (entry titles enriched via
  extraInstallCommands: generation + commit + label + what-changed).
- Automatic rollback (boot counting) is its own piece of work -- NOT free, NOT assumed.

## The Rule
"Adopt the chain that must not break. Build the part that makes it yours." 🌲

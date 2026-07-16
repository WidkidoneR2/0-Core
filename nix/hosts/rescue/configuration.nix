# INT-160: Faelight Forest rescue media -- the "what if" USB.
#
# WHY THIS EXISTS: INT-056's Forest Recovery Protocol is complete and its runbook opens
# Level 3 with "Boot the NixOS installer USB". That USB was never built. A recovery
# protocol whose first step is an artifact you do not have is documentation, not recovery.
#
# SCOPE IS DELIBERATELY SMALL. Rescue media's one job is working when everything else does
# not, so every package added is failure surface on the artifact you reach for while already
# in trouble. NOT the whole forest -- no MangoWM, no Friday, no fsh, no home-manager.
#
# KNOWN LIMIT (measured 2026-07-15, INT-160 gate 3): this ISO will NOT boot while Secure
# Boot is enforcing. NixOS installation media is unsigned and carries no MS-signed shim --
# verified on Christian's own nixos-minimal-25.11 stick (unsigned GRUB BOOTX64.EFI +
# refind_x64.efi, no shimx64.efi) and proven by attaching it to the enforcing VM at
# bootindex=0: "Access Denied -- rejected probably by Secure Boot", then it fell through to
# the signed disk and booted normally -- NOTHING VISIBLE happens.
# So the USB is NOT the escape hatch; the FIRMWARE MENU is:
#     firmware -> DISABLE Secure Boot -> boot this USB -> fix -> re-enable
# It is still worth building: every non-SB failure (panicking kernel, unbootable generation,
# botched disko, LUKS header damage, dying NVMe) is a case where SB is irrelevant and this
# is the only tool.
{ lib, pkgs, modulesPath, ... }:
{
  # Path verified 2026-07-16 by find(1) against the store, not from memory:
  #   <nixpkgs>/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix
  imports = [ "${modulesPath}/installer/cd-dvd/installation-cd-minimal.nix" ];

  networking.hostName = lib.mkForce "faelight-rescue";
  # nixpkgs 26.05 renamed isoImage.isoBaseName -> image.baseName (told us at eval time).
  image.baseName = lib.mkForce "faelight-rescue";

  # Each tool here has a REASON. If you cannot name the failure it addresses, it does not
  # belong on rescue media.
  environment.systemPackages = with pkgs; [
    cryptsetup      # LUKS unlock -- /dev/nvme0n1p2 is crypto_LUKS. Runbook Level 3 step 1.
    btrfs-progs     # mount @root/@nix/@home subvolumes. Runbook Level 3 step 2.
    dosfstools      # fsck.vfat. On 2026-07-15 a truncated ESP file (dir entry present,
                    # cluster chain length 0) made a VM unbootable AND forced the ESP
                    # read-only. fsck was the ONLY way in. This one is not theoretical.
    sbctl           # sbctl verify/sign -- inspect and repair the ESP under Secure Boot.
    sbsigntool      # sbsign/sbverify -- lower-level than sbctl, when sbctl cannot help.
    efibootmgr      # read/repair EFI boot entries when the boot order is the problem.
    nixos-install-tools  # nixos-enter -- the runbook's Level 3 verb.
  ];

  # The runbook belongs ON the media. Docs on the machine that will not boot are not docs.
  environment.etc."faelight/recovery-runbook.md".source = ../../../docs/recovery-runbook.md;

  # Findable without knowing where to look. A rescue tool you have to remember how to use
  # is not a rescue tool.
  users.motd = ''
    ==========================================================
      FAELIGHT FOREST RESCUE MEDIA            (INT-160)
    ==========================================================
      RUNBOOK:  /etc/faelight/recovery-runbook.md
                less /etc/faelight/recovery-runbook.md

      SECURE BOOT LOCKOUT -- the SHORT path (no LUKS needed):
        The ESP cannot be encrypted; firmware reads it before
        any OS exists. So this is mount ONE vfat partition,
        fix ONE file, reboot:
          mount /dev/nvme0n1p1 /mnt
          sbctl verify            # what is unsigned?
          ... restore or re-sign ...
          umount /mnt && reboot

      CANNOT BOOT AT ALL -- runbook Level 3 (needs LUKS):
          cryptsetup open /dev/nvme0n1p2 cryptroot
          mount -o subvol=@root /dev/mapper/cryptroot /mnt
          mount -o subvol=@nix  /dev/mapper/cryptroot /mnt/nix
          mount /dev/nvme0n1p1 /mnt/boot
          nixos-enter --root /mnt

      Disk layout: nvme0n1p1 = ESP (vfat, /boot)
                   nvme0n1p2 = LUKS -> btrfs @root/@home/@nix/@log
    ==========================================================
  '';
}

# Framework 16 factory Secure Boot keys

Captured 2026-07-16 (INT-161), BEFORE step 0 erases the Platform Key.

    sudo nix shell nixpkgs#sbctl -c sbctl --disable-landlock \
      export-enrolled-keys --dir ~/sb-backup --format esl

## These are PUBLIC certificates. That is why they are in the repo.

X.509 certs, not keys. Framework's private keys live in Framework's HSM and were never on this
machine. There is nothing secret here, so the repo is a legitimate off-machine home -- and GitHub
survives this laptop being a brick, which a USB stick in a drawer may not.

By contrast /var/lib/sbctl/keys/*/*.key ARE private and MUST NEVER be committed. Those need physical
media.

## What each file is

    PK.esl   1.4k  Platform Key -- CN=frame.work-LaptopAMDPK.
                   Matches the firmware menu's "Enroll PK signature list: [PKCS7]
                   framework-laptopAMDPK". Step 0 ("Erase all secure boot settings") destroys this.
    KEK.esl  2.8k  Key Exchange Keys -- authorise db changes.
    db.esl   7.4k  Signature database. FIVE certs, decoded 2026-07-16:
                     [0] Microsoft Windows Production PCA 2011      expires Oct 19 2026
                     [1] Microsoft Corporation UEFI CA 2011         EXPIRED Jun 27 2026
                     [2] frame.work-LaptopAMDDB                     expires Oct 14 2120
                     [3] Microsoft UEFI CA 2023                     expires Jun 13 2038
                     [4] Microsoft Option ROM UEFI CA 2023          expires Oct 26 2038
                   (An earlier version of this file said THREE. That was wrong -- it described
                   dbDefault while claiming to describe db.esl. They are different variables.)

## db vs dbDefault -- they are NOT the same, and the difference decided the flag

    dbDefault  = the FACTORY default set. THREE certs: the two 2011 Microsoft CAs + frame.work-
                 LaptopAMDDB. Frozen at manufacture.
    db         = what is ACTUALLY ENROLLED right now. FIVE certs: those three PLUS Microsoft UEFI CA
                 2023 and Microsoft Option ROM UEFI CA 2023.

Framework pushed the 2023 CAs into db via a firmware update. dbDefault was never updated.

THIS IS WHY --firmware-builtin WOULD HAVE BEEN A DOWNGRADE, not just a compromise. It enrols FROM
dbDefault, so it would have given us both 2011 Microsoft CAs (one already expired) and DROPPED both
2023 replacements. Worse than the machine's current state.

INT-161 uses --custom with only the frame.work-LaptopAMDDB cert. All four Microsoft certs go.

Cert [4], "Microsoft Option ROM UEFI CA 2023", is the OpROM signer. Framework ships it -- but PCR 2
is EMPTY on this machine (measured, zero EV_EFI_BOOT_SERVICES_DRIVER events), so nothing is being
validated against it. The cert list and the TPM eventlog agree: no option ROM enters this boot
chain.

    No dbx.esl -- the revocation list is EMPTY on this machine. Not an omission.

## This backup is belt-and-braces, not the only way back

The INSYDE menu has "Restore secure boot to factory settings", which restores these from the
firmware's own storage -- no file, no USB, no network. This copy exists in case that option fails or
does something other than what its name says.

Restoring by hand, if it ever comes to that: boot the INT-160 rescue USB, then use efi-updatevar /
sbctl --custom-bytes with --partial. Do NOT attempt this without reading INT-161's recovery section
first.

## Where these live, and why here

MOVED 2026-08-27 from nix/hosts/framework16/. These are hardware facts about this
Framework 16 -- public X.509 certificates captured from its firmware -- and they have
nothing to do with which operating system runs on it. They outlived NixOS and they will
outlive Omarchy.

## Decode any of these

    sig-list-to-certs db.esl out
    openssl x509 -in out-0.der -inform der -noout -subject -dates

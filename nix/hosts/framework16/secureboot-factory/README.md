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
    db.esl   7.4k  Signature database. THREE certs, decoded 2026-07-16:
                     Microsoft Windows Production PCA 2011    expires Oct 19 2026
                     Microsoft Corporation UEFI CA 2011       EXPIRED Jun 27 2026
                     frame.work-LaptopAMDDB                   expires Oct 14 2120
                   This is why --firmware-builtin was the wrong flag: it enrols FROM dbDefault, so it
                   would have pulled both Microsoft CAs back in. INT-161 uses --custom with only the
                   frame.work-LaptopAMDDB cert.

    No dbx.esl -- the revocation list is EMPTY on this machine. Not an omission.

## This backup is belt-and-braces, not the only way back

The INSYDE menu has "Restore secure boot to factory settings", which restores these from the
firmware's own storage -- no file, no USB, no network. This copy exists in case that option fails or
does something other than what its name says.

Restoring by hand, if it ever comes to that: boot the INT-160 rescue USB, then use efi-updatevar /
sbctl --custom-bytes with --partial. Do NOT attempt this without reading INT-161's recovery section
first.

## Decode any of these

    nix shell nixpkgs#efitools -c sig-list-to-certs db.esl out
    nix shell nixpkgs#openssl -c openssl x509 -in out-0.der -inform der -noout -subject -dates

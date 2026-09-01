# Faelight Forest — Recovery Runbook

> When the system won't let you in: stay calm, work top-down. Most problems
> are solved at Level 0–2 without a USB. A USB is only needed when it won't boot.
>
> **Machine:** Framework 16 · NixOS · LUKS2 + btrfs · MangoWM via greetd
> **Flake target:** `/home/christian/0-core#framework16`
> **Disk:** `/dev/nvme0n1` — ESP `p1` → `/boot`, LUKS `p2` → `cryptroot` (btrfs)
> **Subvols:** `@root`→`/`, `@home`→`/home`, `@nix`→`/nix`, `@log`→`/var/log`

## Key sequences (memorize these)

| Action | Keys | Works when |
|---|---|---|
| Escape mango → console | `Super+Ctrl+2` | mango responsive |
| Escape mango → console | `Fn+Ctrl+Alt+F2` | always (kernel) |
| Return console → mango | `Fn+Ctrl+Alt+F1` | always (kernel) |

**Framework note:** the top row defaults to media keys, so real F-keys need `Fn`.
Hold `Fn` *first*, then `Ctrl+Alt+F2`. If it's finicky, `Fn+Esc` toggles
function-lock so `Ctrl+Alt+Fn` works without holding `Fn`.

The VT switch (`Ctrl+Alt+Fn`) is a **kernel** function — it works even if mango
is completely hung. `tty2`–`tty6` are available.

---

## Level 0 — mango frozen (system still running)

1. `Fn+Ctrl+Alt+F2` → `tty2`, log in as `christian`.
2. Look, then act:
   - `journalctl -b -e` — recent errors this boot
   - `sudo systemctl restart greetd` — restart the login stack (kills frozen mango, fresh login)
3. `Fn+Ctrl+Alt+F1` back to the graphical VT.

## Level 0.5 — a compositor won't start (SafeShell = the 100% recovery)
If you reach the greeter but a compositor (mango / Pinnacle / Miracle) fails to
launch or black-screens, you do NOT need tty2 — the greeter itself offers a
rescue session, and it is the simplest, most reliable recovery there is:
1. At the greeter, press **F3** (session picker) → select **SafeShell**.
2. Log in as `christian`. SafeShell is a bare `nsh` on the VT with NO compositor
   — a full working shell to repair from.
3. Fix the flake / roll back / inspect, then `exit` → pick a working session
   (e.g. MangoWM) at the greeter.
SafeShell is defined in the flake (`environment.etc."greetd/sessions/safeshell.desktop"`,
Exec=nsh) and is ALWAYS present as a picker option. A broken compositor can never
lock you out while SafeShell exists. Proven VM + metal (INT-056, 2026-07-11).

## Level 1 — can't log in (greetd/session broken) but it boots

1. `Fn+Ctrl+Alt+F2` → `tty2`, log in as `christian`.
2. Inspect: `journalctl -u greetd -b`
3. Fix the flake, then rebuild:
   - `cd ~/0-core`
   - `sudo nixos-rebuild switch --flake .#framework16`
4. If the last rebuild caused it → roll back (Level 2).

## Level 2 — roll back to a working generation (it boots)

The fast undo.

**From a tty:**
```
sudo nix-env --list-generations --profile /nix/var/nix/profiles/system
sudo nixos-rebuild switch --rollback
```

**From the boot menu (no login needed):** reboot, select a previous generation.
- systemd-boot: listed directly.
- GRUB: under "NixOS — All configurations".
> Confirm which: `rg boot.loader ~/0-core/nix/hosts/framework16/*.nix`

## Level 3 — won't boot / fully locked out (USB rescue)

> **FIRST: Secure Boot will reject the USB.** Since INT-161 this machine enforces
> Secure Boot with custom keys, and the rescue media is not signed by them.
> Measured twice: it does not error. The firmware silently falls through and boots
> the signed disk, so it looks like the port is dead or the stick is blank.
>
> Before anything below, enter the firmware menu:
>
>     systemctl reboot --firmware-setup      # or the boot-menu key at power-on
>
> Disable Secure Boot there, then boot the USB. No supervisor password is set
> (confirmed 2026-08-18), so the menu is directly reachable.
>
> After the repair, before re-enabling Secure Boot:
>
>     sbctl verify        # confirms every UKI on the ESP is signed
>
> Then re-enable Secure Boot in the firmware menu.
>
> If the keys themselves are gone, use "Restore secure boot to factory settings"
> in the INSYDE menu. It restores from the firmware's own storage -- no file, no
> USB, no network. The factory certs and your sbctl keys are on FORESTBACKUP
> (INT-225). They are NO LONGER IN THIS REPO -- removed
> 2026-08-29 because the boot entries describe this machine and the repo is
> public. Off-machine copies only: FORESTBACKUP holds the unredacted dump at
> faelight-secureboot/, and ~/secureboot-framework16/ holds the redacted set
> until it moves to the external drive.

Boot the NixOS installer USB, then:

```
sudo -i

# 1. Confirm the disk is nvme0n1
lsblk -f

# 2. Unlock LUKS (prompts for your passphrase)
cryptsetup luksOpen /dev/nvme0n1p2 cryptroot

# 3. Mount: @home holds the flake, @nix the store, p1 the bootloader — all required
mount -o subvol=@root,compress=zstd,noatime /dev/mapper/cryptroot /mnt
mount -o subvol=@home,compress=zstd,noatime /dev/mapper/cryptroot /mnt/home
mount -o subvol=@nix,compress=zstd,noatime  /dev/mapper/cryptroot /mnt/nix
mount /dev/nvme0n1p1 /mnt/boot
mount -o subvol=@log,compress=zstd,noatime  /dev/mapper/cryptroot /mnt/var/log   # optional

# 4. Enter the installed system
nixos-enter --root /mnt
```

Inside the real system — fix and rebuild, or roll back:
```
hx /home/christian/0-core/nix/hosts/framework16/configuration.nix
nixos-rebuild switch --flake /home/christian/0-core#framework16
# or undo the last change:
nixos-rebuild switch --rollback
```

Exit and reboot:
```
exit
umount -R /mnt
reboot
```

> Mountpoint dirs (`/mnt/home`, `/mnt/nix`, `/mnt/var/log`, `/mnt/boot`) already
> exist inside `@root`, so no `mkdir` needed.

---

## Notes / gotchas

- **`nixos-enter` beats a manual chroot** — it wires up `/proc`, `/sys`, `/dev`,
  and the nix daemon env for you. Use it.
- **Never run `disko` to "fix" anything** — `disko.nix` *formats* `/dev/nvme0n1`.
  It is install-only.
- **No network on the USB?** `--rollback` and booting an older generation work
  offline from the existing `/nix` store. A fresh `--flake` rebuild that needs to
  fetch will not.
- This file documents the *procedure only*. Your LUKS passphrase and login
  password are never written here — they stay in your head.

## Why this exists

2026-06-09: a greetd/tuigreet change locked login for ~24h. The rules that came
out of it: every login-touching change is rehearsed in the lab VM (INT-024)
before bare metal, and there is always a kernel-level escape (`Fn+Ctrl+Alt+F2`)
plus a known-good generation to roll back to.

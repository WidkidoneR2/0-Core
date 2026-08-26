# Migration runbook -- NixOS to Omarchy

Written 2026-08-26, after rehearsing every step in a VM. Keep a copy on the USB stick and readable
from the phone: once the disk is wiped this machine cannot reach anything you did not carry.

## The one rule
**VERIFY EVERY COPY BY SIZE, NOT BY EXIT CODE.** The rehearsal caught `cp` reporting success while
writing 258K of a 266MB file. Nothing else in this document matters as much: a silently truncated
`state.db` discovered after the wipe is unrecoverable.

    ls -l <file>        # both sides, compare the byte counts
    266448896           # state.db snapshot, exactly

## What is already proven
- Omarchy 4.0.1 installs in **15 seconds** (it is an image copy, not a package install).
- `faelight-shell` builds on Omarchy in **30 seconds**, no patches, no source changes.
- `core` builds clean. `core doctor run` works; 53% on a bare machine, every gap explained by
  "nothing deployed yet" or "no state present" -- not portability.
- A restored `state.db` opens, reads, AND writes. fsh shows real history from it.
- The ISO checksum is verified: `69cbb4e10d98ad831c3c9f245b5757a9d1fedfd0c9592780e977d6f950dea8c3`

## Facts about Omarchy learned in rehearsal
- **git ships by default (2.55.0). rust does NOT.** `sudo pacman -S --noconfirm rust`
- A fresh install may have a stale package DB -- `sudo pacman -Sy` first if a package is
  "target not found".
- systemd, glibc, Hyprland. Every `systemctl` call in the engine works natively.
- **9p shared folders do NOT mount** even with `9pnet_virtio` loaded. Do not plan around them.
- SSH port-forwarding into a qemu guest never worked across a whole session. Not needed on metal.

## BEFORE the wipe -- prepare, in this order

### 1. Fresh snapshot, with fsh stopped
The rehearsal snapshot is dated. Take a new one so no tables differ:

    python3 -c "
    import sqlite3
    c = sqlite3.connect('file:/home/christian/.local/state/faelight/state.db?mode=ro', uri=True)
    c.execute('VACUUM INTO ?', ('/home/christian/state-final.db',))
    c.close()"

`VACUUM INTO` folds the WAL in and writes ONE consistent file. A plain `cp` of a live database with
a 4MB WAL gives you a database missing recent writes.

### 2. Verify it before trusting it

    python3 -c "
    import sqlite3
    c = sqlite3.connect('file:/home/christian/state-final.db?mode=ro&immutable=1', uri=True)
    for t in ['shell_history','friday_knowledge','intent_commits','shell_aliases','command_registry']:
        print(t, c.execute(f'SELECT count(*) FROM {t}').fetchone()[0])
    c.close()"

Expect roughly: shell_history 178k+ · friday_knowledge 915+ · intent_commits 3329 ·
shell_aliases 284 · command_registry 407. `immutable=1` avoids the read-only lock error.

### 3. Two USB sticks. NEVER the same one.
- **Stick A -- installer.** Omarchy ISO written to it. Wiping is expected.
- **Stick B -- payload.** btrfs or ext4, NOT vfat: vfat loses permissions, and an SSH private key
  arriving world-readable is refused by ssh. (`mkfs.ext4` is NOT on this NixOS system;
  `/run/current-system/sw/bin/mkfs.btrfs` is.)

Stick B contents (~400MB total):

    state-final.db                  the snapshot
    payload/.ssh                    keys -- without these you cannot push to GitHub
    payload/.gitconfig
    payload/sb-backup               PK.esl KEK.esl db.esl -- factory Secure Boot keys, ONLY copy
    payload/faelight                ~/.config/faelight
    payload/faelight-shell          ~/.config/faelight-shell -- COPY WITH `cp -rL`
    payload/MIGRATION-RUNBOOK.md    this file

⚠️ **`-L` IS NOT OPTIONAL.** `~/.config/faelight-shell/config.fsh` is a SYMLINK into
`/nix/store/...home-manager-files/`. Copy it without dereferencing and Omarchy gets a dangling link:
fsh starts with zero aliases and zero settings and nothing says why.

### 4. Confirm everything is pushed

    cd ~/0-core && git status --short && git log origin/nixos..HEAD --oneline

Must both be empty. **The code comes down from GitHub, not the stick.** Anything uncommitted is lost.

### 5. Read the payload back from the stick
Mount it, count the rows again, `ls -la payload/.ssh` and confirm `drwx------`. A backup you have
not read is a hope.

## THE WIPE

### 6. Secure Boot OFF -- and know why
Current state: **enabled, setup mode disabled, CUSTOM keys enrolled** (Lanzaboote's).

⚠️ Those keys live in FIRMWARE, not on disk. Wiping the disk does not remove them, so an Omarchy
install with Secure Boot still on will produce a system signed by nothing the firmware trusts, and
it will not boot. **Disable Secure Boot in firmware BEFORE booting the installer** -- not after.

`sb-backup/*.esl` are the factory keys if you ever want the machine back to how it shipped.
Omarchy can re-enroll its own later with `sbctl`; that is a decision for afterwards, not now.

### 7. Install
Boot stick A. The installer asks: keyboard, name, password, full name (skip), email (skip),
hostname, timezone, disk.

⚠️ **THE DISK SCREEN IS THE IRREVERSIBLE STEP.** In rehearsal it offered one empty 60G disk. On
metal it will offer your 3.6T NVMe with NixOS on it. Read it twice.

⚠️ **USERNAME MUST BE `christian`.** `state.db` and the config carry `/home/christian` paths.

Note: `/home` is currently LUKS-encrypted (`cryptroot`). Once the disk is repartitioned the old
filesystem is gone -- there is no casually mounting it afterwards to grab a forgotten file. This is
why step 5 exists.

## AFTER -- restore, in this order

### 8. Toolchain and clone

    sudo pacman -Sy
    sudo pacman -S --noconfirm rust
    cd ~ && git clone https://github.com/WidkidoneR2/0-Core.git 0-core

(Not `--depth 1` on the real machine -- you want the history.)

### 9. Restore state, and VERIFY THE SIZE

    mkdir -p ~/.local/state/faelight
    cp /path/to/stick/state-final.db ~/.local/state/faelight/state.db
    ls -l ~/.local/state/faelight/state.db          # ⚠️ COMPARE THE BYTE COUNT
    chown -R christian:christian ~/.local/state/faelight

⭐ The directory must be writable too, not just the file -- SQLite creates its WAL and shm files
NEXT TO the database.

### 10. Config and keys

    mkdir -p ~/.config
    cp -r /path/to/stick/payload/faelight ~/.config/
    cp -r /path/to/stick/payload/faelight-shell ~/.config/
    cp -r /path/to/stick/payload/.ssh ~/
    chmod 700 ~/.ssh && chmod 600 ~/.ssh/id_ed25519
    cp /path/to/stick/payload/.gitconfig ~/

### 11. Build and check

    cd ~/0-core
    cargo build -p faelight-shell -p core
    ./target/debug/core version        # ⭐ Friday should report your real fact count
    ./target/debug/faelight-shell      # ⭐ press UP -- your history should be there

If Friday reports 0 facts, the database did not restore. Stop and re-copy from the stick before
doing anything else.

## What is NOT solved yet, and is fine
These are all recoverable AFTER the move, with the same tools:
- **Ctrl+Z suspends fsh itself** (INT-188). No process groups, no terminal foreground ownership.
  Reproduced live on Void. Bash's `fg` recovers it.
- **`fsh -c` delegates to `sh`** -- no builtins, aliases, spine or guard apply through that door.
- **The banner still reports `0 done · 0 tools · 0 planned`** on a machine with no ledger --
  confident zeros the doctor no longer tells.
- **Compositor keybinds**: mango's config will not apply. Omarchy runs Hyprland.
- **faelight-logout** needs `python-gobject` and `gtk4-layer-shell` from pacman. It uses
  `systemctl poweroff/reboot`, which Omarchy has.
- **The spine flip** (INT-169) -- deliberately AFTER the move, so a failure names one boundary
  rather than two.

## If something goes wrong
The phone is the lifeline. The things worth having on it: this file, the GitHub repo URL, and the
fact that **the installer stick can boot a live environment** -- from which the payload stick is
readable and the old disk is not yet overwritten if you have not confirmed the disk screen.

Nothing about the tools is fragile. The code is on GitHub, the state is on the stick, and both have
been proven to work on Omarchy. The only irreversible moment is the disk screen in step 7.

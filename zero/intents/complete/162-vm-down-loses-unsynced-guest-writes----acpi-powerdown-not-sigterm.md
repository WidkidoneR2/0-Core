---
id: 162
date: 2026-07-15
type: future
title: "vm down loses unsynced guest writes -- ACPI powerdown, not SIGTERM"
status: complete
tags: [faelight, vm, qemu, bug, data-loss]
---

## Vision
`vm down` stops the VM the way a POWER BUTTON does, not the way a power CORD does.

## The Problem -- found by accident, cost real hours
`vm down` forwarded straight to `faelight-vm kill`, i.e. SIGTERM to qemu. QEMU'S SIGTERM HANDLER
EXITS -- it does not press the guest's power button. The guest dies mid-write and its dirty page
cache dies with it. vfat has no journal: an unflushed write is simply gone.

FOUND DURING THE INT-059 SECURE BOOT REHEARSAL, where it did real damage:
1. It TRUNCATED a real ESP file -- fsck: "File size is 156336 bytes, cluster chain length is 0
   bytes". A file that existed and could not be read.
2. That forced the ESP to remount READ-ONLY (errors=remount-ro), which looked like a permissions
   bug and sent the session chasing a ghost.
3. Worst: the truncated file made the firmware say "Not Found" instead of "Access Denied --
   rejected probably by Secure Boot" -- and THAT got written up as a finding about Secure Boot
   ("the failure never mentions Secure Boot, it reads like a dead SSD"). It was our own bug
   masquerading as firmware behaviour. The finding had to be retracted and corrected.
A tooling bug that corrupts test data does not just cost the test -- it manufactures false
findings and gets them committed.

MINIMAL REPRO (2026-07-15) -- fails on the old code, every time:
    vm ssh 'sudo cp <154112-byte file> /boot/EFI/BOOT/bugtest.bin'    # NO sync
    vm down ; vm up
    vm ssh 'sudo ls -la /boot/EFI/BOOT/bugtest.bin'
    -> ls: cannot access '/boot/EFI/BOOT/bugtest.bin': No such file or directory
NOT truncated -- GONE. Directory entry and all. One cp, one down/up, 154KB evaporated silently.

## The Solution
The launcher exposed NO monitor and NO qmp socket (verified in the live qemu cmdline), so there was
no channel able to reach qemu at all. Added one:
- `vm` script: `-monitor unix:$STATE/monitor.sock,server,nowait` in QEMU_OPTS
- new `faelight-vm down`: connect to the monitor, send `system_powerdown` (a virtual ACPI
  power-button press), poll vm_procs() until qemu exits, fall back to `cmd_kill()` on timeout
- `cmd_kill` UNCHANGED -- the deliberate yank stays available for `vm unlock` and the janitor. It
  just is not the default any more. The verbs now mean what they say: kill kills, down shuts down.
- Uses std::os::unix::net::UnixStream -- ZERO new dependencies (ssh_ready() already reaches into
  std::net the same way), and no socat/nc added to the script's closure.
Why the monitor and not `ssh poweroff`: the monitor talks to QEMU, so it works when the guest's
sshd is dead, when it is mid-boot, or when it is sitting in the OVMF firmware menu -- exactly when
a clean stop matters most.

HONEST LIMIT, stated in the source: ACPI only works if the guest RESPONDS. A hung kernel ignores
it and we fall back to the yank, data loss included. This makes the COMMON case correct; it cannot
make the pathological case safe.

## Success Criteria
- [x] Bug reproduced minimally BEFORE fixing anything
<!-- evidence: 2026-07-15. cp of a 154112-byte file to the guest ESP with no sync, then vm down/up
-> "ls: cannot access bugtest.bin: No such file or directory". Gone entirely, not truncated. -->
- [x] The VM exposes a qemu monitor socket
<!-- evidence: -monitor unix:$STATE/monitor.sock,server,nowait added to QEMU_OPTS in
faelight/packages/faelight/scripts/vm. Verified absent beforehand by reading the live qemu
cmdline out of /proc -- no -monitor, no -qmp existed. cmd_up now rm -f's a stale socket first;
qemu will not bind over one. -->
- [x] `faelight-vm down`: ACPI powerdown -> wait -> kill as fallback
<!-- evidence: cmd_down(25) in faelight/rust-tools/faelight-vm/src/main.rs, dispatched as
["down"]. cargo check -p faelight-vm clean. -->
- [x] PROVEN: an unsynced guest write SURVIVES vm down / vm up
<!-- evidence: 2026-07-15, on the DEPLOYED binary. survivor.bin written ONCE (no sync), then
vm down -> "ACPI powerdown sent -- waiting for the guest to flush and halt..." / "guest shut down
cleanly (3s -- writes flushed)" -> vm up -> file present:
  -rwxr-xr-x 1 root root 154112 Jul 15 23:10 /boot/EFI/BOOT/survivor.bin
  9e57fca70de2338632430004dcc2f234
Same file, same md5, same conditions under which bugtest.bin vanished twenty minutes earlier. A
FIRST attempt at this test was INVALID -- it re-cp'd the file after the reboot and then measured
the new copy. Caught before it was claimed as a pass. -->
- [x] The fallback path ANNOUNCES itself instead of silently eating data
<!-- evidence: the first vm down after deploy hit a VM launched before the socket existed and
printed "no monitor socket -- this VM launched before the graceful path existed" / "falling back
to kill: UNSYNCED GUEST WRITES MAY BE LOST. Relaunch to get it." Three fallback paths (no socket,
socket unreachable, ACPI ignored) each say so. The old code lost data in silence. -->
- [x] `vm down` still clears swtpm and the launcher
<!-- evidence: cmd_down calls cmd_kill() after a clean qemu exit -- swtpm and the launcher hold no
guest data, and a zombie swtpm owns the tpm socket and poisons the NEXT launch (INT-159). -->

## Reference
- INT-027 -- the proving ground this defends; the organic rule (new capability in Rust) followed
- INT-059 -- where the bug did its damage and manufactured a false finding
- INT-159 -- vm_procs() state-dir-scoped discovery, reused here

# Forest Resilience Runbooks

*When hardware argues, the forest keeps working.*

---

## Keyboard-Only Mode (when mouse dies)

### Compositor window navigation
> ⚠️ NEEDS MANGO STEPS -- niri-era commands below are retired (INT-085). See INT-056 / docs/recovery-runbook.md for current mango recovery.
| Action | Keybind |
|--------|---------|
| Focus left/right | `Super+h/l` |
| Focus up/down | `Super+j/k` |
| Move window left/right | `Super+Shift+h/l` |
| Move window up/down | `Super+Shift+j/k` |
| Switch workspace | `Super+1-5` |
| Close window | `Super+q` |
| Center focused window | `Super+Ctrl+Space` ← cursor recovery |
| Open terminal | `Super+Enter` |
| Open menu/launcher | `Super+Escape` |
| Open browser | `Super+b` |
| Open files (yazi) | `Super+Shift+y` |
| Lock screen | `Super+Ctrl+Escape` |

### nsh keyboard-only workflow
- All forest vocabulary works without mouse: `list`, `find`, `delete`, `show`
- File navigation: `yazi` (full keyboard navigation)
- Clipboard: `wl-copy` / `wl-paste` from nsh
- `friday` command for AI assistance without mouse

### If cursor goes off-screen
1. `Super+Ctrl+Space` -- centers focused window (cursor follows)
2. `Super+h/l/j/k` -- refocus a different window
3. `niri msg action center-column` -- from terminal as fallback

---

## Hardware Recovery Runbooks

### Bluetooth failure (2026-05-15 incident pattern)
Symptoms: `bluetoothctl` says "no default controller available"
Root cause: Intel AX210 -22 firmware error after sleep/wake
Step 1: Check for -22 error
journalctl -b | grep -i bluetooth | tail -20
Step 2: rfkill cycle
sudo rfkill block bluetooth && sudo rfkill unblock bluetooth
sudo systemctl restart bluetooth
Step 3: If still failing -- reload kernel modules
sudo rmmod btusb btintel
sudo modprobe btintel btusb
Step 4: If firmware missing
ls /lib/firmware/intel/ibt-*
sudo pacman -S linux-firmware-whence  # if missing
Step 5: Nuclear option
sudo reboot
Permanent fix (prevents autosuspend issues)
echo "options btusb enable_autosuspend=n" | sudo tee /etc/modprobe.d/btusb.conf

### Logi Bolt receiver pairing
Install solaar if needed
sudo pacman -S solaar
Pair device
solaar pair
If cursor drifts after pairing
→ Unplug receiver immediately, try different USB port
Check device status
solaar show

### Mouse alternatives (priority order)
1. Logi Bolt USB receiver -- plug into USB port near charging port
2. Framework laptop trackpad -- always available
3. Keyboard-only mode -- documented above

---

## Compositor Recovery

The forest runs three compositors (mango daily + Pinnacle + Miracle), all selectable
at the greeter, all under the SafeShell net. Recovery is layered (INT-056, proven VM +
metal 2026-07-11). Full detail: `docs/recovery-runbook.md`.

### SafeShell -- the 100% recovery (do this first)
If a compositor fails to launch or black-screens, you do NOT need a TTY. At the greeter,
press **F3** -> select **SafeShell** -> a bare `nsh` with no compositor, to repair from.
SafeShell is always in the picker; a broken compositor can never lock you out.

### Compositor crash / frozen session (mango)
1. `Fn+Ctrl+Alt+F2` -> tty2, log in as `christian` (Fn is a Framework media-row quirk).
2. `sudo systemctl restart greetd` -- restarts the login stack (kills the frozen
   compositor, returns to a fresh greeter).
3. `Fn+Ctrl+Alt+F1` back to the graphical VT.

### VT switch
- To a console: `Fn+Ctrl+Alt+F2` (-> tty2). A **kernel** function -- works even if the
  compositor is fully hung. Provided by autovt (always present).
- Back to the session: `Fn+Ctrl+Alt+F1`.
- Confirmed from mango, Pinnacle, and the greeter (greetd foreground) -- VM + metal.


## Note on Pinnacle Migration
When the second compositor (Miracle, INT-087) is added:
- Keybind names will change (Pinnacle uses different action names)
- Core philosophy stays the same: every action has a keyboard shortcut
- BindState with layer_stack means keyboard-only mode becomes a formal MODE
- `Super+Ctrl+Space` → equivalent Pinnacle action for cursor recovery

---
*"The forest must work in the rain.
It must work without the mouse.
Resilience is not a feature.
It is the foundation."* 🌲

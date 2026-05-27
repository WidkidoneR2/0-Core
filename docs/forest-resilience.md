# Forest Resilience Runbooks

*When hardware argues, the forest keeps working.*

---

## Keyboard-Only Mode (when mouse dies)

### Niri window navigation
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

### fsh keyboard-only workflow
- All forest vocabulary works without mouse: `list`, `find`, `delete`, `show`
- File navigation: `yazi` (full keyboard navigation)
- Clipboard: `wl-copy` / `wl-paste` from fsh
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

### Niri crash / frozen session
From TTY (Ctrl+Alt+F2)
systemctl --user restart niri
Or kill and restart
pkill niri
niri &

### VT switch stuck
Switch to TTY
Ctrl+Alt+F2
Switch back to Wayland session
Ctrl+Alt+F1  (or F7 depending on session)
From TTY: restart session
systemctl --user restart niri

---

## Note on Pinnacle Migration
When faelight-compositor v3 (Pinnacle-based) replaces Niri:
- Keybind names will change (Pinnacle uses different action names)
- Core philosophy stays the same: every action has a keyboard shortcut
- BindState with layer_stack means keyboard-only mode becomes a formal MODE
- `Super+Ctrl+Space` → equivalent Pinnacle action for cursor recovery

---
*"The forest must work in the rain.
It must work without the mouse.
Resilience is not a feature.
It is the foundation."* 🌲

# ═══════════════════════════════════════════════════════════
# 0-Core Profile Export
# Name: gaming
# Exported: 2026-01-05 19:46:22
# System: Arch Linux
# Version: 4.1.0
# ═══════════════════════════════════════════════════════════

# gaming.profile
# Description: Maximum GPU performance for gaming
#
# ACTIVATES:
#   - Performance power mode
#   - Disable notifications (DnD)
#   - Disconnect VPN (lower latency)
#
# DEACTIVATES:
#   - VPN
#   - Normal notifications

[activate]
powerprofilesctl set performance
makoctl mode -a dnd
mullvad disconnect
notify-send "Profile: Gaming" "Performance mode active 🎮"

[deactivate]
powerprofilesctl set balanced
makoctl mode -r dnd

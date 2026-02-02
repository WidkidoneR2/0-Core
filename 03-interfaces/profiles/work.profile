# work.profile
# Description: Focus mode with VPN for work
#
# ACTIVATES:
#   - Balanced power mode
#   - VPN connected
#   - Do Not Disturb (focus)
#
# DEACTIVATES:
#   - Distracting notifications

[activate]
powerprofilesctl set balanced
mullvad connect
makoctl mode -a dnd
notify-send "Profile: Work" "Focus mode active 💼"

[deactivate]
makoctl mode -r dnd

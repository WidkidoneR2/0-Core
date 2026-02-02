# default.profile
# Description: Balanced daily driver - the baseline state
#
# ACTIVATES:
#   - Balanced power mode
#   - Hybrid GPU
#   - Normal notifications
#
# DEACTIVATES:
#   - Nothing (this is the baseline)

[activate]
powerprofilesctl set balanced
makoctl mode -r dnd
notify-send "Profile: Default" "Balanced mode active 🏠"

[deactivate]
# Nothing to deactivate - default is the baseline

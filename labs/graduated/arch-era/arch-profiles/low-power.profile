# low-power.profile
# Description: Battery optimization for extended use
#
# ACTIVATES:
#   - Power saver mode
#   - Disconnect VPN (saves power)
#   - Normal notifications
#
# DEACTIVATES:
#   - Performance features

[activate]
powerprofilesctl set power-saver
mullvad disconnect
notify-send "Profile: Low Power" "Battery saver active 🔋"

[deactivate]
powerprofilesctl set balanced

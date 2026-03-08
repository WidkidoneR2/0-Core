# faelight-dashboard v2.1.0

**TUI System Overview** - Real-time dashboard for Faelight Forest system health and status.

🌲 **Faelight Forest** - Philosophy-driven Linux desktop environment

---

## 🎯 Purpose

Provides a live, interactive terminal dashboard showing:
- System health checks
- Git repository status
- Active system profile
- Planned intents
- Tool statistics
- Security status (VPN, firewall)
- Snapshot information

## 🚀 Usage
```bash
# Launch interactive dashboard
faelight-dashboard

# Show version
faelight-dashboard --version

# Show help
faelight-dashboard --help

# Health check
faelight-dashboard --health
```

## ⌨️ Interactive Controls

Inside the dashboard:

- **h** - Run `dot-doctor` health check
- **g** - Open `lazygit` 
- **i** - List intents
- **r** - Refresh dashboard
- **q** / **Esc** - Quit

## 📊 Dashboard Panels

### 🏥 Health Panel
Shows output from `dot-doctor` health checks:
- Stow packages status
- Core tool availability
- System configuration

### 🔧 System Panel
Displays:
- Git repository status (clean/changes)
- Active system profile
- Count of planned intents

### 🔒 Security Panel
Monitors:
- VPN connection status (Mullvad)
- Firewall status (UFW)
- Latest btrfs snapshot

### 📊 Stats Panel
Counts:
- Number of Rust tools
- Number of installed packages

## 🏗️ Architecture

**Technology:**
- Rust with `ratatui` for TUI rendering
- `crossterm` for terminal control
- Shell command integration for data gathering

**Data Sources:**
- `dot-doctor` - Health checks
- `git status` - Repository status
- `faelight status` - Profile information
- `intent` - Planned work
- `mullvad status` - VPN status
- `systemctl` - Firewall status
- `snapper` - Snapshot information

**Refresh Rate:** 500ms polling interval

## 🔧 Configuration

No configuration required. Dashboard automatically gathers data from:
- `~/0-core/VERSION` - System version
- `~/0-core/INTENT/future/*.md` - Planned intents
- `~/0-core/rust-tools/` - Tool count
- `~/0-core/packages/pkglist.txt` - Package list

## 📜 Philosophy

**Live System Awareness** - The dashboard embodies the 0-Core principle of understanding your system state at all times. Instead of running multiple commands, get a unified view of system health, work status, and security posture.

**Manual Control** - Interactive shortcuts let you drill into specific areas (health, git, intents) without leaving the dashboard context.

**Intentional Monitoring** - Shows not just current state, but planned work (intents), connecting present reality with future direction.

## 🐛 Troubleshooting

**No VPN status shown:**
- Ensure Mullvad VPN is installed
- Run `mullvad status` manually to verify

**No firewall status:**
- UFW must be installed and managed via systemd
- Check: `systemctl status ufw`

**No snapshot information:**
- Requires snapper configuration for root subvolume
- Snapshots may need sudo access to view

**Health checks missing:**
- Verify `dot-doctor` is in PATH
- Run `dot-doctor` manually first

## 🎓 Learning

The dashboard demonstrates:
- **TUI Programming** - Ratatui layout system with panels
- **Event Handling** - Keyboard shortcuts with crossterm
- **Process Integration** - Running shell commands from Rust
- **Data Aggregation** - Gathering information from multiple sources
- **Real-time Updates** - Polling and refreshing display

## 🔄 Version History

- **v1.0.0** (2026-01-21) - Production release
  - Added `--help`, `--version`, `--health` flags
  - Fixed firewall check (systemctl instead of sudo)
  - Graceful snapshot access without sudo
  - Comprehensive README documentation
  - Production-ready stability

- **v0.1.0** (2026-01-11) - Initial TUI implementation
  - Four-panel dashboard layout
  - Interactive shortcuts
  - Real-time data gathering

---

**Part of the Faelight Forest ecosystem** 🌲🦀

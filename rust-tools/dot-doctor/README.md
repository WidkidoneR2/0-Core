# dot-doctor

🏥 **Faelight Forest Health Engine** - Comprehensive system health monitoring for 0-Core

## What It Does

Runs 14 health checks to ensure your system is operating correctly:

1. **Stow Symlinks** - Verifies all config packages are properly stowed
2. **System Services** - Checks faelight-bar and faelight-notify are running
3. **Broken Symlinks** - Scans for broken symlinks in config directories
4. **Yazi Plugins** - Verifies file manager plugins are installed
5. **Binary Dependencies** - Ensures all required CLI tools are available
6. **Git Repository** - Checks for uncommitted changes and unpushed commits
7. **Theme Packages** - Verifies theme packages exist
8. **Scripts** - Ensures core scripts are present and executable
9. **Package Metadata** - Validates .dotmeta documentation
10. **Intent Ledger** - Validates Intent system integrity
11. **Profile System** - Checks profile configuration
12. **Faelight Config** - Validates TOML configuration files
13. **Sway Keybinds** - Detects conflicting keybindings
14. **Security Hardening** - Verifies UFW, fail2ban, Mullvad VPN, SSH hardening

## Usage
```bash
# Run all checks
doctor

# Show detailed explanations
doctor --explain

# Run specific check
doctor --check security

# Show dependency graph
doctor --graph

# Output as JSON
doctor --json
```

## Version

Current: **0.6.0** - Added security hardening checks (v8.1.0)

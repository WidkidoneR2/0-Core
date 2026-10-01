# 🔒 Project 0 - Security Configuration

These files are **system-level** security configurations (not stowed).

## 📁 Files

### **99-hardening.conf**
- **Location:** `/etc/sysctl.d/99-hardening.conf`
- **Purpose:** Kernel hardening settings (Lynis recommendations)
- **Apply:** `sudo sysctl --system`

### **jail.local (or jail.local.example)**
- **Location:** `/etc/fail2ban/jail.local`
- **Purpose:** Fail2ban jail configurations
- **Apply:** `sudo systemctl restart fail2ban`

## 🔧 Installation
```bash
# Install sysctl hardening
sudo cp 99-hardening.conf /etc/sysctl.d/
sudo sysctl --system

# Install fail2ban config (if you have jail.local)
sudo cp jail.local /etc/fail2ban/
sudo systemctl restart fail2ban
sudo fail2ban-client status
```

## 📊 Security Score

**Current Hardening Index:** 71/100 (+3 from v2.7.1)

**Improvements:**
- ✅ Kernel hardening (9 critical settings)
- ✅ Fail2ban jails enabled
- ✅ arch-audit vulnerability scanning
- ✅ Network security hardened

## 🔍 Weekly Audit
```bash
# Run weekly security check
security-check

# Or individual commands
vuln-check        # Show high-risk vulnerabilities
security-score    # Show current hardening index
jail-status       # Check Fail2ban status
```

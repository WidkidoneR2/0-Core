# v3.2.0 Smart Update System - December 16, 2025

## 🔄 Smart Update System

### **Features Added**

#### **safe-update Script**
Smart system update with automatic recovery:
- **Auto-detects yay library issues** (common after pacman updates)
- **Auto-rebuilds yay** on library mismatch
- **Pre/post Btrfs snapshots** (automatic rollback points)
- **Detects .pacnew files** (prompts for review)
- **Post-update health check** (runs dot-doctor)
- **Manual-only execution** (NO systemd timers!)

#### **Fish Functions**
- `weekly-check` - Maintenance with confirmation prompt
- `update-check` - Check updates without installing
- `safe-update` - Direct script access

### **Philosophy: Manual Control**

**Lesson from v3.1 password debugging (12 hours):**

Boot-time systemd timers caused mysterious sudo failures by:
1. Running at boot without credentials
2. Triggering faillock after 3 failed attempts
3. Locking user account until manual reset

**Solution:**
- **NO systemd timers**
- **NO boot automation**  
- **ALL updates require explicit user trigger**
- **Confirmation prompts for safety**

**Result:** Predictable, debuggable, safe. You control when things run.

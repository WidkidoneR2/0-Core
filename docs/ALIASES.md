# Alias Reference - Faelight Forest v6.0.0

**Total Aliases:** 188+  
**Philosophy:** Intentional, organized, documented

---

## 📂 Navigation & Directory Management

### Numbered Structure (0-Core Philosophy)
```bash
core         # cd ~/0-core
src          # cd ~/1-src
work         # cd ~/2-work
keep         # cd ~/3-keep
tmp          # cd ~/9-temp
```

### Quick Navigation
```bash
..           # cd ..
...          # cd ../..
....         # cd ../../..
.....        # cd ../../../..
cdp          # cd - (previous directory)
```

### Directory Stack (Fish-like)
```bash
cd ~1        # Jump to directory 1 in stack
cd ~2        # Jump to directory 2 in stack
dirs -v      # View directory stack
```

### Common Directories
```bash
desk         # ~/Desktop
docs         # ~/Documents
down         # ~/Downloads
pics         # ~/Pictures
vids         # ~/Videos
```

### Config Directories
```bash
conf         # ~/.config
swayconf     # ~/.config/sway
nvimconf     # ~/.config/nvim
zshconf      # ~/.config/zsh
```

---

## 📁 File Management (Modern Tools)

### Eza (Modern ls)
```bash
ls           # eza with icons
ll           # Long format with git status
la           # Show hidden files
l            # Long format
lt           # Sort by modification time
lsize        # Sort by size
tree         # Tree view
```

### Bat (Better cat)
```bash
b            # bat --paging=never (colorized view)
catp         # bat --paging=always (paged view)
catt         # bat --style=plain (plain style)
ccat         # /usr/bin/cat (explicit plain cat)
```

### Fd (Better find)
```bash
search       # fd
findf        # fd --type f (files only)
findd        # fd --type d (directories only)
```

### Fzf (Fuzzy Finder)
```bash
fcd          # Fuzzy cd to directory
vf           # Fuzzy open file in nvim
preview      # Fuzzy find with bat preview
```

### Yazi (File Manager)
```bash
y            # yazi
yy           # yazi
fm           # yazi
ya           # yazi with cd-on-quit
```

---

## 🔒 Core Protection (0-Core Immutability)
```bash
lock-core    # Make 0-core immutable
unlock-core  # Unlock for editing
edit-core    # Edit package with safety
core-status  # Check lock status
```

---

## 🔄 Smart Update System
```bash
safe-update      # Safe system update with snapshots
weekly           # Weekly maintenance check
check-updates    # Check for available updates
update-check     # Same as check-updates
```

---

## 📦 Package Management

### Pacman
```bash
pacu         # sudo pacman -Syu (update)
paci         # sudo pacman -S (install)
pacs         # pacman -Ss (search)
pacr         # sudo pacman -R (remove)
pacrem       # sudo pacman -Rns (remove with deps)
pacinfo      # pacman -Qi (package info)
paclist      # pacman -Qqe (list installed)
```

### Yay (AUR)
```bash
yayu         # yay -Syu (update)
yays         # yay -Ss (search)
yayi         # yay -S (install)
yayr         # yay -R (remove)
ins          # yay -S (install shortcut)
uns          # yay -Rns (uninstall)
yup          # yay -Syu (update)
```

### Maintenance
```bash
cleanup      # Remove orphaned packages
unlock       # Remove pacman lock
orphans      # List orphaned packages
mirror       # Update mirror list
clean-all    # Deep clean (cache + orphans)
fix-keys     # Fix pacman keys
```

---

## 🔧 Git & Version Control

### LazyGit
```bash
lg           # lazygit (best!)
```

### Basic
```bash
g            # git
gst          # git status
gss          # git status -s
```

### Add & Commit
```bash
ga           # git add
gaa          # git add -A
gcm          # git commit -m
gca          # git commit --amend
gcam         # git commit -am
```

### Push & Pull
```bash
gp           # git push
gl           # git pull
gf           # git fetch
```

### Logs
```bash
glog         # git log --oneline -10
gla          # git log --oneline --graph --all
```

### Branches
```bash
gb           # git branch
gba          # git branch -a
gbd          # git branch -d
gbD          # git branch -D
gco          # git checkout
gcb          # git checkout -b
```

### Diff
```bash
gd           # git diff
gds          # git diff --staged
gdp          # git diff (paged with color)
gsh          # git show
```

### Stash
```bash
gstash       # git stash
gstp         # git stash pop
gstl         # git stash list
```

### Undo/Reset
```bash
gundo        # git reset HEAD~1
gunstage     # git reset HEAD
greset       # git reset --hard
gclean       # git clean -fd
```

### 0-Core Management
```bash
dotsave      # Quick commit & push
dotpush      # Commit with date & push
dotstatus    # Check 0-core status
```

---

## 🔍 Core-Diff Aliases

### Quick Checks
```bash
cdiff        # core-diff (short form)
cds          # core-diff summary
cdh          # core-diff --high-risk
cdv          # core-diff --verbose
```

### Visual Inspection
```bash
cdm          # core-diff --open meld
cdd          # core-diff --open delta
```

### Historical Comparisons
```bash
cdlast       # Since last commit
cdrel        # Since last release
```

### Package-Specific
```bash
cdsway       # core-diff wm-sway
cdbar        # core-diff faelight-bar
cdzsh        # core-diff shell-zsh
cdnvim       # core-diff editor-nvim
```

### Combined Workflows
```bash
cdcheck      # cdiff && dot-doctor (morning check)
cdreview     # cdv && cdh (pre-commit review)
```

---

## 💻 System Monitoring & Health

### System Info
```bash
ff           # fastfetch
neofetch     # fastfetch
sysinfo      # fastfetch
```

### Health Checks
```bash
doctor       # dot-doctor
health       # dot-doctor
check-health # dot-doctor
system-health # Full audit
```

### Disk & Memory
```bash
df           # Disk free (human-readable)
du           # Disk usage
duh          # Disk usage sorted
free         # Memory usage
```

### Processes
```bash
psa          # ps auxf
psg          # ps aux | grep
mem          # Top 10 by memory
cpu          # Top 10 by CPU
```

### Network
```bash
myip         # Public IP
localip      # Local IP
pingg        # Ping Google
ports        # Open ports
listening    # Listening services
weather      # wttr.in weather
```

### Snapshots
```bash
snapshots    # List snapshots
snapshot     # Create snapshot
```

---

## 📝 Editor Shortcuts

### Neovim
```bash
v            # nvim
vi           # nvim
vim          # nvim
nv           # nvim
svi          # sudo nvim
```

### Quick Config Editing
```bash
nzsh         # Edit zshrc
nsway        # Edit sway config
nfoot        # Edit foot config
nstarship    # Edit starship config
```

### LazyVim
```bash
lazyvim-update  # Update plugins
lazyvim-clean   # Clean plugins
```

---

## 🖥️ Sway & Desktop

### Sway
```bash
sway-reload  # Reload Sway config
sway-info    # List windows
sway-tree    # Show window tree
```

### faelight-bar
```bash
bar-restart  # Restart faelight-bar
bar-status   # Check bar status
```

### Power
```bash
ssn          # Shutdown
sr           # Reboot
logout       # Exit Sway
suspend      # Suspend
hibernate    # Hibernate
```

---

## 🛠️ Utilities & Quick Actions

### Shell
```bash
c            # clear
h            # history
reload       # Source zshrc
path         # Show PATH
```

### Date & Time
```bash
now          # Current time
nowdate      # Current date
timestamp    # Timestamp format
```

### Sudo
```bash
please       # sudo !!
fucking      # sudo !!
```

### File Operations
```bash
chx          # chmod +x
extract      # tar -xzvf
targz        # tar -czf
untar        # tar -xvf
```

### Clipboard
```bash
yp           # Copy pwd to clipboard
yf           # Copy filename to clipboard
```

---

## 🔐 Security & Auditing

### Lynis Audits
```bash
audit-full   # Full system audit
audit-quick  # Quick audit
security-score # Show hardening score
security-check # Complete security check
```

### Secret Scanning
```bash
scan-secrets  # Gitleaks detect
scan-staged   # Gitleaks on staged
```

### Fail2ban
```bash
jail-status  # Fail2ban status
ban-list     # SSH ban list
```

---

## 📚 Faelight Forest Documentation

### Quick Reference
```bash
keys         # View keybindings
guide        # Complete guide
faelight     # Complete guide
```

### Planning
```bash
roadmap      # View roadmap
ideas        # Edit roadmap
planning     # Planning directory
```

---

## 💼 Productivity Apps
```bash
notes        # Notesnook
notesnook    # Notesnook
kp           # KeePassXC
keepass      # KeePassXC
pass         # KeePassXC
```

---

## 🌐 Web & Browsers

### AI Assistants
```bash
chatgpt      # ChatGPT
claude       # Claude.ai
```

### Common Sites
```bash
youtube      # YouTube
gmail        # Gmail
```

### Browser
```bash
brave        # Brave browser
```

---

## 🔐 Direnv Security
```bash
envrc-check    # View .envrc
envrc-inspect  # Inspect before allowing
envrc-allow    # Allow .envrc
envrc-deny     # Deny .envrc
envrc-status   # Direnv status
```

---

## 💡 Tips

**Finding Aliases:**
```bash
alias | grep keyword   # Search for specific alias
alias | less           # Browse all aliases
```

**Custom Aliases:**
- Add to `~/.config/zsh/.zshrc`
- Group by category
- Document purpose

**Philosophy:**
- Quality over quantity
- Muscle memory > memorization
- Document everything

---

_Last Updated: January 9, 2026 (v6.0.0)_  
_Part of Faelight Forest 0-Core - Sway Edition_

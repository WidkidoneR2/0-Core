# Alias Reference — Faelight Forest v10.3.0
**Total Aliases:** 356  
**Philosophy:** Intentional, organized, documented  
**Updated:** 2026-02-27

---

## Quick access to most-used commands for maximum speed
```bash
c                    # Clean terminal
d                    # System health check
f                    # Main CLI
g                    # Git shortcut
h                    # Command history
l                    # List files
t                    # Teaching tool
v                    # Neovim
y                    # File manager
b                    # Bat viewer
```

## Custom Rust tools for intentional system management
```bash
bar                  # faelight-bar
bootstrap            # faelight-bootstrap
bump                 # bump-system-version
dash                 # faelight-dashboard
dashboard            # faelight-dashboard
dmenu                # faelight-palette
dot                  # dotctl
fm                   # faelight-fm
launcher             # faelight-palette
link                 # faelight-link
lock                 # faelight-lock
menu                 # faelight-menu
notify               # faelight-notify
term                 # faelight-term
zone                 # faelight-zone
doctor               # dot-doctor
check-health         # dot-doctor
health               # dot-doctor
drift                # entropy-check
entropy              # entropy-check
audit                # echo "🏥 Running full audit..." && dot-doctor && entropy-check && security-score
fu                   # faelight-update
sec                  # security-audit
sb                   # faelight-sandbox
sb-diff              # faelight-sandbox diff
sb-status            # faelight-sandbox status
sb-clear             # faelight-sandbox clear
sb-snap              # faelight-sandbox snapshot
sb-snaps             # faelight-sandbox snapshots
sb-restore           # faelight-sandbox restore
sec-scan             # security-audit scan
sec-report           # security-audit report
sec-history          # security-audit history
sec                  # security-audit
sb                   # faelight-sandbox
sb-diff              # faelight-sandbox diff
sb-status            # faelight-sandbox status
sb-clear             # faelight-sandbox clear
sb-snap              # faelight-sandbox snapshot
sb-snaps             # faelight-sandbox snapshots
sb-restore           # faelight-sandbox restore
sec-scan             # security-audit scan
sec-report           # security-audit report
sec-history          # System update
topgrade             # Topgrade replaced by faelight-update
fudr                 # Check updates without applying
fui                  # Interactive update
fuup                 # Quick update
update               # safe-update
safe-update          # ~/0-core/scripts/safe-update
safe-up              # snap-now && safe-update && dot-doctor
fg                   # faelight-git
fga                  # faelight-git add
fgc                  # faelight-git commit
fgp                  # faelight-git push
fgs                  # faelight-git status
hooks                # faelight-hooks
lock-core            # ~/0-core/scripts/core-protect lock
unlock-core          # ~/0-core/scripts/core-protect unlock
edit-core            # ~/0-core/scripts/core-protect edit
core-status          # ~/0-core/scripts/core-protect status
int                  # intent
inta                 # intent add
intc                 # intent complete
intl                 # intent list
ints                 # intent show
guard                # intent-guard
fl                   # faelight-link
stow-check           # faelight-link status
stow-fix             # faelight-link clean
snap                 # faelight-snapshot
snapshot             # faelight-snapshot
snapcreate           # faelight-snapshot create
snaplist             # faelight-snapshot list
snap-now             # faelight-snapshot create "Manual snapshot at $(date +%Y%m%d_%H%M%S)"
snap-before          # echo "📸 Creating safety snapshot..." && snap-now && echo "✅ Snapshot created!"
ff                   # faelight-fetch
getver               # get-version
ver                  # get-version
recent               # recent-files
f-bar                # faelight-bar
f-daemon             # faelight-daemon
daemon-status        # systemctl --user status faelight-daemon
daemon-log           # journalctl --user -u faelight-daemon -n 20 --no-pager
f-bootstrap          # faelight-bootstrap
f-dmenu              # faelight-palette
f-fm                 # faelight-fm
f-guard              # intent-guard
f-hooks              # faelight-hooks
f-launch             # faelight-palette
f-link               # faelight-link
f-lock               # faelight-lock
f-menu               # faelight-menu
f-notify             # faelight-notify
f-recent             # recent-files
f-term               # faelight-term
f-ver                # get-version
f-zone               # faelight-zone
```

## Fast access to common directories in the numbered system
```bash
core                 # v2 orchestrator binary
cls                  # sync all dotfiles — one command
clp                  # preview before syncing
ce                   # core events list
cew                  # live event stream
cpl                  # plugin registry
cpa                  # register plugin
cps                  # plugin status         # live event stream          # today's events
ces                  # ces 1h / ces 30m / ces 2d
cef                  # cef git / cef doctor
csd                  # core simulate doctor
cdt                  # health trend analysis
cdf                  # health forecast      # predict health — no writes
csu                  # preview updates — no writes
cw                   # today's activity summary
cwh                  # health trajectory
cwd                  # cwd git / cwd doctor / cwd security
ctr                  # last 10 events with detail
ctrd                 # ctrd git / ctrd doctor
0core                # navigate to 0-core root
top                  # bottom — better htop
repo                 # git repo summary
bench                # benchmarking
extract              # smart archive extraction
compress             # smart archive creation
diff                 # difftastic — semantic diff
loc                  # lines of code stats
loch                 # 0-core LOC
cdcore               # cd ~/0-core
src                  # cd ~/1-src
work                 # cd ~/2-work
ws                   # workspace-view
keep                 # cd ~/3-keep
conf                 # cd ~/.config
docs                 # cd ~/Documents
down                 # cd ~/Downloads
pics                 # cd ~/Pictures
vids                 # cd ~/Videos
desk                 # cd ~/Desktop
tmp                  # cd ~/9-temp
secrets              # cd ~/secrets
..                   # cd ..
...                  # cd ../..
....                 # cd ../../..
.....                # cd ../../../..
cdp                  # cd -
nvimconf             # cd ~/.config/nvim
swayconf             # cd ~/.config/sway
zshconf              # cd ~/.config/zsh
```

## Version control and development workflows
```bash
g                    # git
ga                   # git add
gaa                  # git add -A
gc                   # git commit -m
gca                  # git commit --amend
gcam                 # git commit -am
gp                   # git push
gl                   # git pull
gst                  # git status
gss                  # git status -s
gd                   # git diff
gds                  # git diff --staged
gdp                  # git diff --color=always | less -R
glog                 # git log --oneline -10
gla                  # git log --oneline --graph --all
gb                   # git branch
gba                  # git branch -a
gbd                  # git branch -d
gbD                  # git branch -D
gco                  # git checkout
gcb                  # git checkout -b
gf                   # git fetch
gsh                  # git show
gstash               # git stash
gstl                 # git stash list
gstp                 # git stash pop
gclean               # git clean -fd
greset               # git reset --hard
gundo                # git reset HEAD~1
gunstage             # git reset HEAD
gcl                  # git clone
lg                   # lazygit
qc                   # git commit -m "Quick update: $(date +%Y-%m-%d)"
qcp                  # git commit -m "Quick update: $(date +%Y-%m-%d)" && git push
dotgit               # cd ~/0-core && git
dotsave              # cd ~/0-core && git add -A && git commit -m "Update configs" && git push
dotpush              # cd ~/0-core && git add -A && git commit -m "Update configs $(date +%Y-%m-%d)" && git push
dotstatus            # cd ~/0-core && git status
dotadd               # dotctl add
dotlist              # dotctl list
dotrem               # dotctl remove
cdiff                # core-diff
cds                  # core-diff summary
cdv                  # core-diff --verbose
cdd                  # core-diff --open delta
cdm                  # core-diff --open meld
cdh                  # core-diff --high-risk
cdlast               # core-diff since HEAD~1
cdrel                # core-diff since $(git describe --tags --abbrev=0 2>/dev/null || echo HEAD)
cdcheck              # cdiff && dot-doctor
cdreview             # cdv && cdh
cdbar                # core-diff faelight-bar
cdsway               # core-diff wm-sway
cdzsh                # core-diff shell-zsh
cdnvim               # core-diff editor-nvim
scan-secrets         # gitleaks detect --no-git -v
scan-staged          # gitleaks protect --staged -v
pre-commit           # echo "🔍 Pre-commit checks..." && gitleaks protect --staged -v && dot-doctor && echo "✅ Safe to commit!"
secrets-mount        # gocryptfs ~/secrets.encrypted ~/secrets && echo "🔓 Secrets mounted"
secrets-unmount      # fusermount -u ~/secrets && echo "🔒 Secrets locked"
arch                 # archaeology-0-core
arch0                # archaeology-0-core
archint              # archaeology-0-core --by-intent
archsince            # archaeology-0-core --since
archtime             # archaeology-0-core --timeline
archwk               # archaeology-0-core --this-week
```

## System package installation and maintenance
```bash
yay                  # Compatibility alias
yayi                 # paru -S
yayr                 # paru -R
yays                 # paru -Ss
yayu                 # paru -Syu
yup                  # paru -Syu
paci                 # Install package
pacr                 # Remove package
pacu                 # Update system
pacs                 # Search packages
pacinfo              # Package info
paclist              # List installed
ins                  # Install package
uns                  # Uninstall + remove deps
orphan-clean         # paru -Rns $(paru -Qtdq) 2>/dev/null || true
cleanup              # faelight-cleanup
f-cleanup            # faelight-cleanup
clean-all            # paru -Sc && paru -Yc
orphans              # pacman -Qtdq
unlock               # sudo rm /var/lib/pacman/db.lck
mirror               # sudo reflector --verbose --latest 10 --protocol https --sort rate --save /etc/pacman.d/mirrorlist
fix-keys             # sudo pacman-key --init && sudo pacman-key --populate && sudo pacman-key --refresh-keys
```

## System management, monitoring, and control
```bash
sysinfo              # fastfetch
neofetch             # fastfetch
sysver               # uname -r
card                 # echo "╔════════════════════════════════════════╗" && echo "║  🌲 FAELIGHT FOREST v9.3.0            ║" && echo "║  🏥 Health: $(dot-doctor | grep "Health:" | awk "{print \$2}")                        ║" && echo "║  📦 Tools: 40 Production Ready         ║" && echo "║  🔒 Security: Hardened                 ║" && echo "╚════════════════════════════════════════╝"
sr                   # reboot
ssn                  # shutdown now
suspend              # systemctl suspend
hibernate            # systemctl hibernate
logout               # swaymsg exit
psa                  # ps auxf
psg                  # ps aux | grep -v grep | grep -i -e VSZ -e
cpu                  # ps auxf | sort -nr -k 3 | head -10
mem                  # ps auxf | sort -nr -k 4 | head -10
ports                # sudo ss -tulanp
listening            # sudo lsof -i -P -n | grep LISTEN
myip                 # curl -s ifconfig.me
localip              # ip -4 addr | grep -oP "(?<=inet\s)\d+(\.\d+){3}" | grep -v 127.0.0.1
pingg                # ping -c 5 google.com
security-check       # sudo pacman -Syu && echo "---" && arch-audit && echo "---" && audit-quick
security-score       # test -f ~/.lynis-score && echo "🛡️  Hardening Index: $(cat ~/.lynis-score)/100" || echo "Run audit-full or audit-quick first"
audit-full           # sudo lynis audit system | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score
audit-quick          # sudo lynis audit system --quick | tee /tmp/lynis-output.txt && grep "Hardening index" /tmp/lynis-output.txt | awk "{print \$4}" > ~/.lynis-score
full-audit           # dot-doctor && entropy-check && security-check
system-health        # dot-doctor && lynis audit system --quick
jail-status          # sudo fail2ban-client status
ban-list             # sudo fail2ban-client status sshd
sway-reload          # swaymsg reload
sway-info            # swaymsg -t get_tree
bar-restart          # ~/0-core/scripts/launch-bar
df                   # df -h
du                   # du -h
duh                  # du -sh * | sort -hr
free                 # free -h
snapshots            # sudo snapper -c root list
snapper-create       # sudo snapper -c root create --description
```

## Listing, viewing, and display utilities
```bash
ls                   # eza --icons --group-directories-first
la                   # eza -a --icons --group-directories-first
ll                   # eza -lah --icons --group-directories-first --git
lt                   # eza -lah --icons --sort=modified --reverse
lsize                # eza -lah --icons --sort=size --reverse
tree                 # eza --tree --icons --group-directories-first
ccat                 # Original cat
cat                  # Replaced with bat
catp                 # Paged bat
catt                 # Plain bat
search               # fd
findf                # fd --type f
findd                # fd --type d
fcd                  # cd $(fd --type d | fzf)
vf                   # nvim $(fd --type f | fzf)
preview              # fzf --preview "bat --color=always {}"
keys                 # bat ~/0-core/docs/KEYBINDINGS.md
keybinds             # keyscan
conflicts            # keyscan
```

## Editor shortcuts and configurations
```bash
nv                   # nvim
vi                   # nvim
vim                  # nvim
svi                  # sudo nvim
lazy                 # nvim
astro                # NVIM_APPNAME=astronvim nvim
chad                 # NVIM_APPNAME=nvchad nvim
lazyvim-update       # nvim --headless "+Lazy! sync" +qa
lazyvim-clean        # nvim --headless "+Lazy! clean" +qa
nzsh                 # nvim ~/.config/zsh/.zshrc
nsway                # nvim ~/.config/sway/config
nbar                 # nvim ~/0-core/rust-tools/faelight-bar/src/main.rs
```

## Miscellaneous useful commands
```bash
now                  # date +"%T"
nowdate              # date +"%Y-%m-%d"
timestamp            # date +"%Y%m%d_%H%M%S"
extract              # tar -xzvf
targz                # tar -czf
untar                # tar -xvf
chx                  # chmod +x
yp                   # Yank path
yf                   # Yank filename
gmail                # xdg-open "https://gmail.com"
youtube              # xdg-open "https://youtube.com"
chatgpt              # xdg-open "https://chat.openai.com"
claude               # xdg-open "https://claude.ai"
weather              # curl wttr.in
prof                 # profile
prof-list            # profile list
prof-switch          # profile switch
wsa                  # workspace-view --active
wss                  # workspace-view --summary
guide                # bat ~/0-core/COMPLETE_GUIDE.md
changelog            # bat ~/0-core/CHANGELOG.md
roadmap              # nvim ~/0-core/docs/planning/ROADMAP.md
planning             # cd ~/0-core/docs/planning && ls
please               # sudo !!
fucking              # sudo !!
reload               # source ~/.config/zsh/.zshrc
s                    # source ~/.zshrc
path                 # echo $PATH | tr ":" "\n"
status               # dot-doctor && echo "" && git status
overview             # fastfetch && echo "" && dot-doctor && echo "" && git -C ~/0-core status -s
check-updates        # update-check
weekly               # weekly-check
lastup               # latest-update
latest               # latest-update
forest-ver           # echo "🌲 Faelight Forest v9.3.0"
release-prep         # echo "📦 Preparing release..." && bump-system-version && compile-changelog.sh && git status
compile-log          # ~/0-core/scripts/compile-changelog.sh
mklog                # ~/0-core/scripts/compile-changelog.sh
envrc-allow          # direnv allow
envrc-deny           # direnv deny
envrc-status         # direnv status
envrc-check          # bat .envrc
envrc-inspect        # bat .envrc && echo "" && echo "⚠️  INSPECT CAREFULLY BEFORE ALLOWING!" && echo "Run: direnv allow"
```

## Auth health monitoring (added 2026-02-11)
```bash
auth-health          # ~/0-core/scripts/check-auth-health
reset-auth           # ~/0-core/scripts/reset-auth
```

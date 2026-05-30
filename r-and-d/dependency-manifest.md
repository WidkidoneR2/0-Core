# Faelight Forest -- NixOS Dependency Manifest
INT-328 · branch: nixos · 2026-05-30
Source: `pacman -Qqe` (121 explicit) → sorted by NixOS target.
Rule: record the **nixpkgs** attribute name, not the Arch name.

## Drops entirely -- Arch-only, no Nix meaning
paru · arch-audit · pacman-contrib · reflector

## Boot / hardware / system → options (lean on nixos-hardware Framework 16 module)
- base, base-devel        → (no equiv; the Nix config *is* the base)
- linux                   → boot.kernelPackages
- linux-firmware          → hardware.enableRedistributableFirmware
- amd-ucode               → hardware.cpu.amd.updateMicrocode
- grub, grub-btrfs, efibootmgr → boot.loader.grub.*
- btrfs-progs             → (implicit when a btrfs fs is declared)
- sudo                    → security.sudo
- vulkan-radeon           → hardware.graphics (radv via mesa)
- vulkan-tools            → systemPackages (diagnostics)

## Services → declarative (install + hand-config collapse into one block each)
- pipewire + pipewire-pulse + wireplumber → services.pipewire { alsa; pulse; wireplumber }
- bluez + bluez-utils     → hardware.bluetooth.enable
- networkmanager          → networking.networkmanager.enable
- iwd                     → networking.wireless.iwd (or NM backend)
- dnsmasq                 → services.dnsmasq (or NM dns)
- fail2ban                → services.fail2ban.enable
- ufw                     → DROP → networking.firewall (native nftables)
- greetd + greetd-tuigreet → services.greetd (tuigreet)
- power-profiles-daemon   → services.power-profiles-daemon.enable
- fwupd                   → services.fwupd.enable
- snapper                 → services.snapper
- polkit-gnome            → security.polkit + polkit_gnome agent
- libratbag               → services.ratbagd.enable        # MX Anywhere 3S
- solaar                  → hardware.logitech / package
- libvirt                 → virtualisation.libvirtd.enable  # ← R&D VM host
- virt-manager            → programs.virt-manager.enable
- qemu-base, qemu-ui-gtk  → (pulled in by libvirtd)
- gocryptfs               → systemPackages

## Display / Wayland / portals
- niri                    → programs.niri (carries forward; → pinnacle later)
- xorg-xwayland           → (pulled by niri / programs.xwayland)
- xdg-desktop-portal-wlr  → xdg.portal.extraPortals
- qt5-wayland, qt6-wayland → systemPackages / qt

## Fonts → fonts.packages   (nerd fonts are now namespaced `nerd-fonts.*`)
- noto-fonts              → noto-fonts
- noto-fonts-emoji        → noto-fonts-color-emoji   [verify attr]
- otf-font-awesome        → font-awesome
- ttf-ibm-plex            → ibm-plex
- ttf-jetbrains-mono      → jetbrains-mono
- ttf-jetbrains-mono-nerd → nerd-fonts.jetbrains-mono   # PRIMARY (12px)
- ttf-hack-nerd           → nerd-fonts.hack
- ttf-meslo-nerd          → nerd-fonts.meslo-lg
- ttf-liberation          → liberation_ttf

## Dev toolchain → flake devShell (`nix develop`, paired with your direnv)
rust(rustc+cargo) · clang · cmake · npm→nodejs · python-pip→python3+pip · base-devel(stdenv) · cargo-audit · cargo-flamegraph

## systemPackages / home.packages -- user tools + apps
CLI: alacritty · atuin* · bandwhich · bat* · bottom(btm) · difftastic · direnv* · dust→du-dust · eza* · fd · git* · git-delta→delta · git-filter-repo · gitleaks · gum · helix · hyperfine · imagemagick · inotify-tools · jq · nushell · onefetch · ouch · pastel · ripgrep · skim · socat · starship* · tailspin · tealdeer · tokei · usbutils · gnu-netcat→netcat-gnu · libguestfs · libheif
Wayland: brightnessctl · cliphist · grim · slurp · wev · wl-clipboard · pamixer · pavucontrol · playerctl
Apps: neovim · keepassxc · discord · libreoffice-fresh
Shell fallback: zsh + zsh-completions → programs.zsh.enable
( * = has a home-manager program module -- prefer it for config integration )

## AUR (vetted, all in nixpkgs except paru)
- brave-bin            → brave
- filen-desktop-bin    → filen-desktop   (sync OK; network-drive mount needs FUSE)
- lla                  → lla
- localsend-bin        → localsend
- mullvad-vpn-bin      → mullvad-vpn + services.mullvad-vpn.enable   (becomes a service)
- notesnook-bin        → notesnook
- onlyoffice-bin       → onlyoffice-bin
- tutanota-desktop-bin → tutanota-desktop   (flagged unmaintained in nixpkgs ~2025; may lag)
- paru                 → DROP

## Forest tools -- NOT pacman; built from source as derivations
- 49 rust-tools/* + engine → pkgs/faelight/, built via flake from committed Cargo.lock
- fsh → derivation + /etc/shells registration + users.users.christian.shell

## Verify-at-build items
noto emoji attr · vulkan-radeon under hardware.graphics · exact nixos-hardware Framework 16 module

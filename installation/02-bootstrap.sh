#!/bin/bash
set -e
echo "🌲 Faelight Forest v6.0 Bootstrap"

mkdir -p ~/0-core ~/1-src ~/2-projects ~/3-archive ~/4-media ~/secrets
chmod 700 ~/secrets

if ! command -v yay &>/dev/null; then
  cd /tmp && git clone https://aur.archlinux.org/yay.git && cd yay
  makepkg -si --noconfirm && cd ~ && rm -rf /tmp/yay
fi

sudo pacman -S --needed --noconfirm sway swaybg swayidle swaylock xdg-desktop-portal-wlr zsh zsh-completions starship foot eza bat fd fzf ripgrep zoxide yazi git-delta neovim rust cmake stow mako fuzzel cliphist wl-clipboard grim slurp brightnessctl playerctl pamixer pipewire pipewire-pulse wireplumber pavucontrol ttf-jetbrains-mono-nerd otf-font-awesome noto-fonts noto-fonts-emoji polkit-gnome ufw fail2ban snapper

yay -S --needed --noconfirm mullvad-vpn-bin brave-bin topgrade

[[ ! -d ~/0-core/.git ]] && rm -rf ~/0-core && git clone https://github.com/WidkidoneR2/0-Core.git ~/0-core

command -v cargo &>/dev/null || (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source "$HOME/.cargo/env")

cd ~/0-core/rust-tools
for dir in */; do [[ -f "${dir}Cargo.toml" ]] && (cd "$dir" && cargo build --release); done

cd ~/0-core
for pkg in shell-zsh prompt-starship editor-nvim fm-yazi vcs-git notif-mako; do [[ -d "$pkg" ]] && stow "$pkg" 2>/dev/null; done

[[ "$SHELL" != "/usr/bin/zsh" ]] && chsh -s /usr/bin/zsh

sudo systemctl enable --now ufw fail2ban
sudo ufw default deny incoming && sudo ufw default allow outgoing && sudo ufw --force enable

echo "✅ Done! Restore secrets, then: exec sway"

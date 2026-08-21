{ config, pkgs, ... }:
{
  imports = [
    ./fsh.nix
    ./alacritty.nix
    ./git.nix
    ./faelight-bar.nix
    ./faelight-wsd.nix
    ./faelight-notify.nix
    ./faelight-insightd.nix
    ./faelight-idle.nix
    ./faelight-wallpaper.nix
    ./faelight-clipboard.nix
  ];

  home.stateVersion = "25.11";
  home.username = "christian";
  home.homeDirectory = "/home/christian";

  xdg.enable = true;

  # Faelight config files (the doctor checks these three)
  xdg.configFile."faelight/config.toml".source = ../dotfiles/faelight/.config/faelight/config.toml;
  xdg.configFile."faelight/profiles.toml".source = ../dotfiles/faelight/.config/faelight/profiles.toml;
  xdg.configFile."faelight/themes.toml".source = ../dotfiles/faelight/.config/faelight/themes.toml;
  xdg.configFile."faelight/term.toml".source = ../dotfiles/faelight/.config/faelight/term.toml;
  xdg.configFile."mango/config.conf".source = ../dotfiles/mango/.config/mango/config.conf;
  xdg.configFile."quickshell/shell.qml".source = ../dotfiles/quickshell/.config/quickshell/shell.qml;

  home.packages = with pkgs; [
    hello
    brave
    lla
    # P2 / Track C: QML desktop-shell framework. Evaluating it as the DRAWER
    # for bar/notifications/launcher/OSD, with the existing faelight-* daemons
    # kept as the WATCHERS -- faelight-wsd already writes workspace state as
    # JSON to ~/.cache/faelight/workspaces, which quickshell can read directly.
    # Pulls Qt6; remove this line if the evaluation says no.
    quickshell
     # CLI tools
    bat
    eza
    fd
    ripgrep
    fzf
    zoxide
    dust
    bottom
    bandwhich
    difftastic
    hyperfine
    tokei
    ouch
    jq
    gum
    tealdeer
    onefetch
    inotify-tools
    usbutils
    atuin
    direnv
    # Nix inspection tools
    nix-tree
    nvd
    nh
    deadnix    # dead-code linter for .nix files
    statix     # Nix anti-pattern / style linter
    # Wayland essentials
    wl-clipboard
    wpaperd
    # Git tools
    lazygit
    # File tools
    mmv-go
    # Lock screen
    hyprlock
    nix-direnv
    delta
    gitleaks
    gocryptfs
    # keybind helpers
    grim
    slurp
    cliphist
    brightnessctl
    playerctl
    pamixer
    # editor + build
    clang
    cmake
    # GUI apps
    filen-desktop
    onlyoffice-desktopeditors
    tutanota-desktop
    notesnook
    # fonts
    nerd-fonts.jetbrains-mono
    nerd-fonts.hack
    noto-fonts-color-emoji
    noto-fonts
    # misc
    pastel
    python3
    sqlite
    ];

  fonts.fontconfig.enable = true;

  programs.direnv.enable = true;
  programs.direnv.nix-direnv.enable = true;
  xdg.configFile."hyprlock/hyprlock.conf" = {
    source = ../dotfiles/hyprlock/.config/hyprlock/hyprlock.conf;
    force = true;
  };
  home.sessionVariables = { EDITOR = "nvim"; VISUAL = "nvim"; };
}
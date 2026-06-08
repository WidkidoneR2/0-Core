{ config, pkgs, ... }:
{
  home.stateVersion = "25.11";

  xdg.enable = true;

  # Faelight config files (the doctor checks these three)
  xdg.configFile."faelight/config.toml".source = ../../config/faelight/.config/faelight/config.toml;
  xdg.configFile."faelight/profiles.toml".source = ../../config/faelight/.config/faelight/profiles.toml;
  xdg.configFile."faelight/themes.toml".source = ../../config/faelight/.config/faelight/themes.toml;
  xdg.configFile."faelight/term.toml".source = ../../config/faelight/.config/faelight/term.toml;

  # Yazi config + plugins
  xdg.configFile."yazi".source = ../../config/yazi/.config/yazi;
xdg.configFile."niri".source = ../../config/niri/.config/niri;
home.packages = with pkgs; [
    brave
    lla
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
    # Wayland essentials
    wl-clipboard
    wpaperd
    # Git tools
    lazygit
    # File tools
    mmv-go
    # Editor
    helix
    # Lock screen
    hyprlock
    nix-direnv
    delta
    gitleaks
    lazygit
    gocryptfs
    # keybind helpers
    grim
    slurp
    wl-clipboard
    cliphist
    brightnessctl
    playerctl
    pamixer
    # editor + build
    helix
    clang
    cmake
    # GUI apps
    filen-desktop
    onlyoffice-desktopeditors
    tutanota-desktop
    notesnook
    yazi
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
  xdg.configFile."faelight-shell/config.fsh".source = ../../config/faelight-shell/.config/faelight-shell/config.fsh;
   xdg.configFile."alacritty".source = ../../config/alacritty/.config/alacritty;
  xdg.configFile."helix" = {
    source = ../../config/helix/.config/helix;
    recursive = true;
  };
  xdg.configFile."hyprlock/hyprlock.conf" = {
    source = ../../config/hyprlock/.config/hyprlock/hyprlock.conf;
    force = true;
  };}

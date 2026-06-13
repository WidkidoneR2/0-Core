{ config, pkgs, ... }:
{
  imports = [
    ./fsh.nix
    ./alacritty.nix
    ./git.nix
  ];

  home.stateVersion = "25.11";
  home.username = "christian";
  home.homeDirectory = "/home/christian";

  xdg.enable = true;

  # Faelight config files (the doctor checks these three)
  xdg.configFile."faelight/config.toml".source = ../../config/faelight/.config/faelight/config.toml;
  xdg.configFile."faelight/profiles.toml".source = ../../config/faelight/.config/faelight/profiles.toml;
  xdg.configFile."faelight/themes.toml".source = ../../config/faelight/.config/faelight/themes.toml;
  xdg.configFile."faelight/term.toml".source = ../../config/faelight/.config/faelight/term.toml;

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
    broot
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
  xdg.configFile."helix" = {
    source = ../../config/helix/.config/helix;
    recursive = true;
  };
  xdg.configFile."hyprlock/hyprlock.conf" = {
    source = ../../config/hyprlock/.config/hyprlock/hyprlock.conf;
    force = true;
  };
  home.sessionVariables = { EDITOR = "hx"; VISUAL = "hx"; };
}
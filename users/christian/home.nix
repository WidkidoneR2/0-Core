{ config, pkgs, ... }:
{
  home.stateVersion = "25.11";

  xdg.enable = true;

  # Faelight config files (the doctor checks these three)
  xdg.configFile."faelight/config.toml".source = ../../03-interfaces/stow/config-faelight/.config/faelight/config.toml;
  xdg.configFile."faelight/profiles.toml".source = ../../03-interfaces/stow/config-faelight/.config/faelight/profiles.toml;
  xdg.configFile."faelight/themes.toml".source = ../../03-interfaces/stow/config-faelight/.config/faelight/themes.toml;
  xdg.configFile."faelight/term.toml".source = ../../03-interfaces/stow/config-faelight/.config/faelight/term.toml;

  # Yazi config + plugins
  xdg.configFile."yazi".source = ../../03-interfaces/stow/fm-yazi/.config/yazi;
xdg.configFile."niri".source = ../../03-interfaces/stow/niri/.config/niri;
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
  xdg.configFile."faelight-shell/config.fsh".source = ../../03-interfaces/stow/shell-faelight/.config/faelight-shell/config.fsh;
   xdg.configFile."alacritty".source = ../../03-interfaces/stow/alacritty/.config/alacritty;}

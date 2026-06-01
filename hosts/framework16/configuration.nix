{ config, pkgs, lib, self, system, ... }:
{
  imports = [ ./hardware-configuration.nix ];

  # --- Boot (UEFI + systemd-boot). LUKS unlock & filesystems come from disko. ---
  boot.loader.systemd-boot.enable = true;
  boot.loader.systemd-boot.configurationLimit = 20;
  boot.loader.efi.canTouchEfiVariables = true;

  nix.settings.experimental-features = [ "nix-command" "flakes" ];
  nixpkgs.config.allowUnfreePredicate = pkg:
    builtins.elem (lib.getName pkg) [
      "filen-desktop"
      "onlyoffice-desktopeditors"
      "discord"
    ];

  # --- Networking (real laptop: WiFi via NetworkManager) ---
  networking.hostName = "framework16";
  networking.networkmanager.enable = true;

  # --- Locale / time (Chicago + US English; change freely) ---
  time.timeZone = "America/Chicago";
  i18n.defaultLocale = "en_US.UTF-8";

  # --- Compressed RAM swap (no hibernation; ideal for a laptop) ---
  zramSwap.enable = true;

  # --- GPU / Wayland (so niri can run) ---
  hardware.graphics.enable = true;
  # Bluetooth
  hardware.bluetooth.enable = true;
  hardware.bluetooth.powerOnBoot = true;

  # XDG portals (file pickers / screenshots for Wayland apps)
  xdg.portal = {
    enable = true;
    extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
    config.common.default = [ "gtk" ];
  };

  # --- Audio (PipeWire) ---
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
  };

  # --- User ---
  users.users.christian = {
    isNormalUser = true;
    extraGroups = [ "wheel" "networkmanager" "video" "input" ];
    # Password set at install (passwd) -- never in this public repo.
  };

  services.openssh = {
    enable = true;
    settings.PasswordAuthentication = true;
    settings.PermitRootLogin = "no";
  };

  # --- The forest + tools ---
  environment.systemPackages = [
    pkgs.git
    pkgs.vim
    self.packages.${system}.faelight-forest
    pkgs.niri
    pkgs.alacritty
    pkgs.yazi
    pkgs.neovim
    pkgs.starship
    pkgs.bat
    pkgs.eza
    pkgs.fd
    pkgs.ripgrep
    pkgs.zoxide
    pkgs.brightnessctl
    pkgs.wireplumber
    pkgs.stow
    pkgs.cargo
    pkgs.rustc
  ];

  home-manager.useGlobalPkgs = true;
  home-manager.useUserPackages = true;
  home-manager.users.christian = import ../../users/christian/home.nix;

  system.stateVersion = "25.11";
}

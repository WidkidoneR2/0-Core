{ config, pkgs, lib, self, system, inputs, ... }:
{
  imports = [ ./hardware-configuration.nix ];

  # --- Boot (UEFI + systemd-boot). LUKS unlock & filesystems come from disko. ---
  boot.loader.systemd-boot.enable = true;
  boot.plymouth.enable = true;
  boot.plymouth.theme = "bgrt";
  boot.loader.systemd-boot.configurationLimit = 5;
  boot.loader.efi.canTouchEfiVariables = true;

  nix.settings.experimental-features = [ "nix-command" "flakes" ];
  nixpkgs.config.allowUnfreePredicate = pkg:
    builtins.elem (lib.getName pkg) [
      "filen-desktop"
      "onlyoffice-desktopeditors"
      "discord"
      "tutanota-desktop"
      "notesnook"
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
  # PAM service for faelight-lock authentication
  security.pam.services.faelight-lock = {};

  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
  };

  # --- User ---
  users.users.christian = {
    isNormalUser = true;
    extraGroups = [ "wheel" "networkmanager" "video" "input" "libvirtd" ];
    # Password set at install (passwd) -- never in this public repo.
  };

  # Firewall (NixOS native nftables -- replaces UFW)
  networking.firewall.enable = true;
  networking.firewall.allowedTCPPorts = [ 22 ];
  networking.firewall.allowedUDPPorts = [ ];

  # Brightness default on boot (use brightnessctl, programs.light removed in 26.05)
  systemd.tmpfiles.rules = [
    "f /sys/class/backlight/amdgpu_bl2/brightness 0644 root root - 31097"
  ];

  # Security services
  services.fail2ban.enable = true;

  # Hardware services
  services.fwupd.enable = true;
  services.power-profiles-daemon.enable = true;

  # Virtualisation (INT-328 R&D VM host)
  virtualisation.libvirtd.enable = true;
  virtualisation.libvirtd.onBoot = "start";
  programs.virt-manager.enable = true;

  services.mullvad-vpn.enable = true;

  # greetd -- display manager, launches faelight-login (forest greeter)
  services.greetd = {
    enable = true;
    settings.default_session = {
      command = "${pkgs.tuigreet}/bin/tuigreet --time --remember --remember-user-session --cmd niri-session --greeting \"🌲 Faelight Forest\" --theme \"border=green;text=green;prompt=green;time=green;action=blue;button=green;container=black;input=green\"";
      user = "greeter";
    };
  };

  # hyprlock PAM authentication
  security.pam.services.hyprlock = {};

  services.openssh = {
    enable = true;
    settings.PasswordAuthentication = true;
    settings.PermitRootLogin = "no";
  };

  # --- The forest + tools ---
  environment.systemPackages = [
    inputs.pinnacle.packages.${system}.pinnacle
    inputs.mango.packages.${system}.mango
    pkgs.protobuf
    pkgs.git
    pkgs.vim
    self.packages.${system}.faelight-forest
    pkgs.niri
    pkgs.alacritty
    pkgs.yazi
    pkgs.neovim
    pkgs.bat
    pkgs.eza
    pkgs.fd
    pkgs.ripgrep
    pkgs.zoxide
    pkgs.brightnessctl
    pkgs.wireplumber
    pkgs.cargo
    pkgs.rustc
  ];

  home-manager.useGlobalPkgs = true;
  home-manager.useUserPackages = true;
  home-manager.users.christian = import ../../users/christian/home.nix;

  system.stateVersion = "25.11";
}

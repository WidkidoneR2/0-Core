{ config, pkgs, self, system, ... }:
{
  imports = [ ./hardware-configuration.nix ];

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  networking.hostName = "faelight-vm";

  users.users.christian = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    initialPassword = "faelight";
  };

  services.openssh = {
    enable = true;
    settings.PasswordAuthentication = true;
    settings.PermitRootLogin = "no";
  };

  environment.systemPackages = [
    pkgs.git
    pkgs.vim
    self.packages.${system}.faelight-forest
    pkgs.niri
    pkgs.alacritty
    pkgs.yazi
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

  system.stateVersion = "26.05";

  # VM display -- virtio GPU for Wayland support
  virtualisation.vmVariant.virtualisation.qemu.options = [
    "-vga none"
    "-device virtio-gpu-pci"
    "-display gtk,gl=on"
  ];

  # Enable DRI for graphics
  hardware.graphics.enable = true;

  # Set fsh as default shell
  users.defaultUserShell = pkgs.bash;

  # Initialize forest runtime on first login
  systemd.tmpfiles.rules = [
    "d /home/christian/0-core/runtime 0755 christian users -"
    "f /home/christian/0-core/runtime/state.db 0644 christian users -"
  ];
}

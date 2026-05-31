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

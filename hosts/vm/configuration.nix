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
  };

  environment.systemPackages = [
    pkgs.git
    pkgs.vim
    self.packages.${system}.faelight-forest
  ];

  system.stateVersion = "25.11";
}

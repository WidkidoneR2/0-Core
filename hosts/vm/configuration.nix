{ config, pkgs, ... }:
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

  environment.systemPackages = with pkgs; [ git vim ];

  system.stateVersion = "25.11";
}

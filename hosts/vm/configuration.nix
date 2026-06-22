{ config, pkgs, self, system, inputs, ... }:
{
  imports = [ ./hardware-configuration.nix ];

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  networking.hostName = "faelight-vm";

  # INT-077: serial console on ttyS0 so the VM can run in-terminal (QEMU -nographic).
  # Additive -- graphical boot still works; console mode is selected at launch via QEMU_OPTS.
  boot.kernelParams = [ "console=ttyS0" ];

  # INT-077: make `build-vm` generate a headless, serial-on-stdio VM by default so the
  # guest runs IN the terminal (copy-paste) -- no graphical window, no QEMU flag-wrangling.
  # The graphical path is still available by overriding QEMU_OPTS at launch if needed.
  virtualisation.vmVariant.virtualisation = {
    graphics = false;
    # INT-077: forward guest SSH (22) -> host 2222 for the reliable console path.
    forwardPorts = [
      { from = "host"; host.port = 2222; guest.port = 22; }
    ];
  };

  # INT-077: ensure a login prompt on the serial console, and autologin in console mode
  # so the in-terminal loop is one step (no password). Graphical path unaffected.
  systemd.services."serial-getty@ttyS0".enable = true;
  services.getty.autologinUser = "christian";

  users.users.christian = {
    isNormalUser = true;
    extraGroups = [ "wheel" "seat" "video" "input" ];
    initialPassword = "faelight";
  };


  services.openssh = {
    enable = true;
    settings.PasswordAuthentication = true;
    settings.PermitRootLogin = "no";
  };

  # Seat management for Wayland compositors
  services.seatd.enable = true;

  # Graphics
  hardware.graphics.enable = true;

  environment.systemPackages = [
    inputs.pinnacle.packages.${system}.pinnacle
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

  users.defaultUserShell = pkgs.bash;

  systemd.tmpfiles.rules = [
    "d /home/christian/0-core/runtime 0755 christian users -"
    "f /home/christian/0-core/runtime/state.db 0644 christian users -"
  ];
}

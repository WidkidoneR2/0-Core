{ config, pkgs, lib, self, system, inputs, ... }:
{
  # INT-024/056: shared VM base. Everything common to BOTH login modes
  # (mirror = tuigreet, regreet = cage+ReGreet). The login layer is a
  # separate module the flake selects. greetd.enable + the session command
  # live in the login modules, not here.
  imports = [ ./hardware-configuration.nix ../../modules/desktop/mango.nix ];

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  # Build performance + storage: parallel builds use all threads;
  # auto-optimise-store hardlinks identical store paths to reclaim disk.
  nix.settings.auto-optimise-store = true;
  nix.settings.max-jobs = "auto";
  nix.settings.cores = 0;

  # INT-043 Phase 4: Cachix binary cache (pull side) in the GUEST, mirroring
  # hosts/framework16. Additive -- keeps cache.nixos.org default.
  nix.settings.extra-substituters = [ "https://faelight-forest.cachix.org" ];
  nix.settings.extra-trusted-public-keys = [
    "faelight-forest.cachix.org-1:IFKABeIAWapKtYNrjD/f3hIFBAUrsQcxA/m1pheT2yM="
  ];

  networking.hostName = "faelight-vm";

  # INT-077: serial console on ttyS0 so the VM can run in-terminal.
  boot.kernelParams = [ "console=tty0" "console=ttyS0" ];

  virtualisation.vmVariant.virtualisation = {
    graphics = true;
    forwardPorts = [
      { from = "host"; host.port = 2222; guest.port = 22; }
    ];
  };

  systemd.services."serial-getty@ttyS0".enable = true;
  services.getty.autologinUser = "christian";

  users.users.christian = {
    isNormalUser = true;
    extraGroups = [ "wheel" "seat" "video" "input" ];
    initialPassword = "faelight";
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILllaW7TLBGy19mQ6zKrCbDtOU4uuZqWVCBG0XXxcL9m christian@faelight-forest"
    ];
  };

  services.openssh = {
    enable = true;
    settings.PasswordAuthentication = true;
    settings.KbdInteractiveAuthentication = false;
    settings.PermitRootLogin = "no";
  };

  # INT-077: TEST-BED ONLY -- passwordless sudo in the disposable guest.
  # Scoped to hosts/vm/, must NEVER apply to framework16.
  security.sudo.wheelNeedsPassword = false;

  # Seat management for Wayland compositors (both modes need this).
  services.seatd.enable = true;

  hardware.graphics.enable = true;

  # INT-077 gate 5: SPICE guest agent -- shared clipboard + display resize.
  services.spice-vdagentd.enable = true;
  services.qemuGuest.enable = true;

  faelight.desktop.mango.enable = true;

  # INT-056: force software GL system-wide in the VM -- virgl breaks
  # client-surface rendering. llvmpipe software path renders reliably.
  environment.variables.LIBGL_ALWAYS_SOFTWARE = "1";

  environment.systemPackages = [
    inputs.pinnacle.packages.${system}.pinnacle
    pkgs.git
    pkgs.vim
    self.packages.${system}.faelight-forest
    self.packages.${system}.faelight-bar-gtk
    self.packages.${system}.faelight-logout
    self.packages.${system}.faelight-launcher
    pkgs.alacritty
    pkgs.yazi
    pkgs.bat
    pkgs.eza
    pkgs.fd
    pkgs.ripgrep
    pkgs.zoxide
    pkgs.brightnessctl
    pkgs.wireplumber
    pkgs.spice-vdagent
    pkgs.cargo
    pkgs.rustc
  ];

  home-manager.useGlobalPkgs = true;
  home-manager.useUserPackages = true;
  home-manager.users.christian = import ../../users/christian/home.nix;

  system.stateVersion = "26.05";

  users.defaultUserShell = pkgs.bash;

  systemd.tmpfiles.rules = [
    # Create the 0-core path chain christian-owned so fsh can create state.db.
    # No empty-file rule: an empty file is not a valid SQLite db; fsh self-heals
    # (create_dir_all + CREATE TABLE IF NOT EXISTS) and makes a real db on first run.
    "d /home/christian/0-core 0755 christian users -"
    "d /home/christian/0-core/runtime 0755 christian users -"
  ];
}

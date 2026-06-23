{ config, pkgs, self, system, inputs, ... }:
{
  imports = [ ./hardware-configuration.nix ];

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  # INT-043 Phase 4: Cachix binary cache (pull side) in the GUEST, mirroring
  # hosts/framework16. Additive -- keeps cache.nixos.org default. Lets a clean VM
  # build PULL the crane faelightDeps derivation instead of recompiling it.
  nix.settings.extra-substituters = [ "https://faelight-forest.cachix.org" ];
  nix.settings.extra-trusted-public-keys = [
    "faelight-forest.cachix.org-1:IFKABeIAWapKtYNrjD/f3hIFBAUrsQcxA/m1pheT2yM="
  ];

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
    # INT-077: authorize the host key so `vm ssh` needs no login password.
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILllaW7TLBGy19mQ6zKrCbDtOU4uuZqWVCBG0XXxcL9m christian@faelight-forest"
    ];
  };


  services.openssh = {
    enable = true;
    settings.PasswordAuthentication = true;
    settings.KbdInteractiveAuthentication = false;  # INT-077: avoid double prompt; key auth preferred
    settings.PermitRootLogin = "no";
  };

  # INT-077: TEST-BED ONLY -- passwordless sudo in the disposable faelight-vm guest so
  # recovery drills (sudo nixos-rebuild ...) run frictionless over `vm ssh`. This is
  # scoped to hosts/vm/ and must NEVER apply to framework16 (real metal keeps sudo password).
  security.sudo.wheelNeedsPassword = false;

  # Seat management for Wayland compositors
  services.seatd.enable = true;

  # Graphics
  hardware.graphics.enable = true;

  # INT-077 gate 5: SPICE guest agent -- shared clipboard + display resize for `vm gui`.
  # Host side uses remote-viewer (virt-viewer) against the QEMU SPICE socket.
  services.spice-vdagentd.enable = true;
  services.qemuGuest.enable = true;

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
    pkgs.spice-vdagent  # INT-077: SPICE clipboard/resize agent
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

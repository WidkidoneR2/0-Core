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
    # INT-087: give the VM real RAM + cores so Miracle (Mir + software GL) has headroom.
    # Default build-vm RAM is ~1024 MB -- far too little for a Mir compositor on llvmpipe;
    # the earlier "needs real metal" failure was likely this ceiling. VM-only (vmVariant),
    # does NOT affect the metal framework16 config. Framework 16 has ample RAM to spare.
    memorySize = 8192;  # 8 GiB
    cores = 4;
    # INT-027 (2026-07-15): FULL BOOT CHAIN. Without these, build-vm boots the guest
    # KERNEL-DIRECT (qemu -kernel/-initrd), skipping firmware AND the bootloader entirely --
    # so the systemd-boot declared above never actually runs, and the VM cannot rehearse
    # bootloader / Secure Boot / early-boot work (INT-049 lifecycle, INT-059 Lanzaboote,
    # INT-078 Everglow). useBootLoader builds a real ESP + installs systemd-boot into the
    # disk and boots FROM it; useEFIBoot boots via OVMF/UEFI firmware instead of SeaBIOS.
    # Result: OVMF -> systemd-boot -> kernel -> initrd -> systemd -> greetd, like metal.
    # Costs a few seconds of boot (firmware + bootloader stages) -- that is the point.
    useBootLoader = true;
    useEFIBoot = true;
    # INT-159 (2026-07-15): .ms CODE is Secure-Boot-capable (built SMM_REQUIRE=TRUE).
    # Plain VARS = NO keys enrolled = SETUP MODE, which is what sbctl needs (INT-059).
    # OVMF_VARS.ms.fd would arrive with Microsoft's keys = user mode = wrong for us.
    # Requires -machine q35,smm=on, injected via QEMU_OPTS (PROVEN reachable: qemu merges
    # a second -machine over the module's accel=kvm:tcg, and KVM survives -- /dev/kvm open).
    efi.firmware = "${pkgs.OVMFFull.fd}/FV/OVMF_CODE.ms.fd";
    efi.variables = "${pkgs.OVMFFull.fd}/FV/OVMF_VARS.fd";
    # INT-159 gate 3: swtpm. The module spawns it, wires the socket, runs tpm2_startup.
    # Measured Boot (INT-059) needs a TPM to measure into; guest reported "TPM2 Support: no".
    # CAUTION (INT-027): a zombie swtpm survived `vm down` holding the launch lock invisibly --
    # vm_pids only greps qemu-system-x86_64. That is gate 4 of this intent.
    tpm.enable = true;
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
    pkgs.miracle-wm  # INT-087: second compositor (VM test)
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
  home-manager.users.christian = import ../../home/christian/home.nix;

  system.stateVersion = "26.05";

  users.defaultUserShell = pkgs.bash;

  # INT-087: VM-only Miracle session so the greetd picker offers mango OR miracle.
  # Kept here (not mango.nix) so Miracle stays confined to the VM testbed until
  # the charter's AFTER-085/086 sequencing is met for real hardware.
  environment.etc."greetd/sessions/miracle.desktop".text = ''
    [Desktop Entry]
    Name=Miracle
    Exec=miracle-wm
    Type=Application
  '';

  # INT-056 Phase 2: SafeShell fallback session -- the anti-lockout safety net.
  # A bare fsh login on the VT, NO compositor/Wayland. If every compositor
  # session fails to launch (bad GPU lib, broken config), the greeter still
  # offers "Shell" -> a working login shell to repair the system from.
  # Tested VM-first per 056's Rule before it graduates to metal.
  environment.etc."greetd/sessions/safeshell.desktop".text = ''
    [Desktop Entry]
    Name=SafeShell
    Exec=fsh
    Type=Application
  '';

  systemd.tmpfiles.rules = [
    # Create the 0-core path chain christian-owned so fsh can create state.db.
    # No empty-file rule: an empty file is not a valid SQLite db; fsh self-heals
    # (create_dir_all + CREATE TABLE IF NOT EXISTS) and makes a real db on first run.
    "d /home/christian/0-core 0755 christian users -"
    "d /home/christian/0-core/runtime 0755 christian users -"
  ];
}

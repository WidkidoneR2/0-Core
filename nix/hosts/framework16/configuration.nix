{ config, pkgs, lib, self, system, inputs, ... }:
{
  imports = [
    ./hardware-configuration.nix
    ../../modules/desktop/pinnacle.nix
    ../../modules/desktop/mango.nix
    ../../modules/desktop/miracle.nix
    ../../modules/desktop/greetd.nix           # INT-061 Phase 2
    ../../profiles/base.nix                    # INT-061 LAYER 2
    inputs.lanzaboote.nixosModules.lanzaboote  # INT-161
  ];

  # --- Boot (UEFI + Lanzaboote/Secure Boot). LUKS unlock & filesystems come from disko. ---
  # INT-161: lanzaboote REPLACES the systemd-boot module -- it installs its own signed copy of
  # systemd-boot plus a signed UKI per generation. mkForce because the line below is `= true` in
  # plain assignment and lanzaboote's assertion refuses to coexist.
  boot.loader.systemd-boot.enable = lib.mkForce false;
  boot.lanzaboote = {
    enable = true;
    pkiBundle = "/var/lib/sbctl";  # sbctl's default since 0.15. Keys live OUTSIDE the repo.
    configurationLimit = 15;       # carried over from systemd-boot's -- see note below.
  };

  boot.plymouth.enable = true;
  boot.plymouth.theme = "bgrt";

  # INT-161: boot.loader.systemd-boot.configurationLimit STOPS APPLYING once the module is forced
  # off -- lanzaboote reads boot.lanzaboote.configurationLimit instead. Without carrying it over we
  # would silently go from 15 entries on the ESP to all 110 generations, and /boot is 4G.
  boot.loader.efi.canTouchEfiVariables = true;

  # INT-061 LAYER 2: experimental-features, auto-optimise-store, max-jobs and cores moved to
  # profiles/base.nix -- they were byte-identical here and in vm/base.nix.

  # INT-043: Attic self-hosted binary cache (pull side). Additive -- keeps
  # cache.nixos.org default (Attic priority 41 vs nixos.org 40, so nixos.org is
  # consulted first, Attic supplements with our crane deps closure). Replaced Cachix,
  # whose multi-tenant content-dedup refused to serve our crane paths (proven 2026-07-07;
  # Attic clean-pull of the full 667-path closure verified). Server: nix/modules/services/atticd.nix.
  nix.settings.extra-substituters = [ "http://127.0.0.1:8080/faelight" ];
  nix.settings.extra-trusted-public-keys = [
    "faelight:oyzBMXRQvmCpv7tXJHstiYm/4C+kDjH8rjfEe1sZecU="
  ];
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
  virtualisation.libvirtd.onBoot = "ignore";
  programs.virt-manager.enable = true;

  services.mullvad-vpn.enable = true;

  # greetd -- display manager with session picker. INT-061 Phase 2: the service, the
  # tuigreet command and the SafeShell entry now live in modules/desktop/greetd.nix so
  # metal and the VM cannot drift. They already had: this host said button=white while
  # hosts/vm/login-mirror.nix -- which called itself an "exact replica" -- said
  # button=lightmagenta. Metal's value won; both now read it from one place.
  faelight.desktop.greetd.enable = true;

  # INT-056 SafeShell rescue session: now provided by modules/desktop/greetd.nix
  # (faelight.desktop.greetd.safeShell, default true). Identical entry, one definition,
  # shared with the VM instead of copied into it.


  services.openssh = {
    enable = true;
    settings.PasswordAuthentication = true;
    settings.PermitRootLogin = "no";
  };

  # Declarative /etc/faelight/VERSION from meta/VERSION (INT-031 de-Arch).
  # faelight-login reads this; faelight-release no longer writes it.
  environment.etc."faelight/VERSION".text = builtins.readFile ../../../faelight/meta/VERSION;

  # --- The forest + tools ---
  environment.systemPackages = [
    inputs.pinnacle.packages.${system}.pinnacle
    pkgs.protobuf
    pkgs.git
    pkgs.vim
    self.packages.${system}.faelight-forest
    self.packages.${system}.faelight-logout
    self.packages.${system}.faelight-launcher  # INT-084
    self.packages.${system}.faelight-bar-gtk
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
    inputs.attic.packages.${system}.attic-client  # INT-043: attic CLI (replaced pkgs.cachix)
    pkgs.virt-viewer  # INT-077: SPICE client (remote-viewer) for `vm gui`
    pkgs.sbctl  # INT-161: Secure Boot key management. Lanzaboote's own docs list this
                # ("For debugging and troubleshooting Secure Boot"). NOT already present from 059 --
                # that was the VM config. A locked-out machine cannot `nix shell nixpkgs#sbctl`
                # without a network, and sbctl verify is the pre-reboot check that catches a lockout
                # before it happens.
  ];

  faelight.desktop.pinnacle.enable = true;
  faelight.desktop.mango.enable = true;
  faelight.desktop.miracle.enable = true; # INT-087: enable Miracle as selectable session (056 SafeShell net confirmed)

  home-manager.useGlobalPkgs = true;
  home-manager.useUserPackages = true;
  home-manager.users.christian = import ../../home/christian/home.nix;

  # EDITOR: NixOS ships EDITOR=nano in /etc/set-environment by default, and that file
  # WINS the login chain over home-manager's hm-session-vars (it sources later). INT-147
  # tried "disable nano + let home-manager own EDITOR" but the nano default re-appeared in
  # set-environment regardless of programs.nano.enable, so a fresh login still got nano.
  # Correct fix: set EDITOR at the SYSTEM level so set-environment itself exports nvim.
  # Verified by full-chain trace: `bash --login -x` -> EDITOR=nvim. (INT-147 follow-up)
  programs.nano.enable = false;
  environment.variables.EDITOR = "nvim";
  environment.variables.VISUAL = "nvim";

  system.stateVersion = "25.11";
}

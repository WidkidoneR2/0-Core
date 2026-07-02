{ config, pkgs, self, system, inputs, ... }:
{
  imports = [ ./hardware-configuration.nix ../../modules/desktop/mango.nix ];

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  # Build performance + storage (2026-06-24): parallel builds use all 16 threads;
  # auto-optimise-store hardlinks identical store paths to reclaim disk.
  nix.settings.auto-optimise-store = true;
  nix.settings.max-jobs = "auto";
  nix.settings.cores = 0;

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
  boot.kernelParams = [ "console=tty0" "console=ttyS0" ];  # INT-056: tty0 first so the graphical framebuffer initializes (was ttyS0-only = headless)

  # INT-077: make `build-vm` generate a headless, serial-on-stdio VM by default so the
  # guest runs IN the terminal (copy-paste) -- no graphical window, no QEMU flag-wrangling.
  # The graphical path is still available by overriding QEMU_OPTS at launch if needed.
  virtualisation.vmVariant.virtualisation = {
    graphics = true;  # INT-056: was false (headless). true = real graphical framebuffer the compositor can drive.
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

  # INT-056: mirror framework16's login flow -- greetd -> tuigreet --cmd mango.
  # Makes the VM a faithful login-test surface for the recovery/login cluster.
  # The serial-console + SSH path (above) stays intact, so `vm ssh` rescues a
  # broken graphical login -- a working preview of the 056 safety pattern itself.
  # INT-056: ReGreet via its NixOS module. The module enables a default cage+regreet session
  # WITH proper config (the raw 'cage -- regreet' command rendered an EMPTY window because
  # regreet had no config). greetd stays enabled; the module manages default_session.
  services.greetd.enable = true;
  time.timeZone = "America/Chicago";  # INT-054: Central, matches host
  programs.regreet.enable = true;
  # INT-054: candy-neon forest theme for ReGreet (GTK4 CSS). Near-black green base,
  # neon-lime accents, electric-aqua highlights. VM-tested before any host change.
  programs.regreet.extraCss = ''
    /* INT-054 candy-neon GLASS theme -- forest greeter, GTK4.
       Near-black green base, translucent glass card, neon-lime glow, aqua focus. */
    window, .background {
      background-color: #0a0f0c;
      background-image: radial-gradient(circle at 50% 35%, #12200f 0%, #0a0f0c 55%, #060a07 100%);
    }
    box, grid { background-color: transparent; }

    /* The login card: translucent glass with a top-edge light catch + outer glow. */
    .horizontal:not(button), window > box {
      background-color: rgba(16, 22, 14, 0.72);
      border: 1px solid rgba(57, 255, 20, 0.35);
      border-top: 1px solid rgba(120, 255, 90, 0.55);
      border-radius: 16px;
      box-shadow: 0 0 28px rgba(57, 255, 20, 0.18),
                  inset 0 1px 0 rgba(180, 255, 150, 0.18);
    }

    label, .greeter {
      color: #d8f5d0;
      font-family: "JetBrainsMono Nerd Font", monospace;
      text-shadow: 0 0 4px rgba(57, 255, 20, 0.25);
    }

    /* Clock: big glowing lime. */
    .clock {
      color: #6dff3c;
      font-family: "JetBrainsMono Nerd Font", monospace;
      font-size: 2.4em;
      font-weight: bold;
      text-shadow: 0 0 12px rgba(57, 255, 20, 0.55),
                   0 0 24px rgba(57, 255, 20, 0.25);
    }

    /* Entry: glassy inset field, lime border, glow on focus. */
    entry {
      background-color: rgba(8, 12, 8, 0.85);
      color: #6dff3c;
      caret-color: #6dff3c;
      border: 1.5px solid rgba(57, 255, 20, 0.5);
      border-radius: 10px;
      padding: 9px 14px;
      box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.6),
                  inset 0 1px 0 rgba(120, 255, 90, 0.12);
    }
    entry:focus {
      border-color: #50dcff;
      box-shadow: 0 0 14px rgba(80, 220, 255, 0.5),
                  inset 0 2px 6px rgba(0, 0, 0, 0.6);
    }

    /* Buttons: aqua glass, fill-on-hover. */
    button {
      background-image: linear-gradient(rgba(28, 40, 26, 0.9), rgba(14, 22, 12, 0.9));
      color: #50dcff;
      border: 1.5px solid rgba(80, 220, 255, 0.65);
      border-radius: 10px;
      padding: 7px 18px;
      box-shadow: inset 0 1px 0 rgba(150, 230, 255, 0.18),
                  0 0 10px rgba(80, 220, 255, 0.12);
      text-shadow: 0 0 4px rgba(80, 220, 255, 0.4);
    }
    button:hover {
      background-image: linear-gradient(#50dcff, #3bb8e0);
      color: #06120a;
      box-shadow: 0 0 16px rgba(80, 220, 255, 0.55);
    }
    button:active { background-image: linear-gradient(#3bb8e0, #2f9ec0); }

    /* User / Session fields are comboboxes wrapping an entry. Give the ENTRY the
       single border; make the combobox frame + its inner box add nothing. One clean
       shape per field, no overlap. */
    combobox,
    combobox > box,
    combobox box.linked,
    combobox button {
      background-color: transparent;
      background-image: none;
      border: none;
      box-shadow: none;
      outline: none;
      color: #d8f5d0;
    }
    combobox entry {
      background-color: rgba(8, 12, 8, 0.85);
      color: #6dff3c;
      border: 1.5px solid rgba(57, 255, 20, 0.5);
      border-radius: 10px;
      box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.6);
    }
    combobox entry:focus {
      border-color: #50dcff;
      box-shadow: 0 0 14px rgba(80, 220, 255, 0.5),
                  inset 0 2px 6px rgba(0, 0, 0, 0.6);
    }
    /* Dropdown arrow inside the field. */
    combobox arrow { color: #ff7b6b; }

    /* Pencil (edit) buttons: small flat amber chips, distinct from the aqua Login. */
    button.image-button {
      background-image: none;
      background-color: rgba(20, 28, 16, 0.6);
      color: #ffb347;
      border: 1px solid rgba(255, 179, 71, 0.5);
      border-radius: 8px;
      box-shadow: none;
      padding: 4px 8px;
      min-width: 0;
    }
    button.image-button:hover {
      background-color: rgba(255, 179, 71, 0.2);
      border-color: rgba(255, 179, 71, 0.8);
    }
    button.image-button image { color: #ffb347; }

    /* Kill ALL frames (greeting box, bottom button-row connector) + separators. */
    frame, frame > border, .frame {
      border: none;
      box-shadow: none;
      background: none;
      background-color: transparent;
    }
    separator {
      background-color: transparent;
      background-image: none;
      min-height: 0;
      min-width: 0;
      border: none;
    }
  '';

  # INT-056: the greetd 'greeter' user must be in input/seat/video to read /dev/input/event*
  # (crw-rw---- root:input) -- without this, ReGreet renders but accepts NO keyboard input.
  users.users.greeter.extraGroups = [ "input" "seat" "video" ];
  faelight.desktop.mango.enable = true;
  # INT-056: force software GL system-wide in the VM -- virgl breaks client-surface rendering
  # (compositor paints but app windows are invisible). llvmpipe software path renders reliably.
  environment.variables.LIBGL_ALWAYS_SOFTWARE = "1";  # INT-056: kept (mango module); pinnacle launched directly via greetd --cmd below (binary is in systemPackages)

  environment.systemPackages = [
    inputs.pinnacle.packages.${system}.pinnacle
    pkgs.git
    pkgs.vim
    self.packages.${system}.faelight-forest
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
  home-manager.users.christian = import ../../home/christian/home.nix;

  system.stateVersion = "26.05";

  users.defaultUserShell = pkgs.bash;

  systemd.tmpfiles.rules = [
    "d /home/christian/0-core/runtime 0755 christian users -"
    "f /home/christian/0-core/runtime/state.db 0644 christian users -"
  ];
}

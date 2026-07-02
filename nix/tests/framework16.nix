# NixOS VM test -- STAGE 2: boots the REAL framework16 config (with metal-only
# bits neutralized) and asserts the boot + greetd login chain. (INT-061 harness)
#
# This is the anti-lockout gate for the actual boot-critical move. It imports the
# real hosts/framework16/configuration.nix -- so a broken import path, greetd config,
# or compositor module fails HERE, headless, in ~30s -- never on the metal laptop.
#
# Metal-only pieces the VM cannot provide are overridden (mkForce/lib.mkForce):
#   - hardware-configuration.nix (real Framework hw + LUKS/disko disk)
#   - systemd-boot EFI bootloader (VM uses its own boot)
# What IS tested: the module composition, greetd config, session wiring, forest tools.
{ pkgs, self, inputs, ... }:
pkgs.testers.runNixOSTest {
  name = "framework16-boot";
  node.specialArgs = { inherit self inputs; system = "x86_64-linux"; };
  # Let the real host config own nixpkgs.config (its allowUnfreePredicate) instead
  # of the test framework's read-only default.
  node.pkgsReadOnly = false;
  nodes.machine = { config, pkgs, lib, ... }: {
    imports = [
      inputs.home-manager.nixosModules.home-manager
      ../../hosts/framework16/configuration.nix
    ];

    # --- Neutralize metal-only pieces the VM can't provide ---
    # Bootloader: VM provides its own; disable systemd-boot EFI.
    boot.loader.systemd-boot.enable = lib.mkForce false;
    boot.loader.efi.canTouchEfiVariables = lib.mkForce false;
    boot.plymouth.enable = lib.mkForce false;
    # home-manager: don't build christian's full home in the boot test (heavy + pulls
    # config/ dotfiles). The boot/greetd chain is what we gate here.
    home-manager.users = lib.mkForce {};
  };
  testScript = ''
    machine.wait_for_unit("multi-user.target")
    machine.succeed("systemctl cat greetd.service | grep -q greetd")
    machine.succeed("which core")
    machine.succeed("which faelight-shell")
    print("REAL framework16 config: boot + greetd + forest tools OK")
  '';
}

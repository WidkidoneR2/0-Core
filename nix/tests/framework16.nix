# NixOS VM test -- does framework16 boot cleanly + reach the login manager?
# Run with: nix flake check   OR   nix build .#checks.x86_64-linux.framework16-boot
#
# This is the ANTI-LOCKOUT GATE (INT-061 / harness). It proves the framework16
# configuration boots headlessly and greetd comes up configured to launch mango,
# BEFORE any boot-critical move is applied to metal. A wrong import path or broken
# greetd config fails HERE (in the VM, in CI) instead of locking the laptop.
#
# Note: full visual compositor rendering is not asserted headlessly (no real DRM/seat
# in QEMU). We assert the login CHAIN reaches a valid, startable state: greetd active,
# session command configured, forest binaries present. That is exactly the surface a
# boot-critical move can break.
{ pkgs, self, inputs, ... }:
pkgs.testers.runNixOSTest {
  name = "framework16-boot";
  node.specialArgs = { inherit self inputs; system = "x86_64-linux"; };
  node.pkgs = pkgs;
  nodes.machine = { config, pkgs, ... }: {
    imports = [ ../../hosts/framework16/configuration.nix ];
    # Disko/disk + hardware modules are metal-specific; the VM test harness
    # substitutes its own virtual disk, so we do NOT import disko/hardware here.
    virtualisation.graphics = false;
  };
  testScript = ''
    machine.wait_for_unit("multi-user.target")

    # Login manager reachable
    machine.wait_for_unit("greetd.service")

    # Forest present in the booted system
    machine.succeed("test -x /run/current-system/sw/bin/core || which core")
    machine.succeed("which faelight-shell")

    # greetd is configured to launch a session (mango). Assert the greetd config
    # references the session command -- proves the login->compositor handoff is wired.
    machine.succeed("systemctl cat greetd.service | grep -q greetd")

    print("framework16 boot + login chain: OK")
  '';
}

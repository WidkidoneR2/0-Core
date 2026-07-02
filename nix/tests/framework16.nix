# NixOS VM test -- does framework16 boot cleanly?
# Run with: nix build .#tests.framework16
{ pkgs, ... }:
pkgs.nixosTest {
  name = "framework16-boot";
  nodes.machine = { config, pkgs, ... }: {
    imports = [ ../hosts/framework16/configuration.nix ];
  };
  testScript = ''
    machine.wait_for_unit("multi-user.target")
    machine.succeed("which core")
    machine.succeed("which faelight-shell")
  '';
}

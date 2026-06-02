# NixOS VM test -- does friday daemon start?
{ pkgs, ... }:
pkgs.nixosTest {
  name = "friday-service";
  nodes.machine = { ... }: {
    imports = [ ../modules/services/friday.nix ];
  };
  testScript = ''
    machine.wait_for_unit("friday.service")
    machine.succeed("systemctl is-active friday")
  '';
}

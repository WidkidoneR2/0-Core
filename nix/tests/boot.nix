# NixOS VM test -- generic boot smoke test
{ pkgs, ... }:
pkgs.nixosTest {
  name = "boot-smoke";
  nodes.machine = _: {};
  testScript = ''
    machine.wait_for_unit("multi-user.target")
    machine.succeed("echo forest alive")
  '';
}

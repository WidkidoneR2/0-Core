{ config, pkgs, lib, ... }:
# INT-087 P1: Miracle-wm second compositor profile (Mir-based, Sway-IPC).
# DORMANT MODULE -- intentionally NOT imported by any host and defaults to disabled.
# Nothing here emits until this file is added to a host's imports AND enable=true.
# Wiring into greetd is login-touching and VM-gated per INT-056 (Forest Recovery
# Protocol); do NOT enable until 056's fallback session + greeter-escape are confirmed.
{
  options.faelight.desktop.miracle.enable =
    lib.mkEnableOption "Miracle-wm Wayland compositor (second profile, Mir-based)";

  config = lib.mkIf config.faelight.desktop.miracle.enable {

    environment.systemPackages = [
      pkgs.miracle-wm
    ];

    # greetd session entry (mirrors mango.nix:13-18).
    # Miracle uses a YAML config (~/.config/miracle-wm.yaml), not mango's -c config.conf.
    # NON-DEFAULT by design: this is a SELECTABLE session at the greeter; mango stays
    # the default login. A Miracle launch failure means picking mango at greetd --
    # never a lockout. (Only true once the module is wired + the 056 net is confirmed.)
    environment.etc."greetd/sessions/miracle.desktop".text = ''
      [Desktop Entry]
      Name=Miracle-wm
      Exec=miracle-wm
      Type=Application
    '';

    # GPU / DRM access (mirrors mango.nix:21-22).
    hardware.graphics.enable = true;
    services.udev.packages = [ pkgs.libinput ];
  };
}

{ config, pkgs, lib, inputs, system, ... }:
{
  options.faelight.desktop.pinnacle.enable =
    lib.mkEnableOption "Pinnacle Wayland compositor";

  config = lib.mkIf config.faelight.desktop.pinnacle.enable {

    environment.systemPackages = [
      inputs.pinnacle.packages.${system}.pinnacle
    ];

    # XDG portal for Pinnacle (wlr-compatible)
    xdg.portal = {
      enable = true;
      wlr.enable = true;
      extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
      config.common.default = [ "wlr" "gtk" ];
    };

    # greetd session entry
    environment.etc."greetd/sessions/pinnacle.desktop".text = ''
      [Desktop Entry]
      Name=Pinnacle
      Exec=pinnacle --session
      Type=Application
    '';

    # GPU / DRM access for Pinnacle
    hardware.graphics.enable = true;
    services.udev.packages = [ pkgs.libinput ];
  };
}

{ config, pkgs, lib, ... }:
{
  options.faelight.desktop.mango.enable =
    lib.mkEnableOption "MangoWM Wayland compositor";

  config = lib.mkIf config.faelight.desktop.mango.enable {

    environment.systemPackages = [
      pkgs.mangowc
    ];

    # greetd session entry
    environment.etc."greetd/sessions/mango.desktop".text = ''
      [Desktop Entry]
      Name=MangoWM
      Exec=mango -c /home/christian/.config/mango/config.conf
      Type=Application
    '';

    # GPU / DRM access
    hardware.graphics.enable = true;
    services.udev.packages = [ pkgs.libinput ];
  };
}

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

    # Autostart faelight-notify with stderr suppressed
    systemd.user.services.faelight-notify = {
      description = "Faelight notification daemon";
      wantedBy = [ "graphical-session.target" ];
      partOf = [ "graphical-session.target" ];
      serviceConfig = {
        ExecStart = "${pkgs.faelight-forest}/bin/faelight-notify";
        Restart = "on-failure";
        RestartSec = "2s";
        StandardError = "journal";
      };
    };

    # GPU / DRM access
    hardware.graphics.enable = true;
    services.udev.packages = [ pkgs.libinput ];
  };
}

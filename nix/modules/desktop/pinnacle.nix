{ config, pkgs, lib, inputs, system, ... }:
{
  options.faelight.desktop.pinnacle.enable =
    lib.mkEnableOption "Pinnacle Wayland compositor";

  config = lib.mkIf config.faelight.desktop.pinnacle.enable {

    environment.systemPackages = [
      inputs.pinnacle.packages.${system}.pinnacle
      # INT-067: lua interpreter for Pinnacle's config (pinnacle.toml runs
      # `lua lua/init.lua`). Without this in the SYSTEM path, the greetd-launched
      # session can't find lua and silently falls back to Pinnacle's default
      # config -- which is why the custom forest keybinds (Super+B/E/P) never
      # loaded. lua5_4 matches the config's Lua version.
      pkgs.lua5_4
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

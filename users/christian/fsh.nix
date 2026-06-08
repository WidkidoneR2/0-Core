{ config, pkgs, ... }:
{
  # fsh config file -- managed by Home Manager
  xdg.configFile."faelight-shell/config.fsh".source =
    ../../config/faelight-shell/.config/faelight-shell/config.fsh;

  # fsh is the default shell -- set via configuration.nix users.users.christian.shell
  # This module owns the config file only; the binary comes from faelight-forest package
}

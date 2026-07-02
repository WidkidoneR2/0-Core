{ config, pkgs, ... }:
{
  # Alacritty terminal -- config managed by Home Manager
  # Binary comes from environment.systemPackages in configuration.nix
  xdg.configFile."alacritty".source =
    ../dotfiles/alacritty/.config/alacritty;

  # Font: JetBrainsMono Nerd Font 12px (set in alacritty.toml)
  # Shell: faelight-shell (set in alacritty.toml terminal.shell)
}

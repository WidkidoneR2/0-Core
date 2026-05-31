{ config, pkgs, ... }:
{
  home.stateVersion = "25.11";

  xdg.enable = true;

  # Faelight config files (the doctor checks these three)
  xdg.configFile."faelight/config.toml".source = ../../03-interfaces/stow/config-faelight/.config/faelight/config.toml;
  xdg.configFile."faelight/profiles.toml".source = ../../03-interfaces/stow/config-faelight/.config/faelight/profiles.toml;
  xdg.configFile."faelight/themes.toml".source = ../../03-interfaces/stow/config-faelight/.config/faelight/themes.toml;
  xdg.configFile."faelight/term.toml".source = ../../03-interfaces/stow/config-faelight/.config/faelight/term.toml;

  # Yazi config + plugins
  xdg.configFile."yazi".source = ../../03-interfaces/stow/fm-yazi/.config/yazi;
}

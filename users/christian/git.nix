{ config, pkgs, ... }:
{
  programs.git = {
    enable = true;
    settings = {
      user.name     = "WidkidoneR2";
      user.email    = "WidkidoneR2@users.noreply.github.com";
      core.editor   = "nvim";
      credential.helper     = "store";
      init.defaultBranch    = "main";
      push.autoSetupRemote  = true;
    };
  };

  programs.delta = {
    enable = true;
    enableGitIntegration = true;
    options = {
      navigate     = true;
      line-numbers = true;
      syntax-theme = "gruvbox-dark";
    };
  };
}

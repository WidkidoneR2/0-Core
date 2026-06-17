{ ... }:
{
  # --- Faelight Notify (INT-065) ---
  # org.freedesktop.Notifications daemon + wlr-layer-shell overlay.
  # Hangs off faelight-session.target (defined in faelight-bar.nix) so it
  # auto-starts with the mango session and survives reboots/rebuilds --
  # retiring the manual `setsid faelight-notify` dance for good. systemd is
  # now the sole starter, so the singleton guard never fires.
  systemd.user.services.faelight-notify = {
    Unit = {
      Description = "Faelight Notify";
      PartOf = [ "faelight-session.target" ];
      After = [ "faelight-session.target" ];
    };
    Service = {
      ExecStart = "/run/current-system/sw/bin/faelight-notify";
      Restart = "always";
      RestartSec = 3;
    };
    Install.WantedBy = [ "faelight-session.target" ];
  };
}

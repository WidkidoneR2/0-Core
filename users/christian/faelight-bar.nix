{ ... }:
{
  # --- Faelight session group ---
  # graphical-session.target refuses manual start, so we own a startable
  # target. Mango's exec-once starts this; per-session services (bar now,
  # notify / bar-v2 later) hang off it via WantedBy.
  systemd.user.targets.faelight-session = {
    Unit.Description = "Faelight graphical session group";
  };

  # --- Faelight Bar (INT-053) ---
  # wlr-layer-shell client. Restart=always is the seatbelt now that the
  # broken-pipe bug (loop never drained the Wayland socket) is fixed.
  systemd.user.services.faelight-bar = {
    Unit = {
      Description = "Faelight Bar";
      PartOf = [ "faelight-session.target" ];
      After = [ "faelight-session.target" ];
    };
    Service = {
      ExecStart = "/run/current-system/sw/bin/faelight-bar";
      Restart = "always";
      RestartSec = 3;
    };
    Install.WantedBy = [ "faelight-session.target" ];
  };
}

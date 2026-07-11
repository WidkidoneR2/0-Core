{ ... }:
{
  # --- Faelight Insight Daemon (INT-114) ---
  # The nervous system of Faelight Forest. Watches runtime/state.db on a 30s
  # poll, processes signals (e.g. failure-loop detection), and surfaces
  # insights. Renamed from faelight-contextd (INT-114) to end the context/
  # contextd one-letter collision; revived here as a proper home-manager
  # service (the old loose dotfile unit was orphaned and never deployed --
  # which is why the "Nervous System" health factor always scored false).
  # faelight-session.target is defined in faelight-bar.nix.
  systemd.user.services.faelight-insightd = {
    Unit = {
      Description = "Faelight Insight Daemon (state.db signals -> insights)";
      PartOf = [ "faelight-session.target" ];
      After = [ "faelight-session.target" ];
    };
    Service = {
      ExecStart = "/run/current-system/sw/bin/faelight-insightd start";
      Restart = "on-failure";
      RestartSec = 10;
    };
    Install.WantedBy = [ "faelight-session.target" ];
  };
}

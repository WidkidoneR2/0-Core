_:
{
  # --- Faelight Idle (P2) ---
  # Wayland idle-notify watcher. Spawns faelight-lock when the seat goes idle,
  # and emits idle.start / idle.lock / idle.end events.
  #
  # Wired 2026-08-19. Before this the crate compiled and never ran: Super+L
  # locked manually, nothing locked automatically, so the machine stayed open
  # when left alone -- with Secure Boot enforced, sshd off and five sandbox
  # policies, the screen was the way in.
  #
  # 600s = 10 minutes. Deliberately not shorter: a lock that fires while you
  # are reading is a lock you will disable.
  systemd.user.services.faelight-idle = {
    Unit = {
      Description = "Faelight Idle";
      PartOf = [ "faelight-session.target" ];
      After = [ "faelight-session.target" ];
    };
    Service = {
      ExecStart = "/run/current-system/sw/bin/faelight-idle --timeout 600";
      Restart = "always";
      RestartSec = 3;
    };
    Install.WantedBy = [ "faelight-session.target" ];
  };
}

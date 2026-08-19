_:
{
  # --- Faelight Wallpaper (P2) ---
  # Health-reactive background daemon. The colour shifts with system health,
  # which is the one thing no off-the-shelf wallpaper tool can do -- wpaperd
  # draws images, this draws a reading of the machine.
  #
  # Wired 2026-08-19. The crate compiled and never ran: nothing drew a
  # background at all, so the desktop showed mango's rootcolor.
  # Use --static-color to disable the health reaction, --color to pin one.
  systemd.user.services.faelight-wallpaper = {
    Unit = {
      Description = "Faelight Wallpaper";
      PartOf = [ "faelight-session.target" ];
      After = [ "faelight-session.target" ];
    };
    Service = {
      ExecStart = "/run/current-system/sw/bin/faelight-wallpaper";
      Restart = "always";
      RestartSec = 3;
    };
    Install.WantedBy = [ "faelight-session.target" ];
  };
}

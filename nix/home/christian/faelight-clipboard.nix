_:
{
  # --- Faelight Clipboard (P2) ---
  # Clipboard history daemon. faelight-clipboard watch records; pick offers
  # the history through fzf, bound to SUPER+V in mango.
  #
  # Wired 2026-08-19. The crate compiled and never ran, and no clipboard
  # keybind existed at all -- copy history simply was not kept. cliphist is
  # installed but was not running either, so there is no conflict.
  systemd.user.services.faelight-clipboard = {
    Unit = {
      Description = "Faelight Clipboard";
      PartOf = [ "faelight-session.target" ];
      After = [ "faelight-session.target" ];
    };
    Service = {
      ExecStart = "/run/current-system/sw/bin/faelight-clipboard watch";
      Restart = "always";
      RestartSec = 3;
    };
    Install.WantedBy = [ "faelight-session.target" ];
  };
}

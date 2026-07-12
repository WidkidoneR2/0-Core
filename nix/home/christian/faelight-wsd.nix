_:
{
  # --- Faelight Workspace Daemon (INT-053) ---
  # dwl-ipc client (zdwl_ipc_manager_v2): tracks mango tag state and writes
  # ~/.cache/faelight/workspaces as JSON for the bar to read. Self-contained
  # Rust binary -- pure-Rust Wayland backend, no system libwayland.
  # faelight-session.target is defined in faelight-bar.nix.
  systemd.user.services.faelight-wsd = {
    Unit = {
      Description = "Faelight Workspace Daemon (dwl-ipc -> JSON)";
      PartOf = [ "faelight-session.target" ];
      After = [ "faelight-session.target" ];
    };
    Service = {
      ExecStart = "/run/current-system/sw/bin/faelight-wsd";
      Restart = "always";
      RestartSec = 3;
    };
    Install.WantedBy = [ "faelight-session.target" ];
  };
}

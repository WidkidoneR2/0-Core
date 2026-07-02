{ config, pkgs, lib, ... }:
{
  # INT-024/056: MIRROR login mode -- greetd -> tuigreet -> mango.
  # Exact replica of hosts/framework16's login (line 108) so the VM is a
  # faithful test surface for the current real system. No cage, no DRM
  # handoff: tuigreet owns the VT and hands the seat straight to mango.
  services.greetd.enable = true;
  time.timeZone = "America/Chicago";  # match host

  services.greetd.settings.default_session.command =
    "${pkgs.tuigreet}/bin/tuigreet --time --remember-session --sessions /etc/greetd/sessions --greeting \"Welcome to Faelight Forest\" --theme 'border=green;title=lightgreen;greet=lightgreen;text=lightcyan;time=lightgreen;prompt=lightcyan;input=lightgreen;action=lightyellow;button=lightmagenta;container=black' --cmd mango";
}

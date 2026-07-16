{ config, pkgs, lib, ... }:
# INT-061 Phase 2: greetd/tuigreet login as its OWN module.
#
# The v2 charter (INT-061, decision #3): "greetd gets its OWN module. Login is
# lockout-class; isolating it makes it testable and keeps boot/login changes surgical."
#
# WHY THIS EXISTS -- measured 2026-07-16, not theorised. The tuigreet command was
# hand-copied between hosts/framework16/configuration.nix:125 and hosts/vm/login-mirror.nix:11.
# login-mirror.nix:4 called itself "Exact replica of hosts/framework16's login (line 108)".
# It was not:
#     framework16      -> ...action=lightyellow;button=white;container=black
#     vm/login-mirror  -> ...action=lightyellow;button=lightmagenta;container=black
# The VM that exists to MIRROR metal's login was testing a different greeter than metal
# runs. Nobody did that on purpose -- it is what two hand-maintained copies of one string
# do over time. (That comment also said "line 108"; the block had moved to 122.)
#
# METAL WINS: button=white is what the real laptop renders, so it is the canonical value.
# Change it here and both metal and the VM move together, by construction.
#
# safeShell is INDEPENDENT of enable on purpose: hosts/vm/base.nix wants the rescue session
# for BOTH login variants, but only login-mirror.nix enables tuigreet -- login-regreet.nix
# runs cage+ReGreet instead and must not have a tuigreet command forced on it.
{
  options.faelight.desktop.greetd = {
    enable = lib.mkEnableOption "greetd + tuigreet login, Faelight Forest themed";

    defaultSession = lib.mkOption {
      type = lib.types.str;
      default = "mango";
      description = "Session tuigreet pre-selects via --cmd. Must match a .desktop Exec.";
    };

    safeShell = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = ''
        Offer the SafeShell rescue session -- INT-056's anti-lockout net. A bare fsh login
        on the VT, no compositor, no Wayland. If every compositor session fails to launch,
        the greeter still offers a working shell to repair the system from. This is what
        makes the 2026-06-09 24h lockout structurally impossible to repeat.
        VM-proven (base.nix Phase 2: BadCompositor failed, SafeShell survived) before it
        graduated to metal.
      '';
    };
  };

  config = lib.mkMerge [
    (lib.mkIf config.faelight.desktop.greetd.enable {
      services.greetd = {
        enable = true;
        settings.default_session = {
          command = "${pkgs.tuigreet}/bin/tuigreet --time --remember-session --sessions /etc/greetd/sessions --greeting \"Welcome to Faelight Forest\" --theme 'border=green;title=lightgreen;greet=lightgreen;text=lightcyan;time=lightgreen;prompt=lightcyan;input=lightgreen;action=lightyellow;button=white;container=black' --cmd ${config.faelight.desktop.greetd.defaultSession}";
          user = "greeter";
        };
      };
    })

    (lib.mkIf config.faelight.desktop.greetd.safeShell {
      environment.etc."greetd/sessions/safeshell.desktop".text = ''
        [Desktop Entry]
        Name=SafeShell
        Exec=fsh
        Type=Application
      '';
    })
  ];
}

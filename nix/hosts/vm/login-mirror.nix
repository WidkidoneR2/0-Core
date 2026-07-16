{ config, pkgs, lib, ... }:
{
  # INT-024/056: MIRROR login mode -- greetd -> tuigreet -> mango. No cage, no DRM
  # handoff: tuigreet owns the VT and hands the seat straight to mango.
  #
  # INT-061 Phase 2 (2026-07-16): THIS FILE USED TO LIE, and it is the reason
  # modules/desktop/greetd.nix exists.
  #
  # It said: "Exact replica of hosts/framework16's login (line 108) so the VM is a
  # faithful test surface for the current real system."
  # Three things wrong in one sentence:
  #   1. NOT a replica -- its tuigreet --theme said button=lightmagenta where metal
  #      said button=white. The VM built to MIRROR metal's login was testing a
  #      different greeter than metal runs.
  #   2. "line 108" -- the block had moved to 122. Line references rot.
  #   3. "replica" was only ever true at the instant it was typed. It was a COPY, and
  #      copies drift. This one did, and nobody chose it.
  #
  # Now the tuigreet command lives in ONE place (modules/desktop/greetd.nix, imported by
  # base.nix) and this host just switches it on. The mirror is a mirror BY CONSTRUCTION,
  # not by vigilance. Change the theme there and metal and VM move together.
  faelight.desktop.greetd.enable = true;

  time.timeZone = "America/Chicago";  # match host
}

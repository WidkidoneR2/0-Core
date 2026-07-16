{ lib, ... }:
# INT-061 LAYER 2 -- the OS registry, expressed in Nix.
#
# The v2 charter: "Declarative-over-imperative: the OS-level registry is expressed in Nix
# (flake/profiles/modules/hosts); no imperative drift." profiles/ was specified in the charter
# and never built -- it was the last missing piece of the whole v2 tree.
#
# base.nix = settings EVERY machine needs, regardless of what it is for.
#
# WHY THESE FOUR AND NOT MORE -- measured 2026-07-16, not designed:
# framework16 and vm/base.nix declared these four identically, by hand, in two files:
#     nix.settings.experimental-features   framework16:31   vm/base.nix:39
#     nix.settings.auto-optimise-store     framework16:35   vm/base.nix:43
#     nix.settings.max-jobs                framework16:36   vm/base.nix:44
#     nix.settings.cores                   framework16:37   vm/base.nix:45
# Byte-identical duplication across hosts is exactly what a profile is for.
#
# WHAT IS DELIBERATELY *NOT* HERE:
#   - extra-substituters / extra-trusted-public-keys. These LOOK like duplication and are
#     not: framework16 pulls from a self-hosted Attic at 127.0.0.1:8080, vm/base.nix still
#     points at Cachix. Collapsing them would be wrong -- 127.0.0.1 inside the VM means the
#     VM, not the host. See vm/base.nix for the finding; it needs its own intent.
#   - time.timeZone. Duplicated THREE times (framework16:62, login-mirror.nix:8,
#     login-regreet.nix:8, the last two both commented "matches host" while not reading the
#     host). It belongs here, but the VM login modules set it and moving it is a separate,
#     testable change.
#   - The charter also names desktop.nix / laptop.nix / development.nix / security.nix.
#     NOT built, deliberately: each would have exactly ONE consumer today (one laptop, one
#     VM, one rescue image). A profile with one consumer is ceremony, not structure. They
#     get built when a second machine needs them -- which is when they start being true.
#
# The charter's Rule: "The structure is the philosophy made visible. If you cannot point at
# the layer in the tree, it is not 0-Core -- it is just files." LAYER 2 is now pointable-at.
{
  # Flakes + the modern CLI. Every machine in this forest is flake-managed.
  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  # Build performance + storage: parallel builds use every available thread;
  # auto-optimise-store hardlinks identical store paths to reclaim disk.
  nix.settings.auto-optimise-store = true;
  nix.settings.max-jobs = "auto";
  nix.settings.cores = 0;
}

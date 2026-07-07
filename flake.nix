{
  description = "Faelight NixOS";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  inputs.home-manager.url = "github:nix-community/home-manager/release-26.05";
  inputs.home-manager.inputs.nixpkgs.follows = "nixpkgs";
  inputs.disko.url = "github:nix-community/disko";
  inputs.disko.inputs.nixpkgs.follows = "nixpkgs";
  inputs.nixos-hardware.url = "github:NixOS/nixos-hardware";
  inputs.pinnacle.url = "github:pinnacle-comp/pinnacle";
  inputs.pinnacle.inputs.nixpkgs.follows = "nixpkgs";
  inputs.crane.url = "github:ipetkov/crane";
  # INT-043: Attic self-hosted binary cache (replaces Cachix -- Cachix's multi-tenant
  # content-dedup refused to serve our crane deps closure; Attic is single-tenant/ours
  # with configurable upstream-skip, so it actually serves the closure).
  inputs.attic.url = "github:zhaofengli/attic";
  inputs.attic.inputs.nixpkgs.follows = "nixpkgs";
  # INT-122: nixCats -- real-Lua-packaged-by-Nix neovim (migrating off nixvim).
  # Bring its own nixpkgs is NOT needed; nixCats is a library builder, follows ours.
  inputs.nixcats.url = "github:BirdeeHub/nixCats-nvim";
  # INT-119: git-hooks.nix -- adds a SANDBOXED, reproducible `nix flake check` hook
  # gate (read-only FS, no network, pinned tools). COMPLEMENTS faelight-hooks (which
  # stays the commit-time authority but shells out to host tools + skips when absent).
  inputs.git-hooks.url = "github:cachix/git-hooks.nix";
  inputs.git-hooks.inputs.nixpkgs.follows = "nixpkgs";

  outputs = { self, nixpkgs, home-manager, disko, nixos-hardware, pinnacle, crane, ... }@inputs:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      craneLib = crane.mkLib pkgs;
      faelightCommonArgs = {
        src = ./.;
        strictDeps = true;
        cargoExtraArgs = "--workspace --locked";
        doCheck = false;
        dontUseCmakeConfigure = true;
        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.rustPlatform.bindgenHook
          pkgs.cmake
          pkgs.makeWrapper
        ];
        buildInputs = [
          pkgs.wayland pkgs.libxkbcommon pkgs.libinput pkgs.libdisplay-info
          pkgs.seatd pkgs.udev pkgs.libdrm pkgs.libgbm pkgs.libGL pkgs.openssl
          pkgs.zlib pkgs.fontconfig pkgs.freetype pkgs.vulkan-loader pkgs.pango
          pkgs.cairo pkgs.pixman pkgs.dbus pkgs.pam pkgs.libsodium
        ];
      };
      faelightDeps = craneLib.buildDepsOnly (faelightCommonArgs // {
        pname = "faelight-forest-deps";
        version = "9.2.0";
        # INT-043: keep the deps hash stable so the Cachix push stays valid across
        # daily work. With src = ./. the deps hash churned on every repo change.
        # We build a manifests-only source (Cargo.toml + Cargo.lock, no .rs), then
        # run normalize-deps-versions.sh which (1) zeroes each [package] version so
        # our cicomplete bumps do not move the hash, and (2) stubs targets so cargo
        # can check the workspace. We do NOT use crane's mkDummySrc: it embeds a
        # store path in the dummy TOML (issue #117) which this Nix rejects. Result:
        # the deps hash changes ONLY when real third-party dependencies change.
        # NB: pass as `dummySrc` (not `src`) so crane uses our prepared source
        # verbatim and SKIPS its own mkDummySrc -- avoiding both the store-path
        # build line (issue #117) and crane's panic-handler dummy which collides
        # with our crate literally named `core`.
        dummySrc =
          let
            manifests = pkgs.lib.fileset.toSource {
              root = ./.;
              fileset = pkgs.lib.fileset.fileFilter
                (file: file.name == "Cargo.toml" || file.name == "Cargo.lock")
                ./.;
            };
          in
          pkgs.runCommand "faelight-deps-src" { nativeBuildInputs = [ pkgs.gawk pkgs.gnugrep pkgs.findutils ]; } ''
            cp -r ${manifests} $out
            chmod -R u+w $out
            ${pkgs.bash}/bin/bash ${./faelight/packages/faelight/scripts/normalize-deps-versions.sh} $out
          '';
      });
    in {
      # INT-024: faelight-vm has TWO login modes sharing one base.
      #   faelight-vm          = base + tuigreet  (mirrors hosts/framework16)
      #   faelight-vm-regreet  = base + ReGreet   (migration testbed)
      nixosConfigurations.faelight-vm = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit self system inputs; };
        modules = [
          home-manager.nixosModules.home-manager
          ./nix/hosts/vm/base.nix
          ./nix/hosts/vm/login-mirror.nix
        ];
      };

      nixosConfigurations.faelight-vm-regreet = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit self system inputs; };
        modules = [
          home-manager.nixosModules.home-manager
          ./nix/hosts/vm/base.nix
          ./nix/hosts/vm/login-regreet.nix
        ];
      };

      nixosConfigurations.framework16 = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit self system inputs; };
        modules = [
          disko.nixosModules.disko
          ./nix/hosts/framework16/disko.nix
          nixos-hardware.nixosModules.framework-16-7040-amd
          home-manager.nixosModules.home-manager
          ./nix/hosts/framework16/configuration.nix
          ./nix/modules/services/atticd.nix
          { system.configurationRevision = self.rev or self.dirtyRev or "dirty"; }
        ];
      };

      packages.${system} = {
        # INT-043: crane deps-only derivation -- the cacheable unit pushed to Cachix
        faelight-deps = faelightDeps;
        faelight-forest = craneLib.buildPackage (faelightCommonArgs // {
          pname = "faelight-forest";
          version = "9.2.0";
          cargoArtifacts = faelightDeps;
          postFixup = ''
            for f in "$out"/bin/*; do
              wrapProgram "$f" \
                --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath [
                  pkgs.wayland
                  pkgs.libxkbcommon
                  pkgs.libGL
                  pkgs.vulkan-loader
                  pkgs.libgbm
                  pkgs.libdrm
                  pkgs.libinput
                  pkgs.seatd
                  pkgs.udev
                  pkgs.fontconfig
                  pkgs.freetype
                ]}
            done
            ln -s faelight-shell "$out/bin/fsh"
          '';
        });
        core = craneLib.buildPackage (faelightCommonArgs // {
          pname = "core";
          version = "3.1.0";
          cargoArtifacts = faelightDeps;
          cargoExtraArgs = "-p core --locked";
        });
        faelight-bar-gtk = let
          py = pkgs.python3.withPackages (ps: [ ps.pygobject3 ]);
        in pkgs.stdenv.mkDerivation {
          pname = "faelight-bar-gtk";
          version = "0.1.0";
          src = ./faelight/packages/faelight-bar-gtk;
          nativeBuildInputs = [ pkgs.wrapGAppsHook4 pkgs.gobject-introspection ];
          buildInputs = [ py pkgs.gtk4 pkgs.gtk4-layer-shell pkgs.librsvg pkgs.adwaita-icon-theme ];
          dontConfigure = true;
          dontBuild = true;
          installPhase = ''
            runHook preInstall
            mkdir -p $out/bin
            { echo '#!${py}/bin/python3'; cat main.py; } > $out/bin/faelight-bar-gtk
            chmod +x $out/bin/faelight-bar-gtk
            runHook postInstall
          '';
          preFixup = ''
            gappsWrapperArgs+=(
              --set GDK_BACKEND wayland
              --set LD_PRELOAD ${pkgs.gtk4-layer-shell}/lib/libgtk4-layer-shell.so
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.systemd ]}
            )
          '';
        };

        faelight-logout = let
          py = pkgs.python3.withPackages (ps: [ ps.pygobject3 ]);
        in pkgs.stdenv.mkDerivation {
          pname = "faelight-logout";
          version = "0.1.0";
          src = ./faelight/packages/faelight-logout;
          nativeBuildInputs = [ pkgs.wrapGAppsHook4 pkgs.gobject-introspection ];
          buildInputs = [ py pkgs.gtk4 pkgs.gtk4-layer-shell pkgs.librsvg pkgs.adwaita-icon-theme ];
          dontConfigure = true;
          dontBuild = true;
          installPhase = ''
            runHook preInstall
            mkdir -p $out/bin
            { echo '#!${py}/bin/python3'; cat main.py; } > $out/bin/faelight-logout
            chmod +x $out/bin/faelight-logout
            runHook postInstall
          '';
          preFixup = ''
            gappsWrapperArgs+=(
              --set GDK_BACKEND wayland
              --set LD_PRELOAD ${pkgs.gtk4-layer-shell}/lib/libgtk4-layer-shell.so
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.systemd ]}
            )
          '';
        };

        # INT-084: faelight-launcher -- candy-neon GTK4 app launcher (logout's twin recipe).
        # wrapGAppsHook4 + the LD_PRELOAD preFixup make layer-shell work as a clean binary.
        faelight-launcher = let
          py = pkgs.python3.withPackages (ps: [ ps.pygobject3 ]);
        in pkgs.stdenv.mkDerivation {
          pname = "faelight-launcher";
          version = "0.1.0";
          src = ./faelight/packages/faelight-launcher;
          nativeBuildInputs = [ pkgs.wrapGAppsHook4 pkgs.gobject-introspection ];
          buildInputs = [ py pkgs.gtk4 pkgs.gtk4-layer-shell pkgs.librsvg pkgs.adwaita-icon-theme ];
          dontConfigure = true;
          dontBuild = true;
          installPhase = ''
            runHook preInstall
            mkdir -p $out/bin
            { echo '#!${py}/bin/python3'; cat main.py; } > $out/bin/faelight-launcher
            chmod +x $out/bin/faelight-launcher
            runHook postInstall
          '';
          preFixup = ''
            gappsWrapperArgs+=(
              --set GDK_BACKEND wayland
              --set LD_PRELOAD ${pkgs.gtk4-layer-shell}/lib/libgtk4-layer-shell.so
            )
          '';
        };
      };
      # INT-061 harness: anti-lockout VM boot test. Proves framework16 boots
      # headlessly + greetd reachable BEFORE any boot-critical move hits metal.
      checks.${system} = {
        framework16-boot = import ./nix/tests/framework16.nix {
          inherit pkgs self inputs;
        };
        # INT-119: sandboxed hook gate. `nix flake check` runs these in a Nix sandbox
        # (read-only FS, no network, pinned tools) -- the unskippable/reproducible
        # guarantee faelight-hooks (host-tool shell-outs, skips when absent) can't give.
        # faelight-hooks stays the commit-time authority; this is the flake-check gate.
        pre-commit-check = inputs.git-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            rustfmt.enable = true;      # overlaps faelight-hooks rustfmt -- but GUARANTEED here
            ripsecrets.enable = true;   # secret scan (overlaps gitleaks) -- sandboxed
          };
        };
      };


      devShells.${system}.default = let
        # INT-122: build our candy-neon neovim via nixCats (real Lua, migrated off nixvim).
        forestNvim = import ./nix/home/dotfiles/forest-nvim {
          inherit pkgs;
          nixCats = inputs.nixcats;
        };
      in pkgs.mkShell {
        name = "friday-dev";
        buildInputs = (with pkgs; [
          # Rust toolchain
          rustc
          cargo
          rust-analyzer
          clippy
          rustfmt
          # Build tools
          pkg-config
          clang
          cmake
          openssl
          openssl.dev
          # Forest tools
          sqlite
          python3
          git
          # Cargo tools
          cargo-audit
          cargo-watch
          bacon
          cargo-nextest
          # Dev utilities
          ripgrep
          fd
          jq
        ]) ++ [ forestNvim ];
        shellHook = ''
          echo "🌲 Faelight Forest -- friday-dev shell"
          echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
          echo "  rustc: $(rustc --version)"
          echo "  cargo: $(cargo --version)"
          echo "  Ready for forest development"
          echo ""
        '';
      };
    };
}

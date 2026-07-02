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
  # INT-090 Phase 3: nixvim for the friday-dev devShell. Pin nixos-26.05 (NOT main); NO
  # nixpkgs.follows -- Phase 0 lesson: let nixvim bring its own tested nixpkgs.
  inputs.nixvim.url = "github:nix-community/nixvim/nixos-26.05";

  outputs = { self, nixpkgs, home-manager, disko, nixos-hardware, pinnacle, crane, nixvim, ... }@inputs:
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
          ./hosts/vm/base.nix
          ./hosts/vm/login-mirror.nix
        ];
      };

      nixosConfigurations.faelight-vm-regreet = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit self system inputs; };
        modules = [
          home-manager.nixosModules.home-manager
          ./hosts/vm/base.nix
          ./hosts/vm/login-regreet.nix
        ];
      };

      nixosConfigurations.framework16 = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit self system inputs; };
        modules = [
          disko.nixosModules.disko
          ./hosts/framework16/disko.nix
          nixos-hardware.nixosModules.framework-16-7040-amd
          home-manager.nixosModules.home-manager
          ./hosts/framework16/configuration.nix
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

      devShells.${system}.default = let
        # INT-090 Phase 3: build our candy-neon nixvim as an `nvim` package for this shell only.
        forestNvim = nixvim.legacyPackages.${system}.makeNixvimWithModule {
          inherit pkgs;
          module = import ./config/nixvim/default.nix;
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

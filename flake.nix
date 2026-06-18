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
      });
    in {
      nixosConfigurations.faelight-vm = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit self system inputs; };
        modules = [
          home-manager.nixosModules.home-manager
          ./hosts/vm/configuration.nix
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
        ];
      };

      packages.${system} = {
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
          src = ./pkgs/faelight-bar-gtk;
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
          src = ./pkgs/faelight-logout;
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
      };

      devShells.${system}.default = pkgs.mkShell {
        name = "friday-dev";
        buildInputs = with pkgs; [
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
        ];
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

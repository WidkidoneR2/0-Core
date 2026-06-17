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

  outputs = { self, nixpkgs, home-manager, disko, nixos-hardware, pinnacle, ... }@inputs:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
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
        faelight-forest = pkgs.rustPlatform.buildRustPackage {
          pname = "faelight-forest";
          version = "9.2.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "smithay-0.7.0" = "sha256-nZCWI3dmDVWBXpKiw3gtemYitUOzDjL12yVWYDYSM2E=";
              "smithay-drm-extras-0.1.0" = "sha256-nZCWI3dmDVWBXpKiw3gtemYitUOzDjL12yVWYDYSM2E=";
            };
          };
          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.rustPlatform.bindgenHook
            pkgs.cmake
	    pkgs.makeWrapper
          ];
          dontUseCmakeConfigure = true;
          buildInputs = [
            pkgs.wayland
            pkgs.libxkbcommon
            pkgs.libinput
            pkgs.libdisplay-info
            pkgs.seatd
            pkgs.udev
            pkgs.libdrm
            pkgs.libgbm
            pkgs.libGL
            pkgs.openssl
            pkgs.zlib
            pkgs.fontconfig
            pkgs.freetype
            pkgs.vulkan-loader
            pkgs.pango
            pkgs.cairo
            pkgs.pixman
            pkgs.dbus
            pkgs.pam
            pkgs.libsodium
          ];
          cargoBuildFlags = [ "--workspace" ];
          doCheck = false;
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
          '';

        };

        core = pkgs.rustPlatform.buildRustPackage {
          pname = "core";
          version = "3.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "smithay-0.7.0" = "sha256-nZCWI3dmDVWBXpKiw3gtemYitUOzDjL12yVWYDYSM2E=";
              "smithay-drm-extras-0.1.0" = "sha256-nZCWI3dmDVWBXpKiw3gtemYitUOzDjL12yVWYDYSM2E=";
            };
          };
          cargoBuildFlags = [ "-p" "core" ];
          doCheck = false;
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

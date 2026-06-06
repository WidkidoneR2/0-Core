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
        specialArgs = { inherit self system; };
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
          version = "9.1.0";
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

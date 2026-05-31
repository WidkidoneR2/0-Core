{
  description = "Faelight NixOS";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
  inputs.home-manager.url = "github:nix-community/home-manager/release-25.11";
  inputs.home-manager.inputs.nixpkgs.follows = "nixpkgs";

  outputs = { self, nixpkgs, home-manager }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      nixosConfigurations.faelight-vm = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit self system; };
        modules = [
          home-manager.nixosModules.home-manager
          ./hosts/vm/configuration.nix
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
    };
}

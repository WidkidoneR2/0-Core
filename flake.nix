{
  description = "Faelight NixOS";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      nixosConfigurations.faelight-vm = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [ ./hosts/vm/configuration.nix ];
      };

      packages.${system} = {
        get-version = pkgs.rustPlatform.buildRustPackage {
          pname = "get-version";
          version = "4.0.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "smithay-0.7.0" = "sha256-nZCWI3dmDVWBXpKiw3gtemYitUOzDjL12yVWYDYSM2E=";
            };
          };
          cargoBuildFlags = [ "-p" "get-version" ];
          doCheck = false;
        };

        faelight-compositor = pkgs.rustPlatform.buildRustPackage {
          pname = "faelight-compositor";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "smithay-0.7.0" = "sha256-nZCWI3dmDVWBXpKiw3gtemYitUOzDjL12yVWYDYSM2E=";
              "smithay-drm-extras-0.1.0" = "sha256-nZCWI3dmDVWBXpKiw3gtemYitUOzDjL12yVWYDYSM2E=";
            };
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
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
          ];
          cargoBuildFlags = [ "-p" "faelight-compositor" ];
          doCheck = false;
        };

        faelight-shell = pkgs.rustPlatform.buildRustPackage {
          pname = "faelight-shell";
          version = "2.3.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "smithay-0.7.0" = "sha256-nZCWI3dmDVWBXpKiw3gtemYitUOzDjL12yVWYDYSM2E=";
              "smithay-drm-extras-0.1.0" = "sha256-nZCWI3dmDVWBXpKiw3gtemYitUOzDjL12yVWYDYSM2E=";
            };
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl pkgs.zlib ];
          cargoBuildFlags = [ "-p" "faelight-shell" ];
          doCheck = false;
        };
      };
    };
}

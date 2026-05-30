{
  description = "Faelight NixOS";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";

  outputs = { self, nixpkgs }: {
    nixosConfigurations.faelight-vm = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [ ./hosts/vm/configuration.nix ];
    };
  };
}

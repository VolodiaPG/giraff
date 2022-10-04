{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
  };

  outputs = { self, nixpkgs, ... }@inputs:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs { inherit system; config = { allowUnfree = true; }; };
      lib = nixpkgs.lib;

    in
    {
      nixosConfigurations.isoimage = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          ./iso.nix
          "${nixpkgs}/nixos/modules/installer/cd-dvd/installation-cd-base.nix"
          #"${nixpkgs}/nixos/modules/installer/cd-dvd/iso-image.nix"
        ];
      };

      devShells.${system} = {
        default = pkgs.mkShell {
          buildInputs = [
            pkgs.python3
            pkgs.ansible
            (pkgs.callPackage ./enoslib {})
          ];
        };
      };
    };
}

{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    alejandra = {
      url = "github:kamadorueda/alejandra/3.0.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    impermanence.url = "github:nix-community/impermanence";
    flake-utils.url = "github:numtide/flake-utils";
  };

  nixConfig = {
    extra-trusted-substituters = "https://nix-community.cachix.org";
    extra-trusted-public-keys = "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs=";
  };

  outputs = inputs:
    with inputs; let
      inherit (self) outputs;
    in
      flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (
        system: let
          pkgs = nixpkgs.legacyPackages.${system};
        in {
          packages = import ./pkgs {inherit pkgs inputs outputs;};
        }
      )
      // flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = nixpkgs.legacyPackages.${system};
        in {
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              just
              nixos-rebuild
              qemu
              mprocs
              nix-output-monitor
              sshpass
            ];
          };
        }
      )
      // {
        nixosModules = import ./modules;

        formatter = alejandra.defaultPackage;
      };
}

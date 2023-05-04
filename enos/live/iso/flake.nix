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
  };

  nixConfig = {
    extra-trusted-substituters = "https://nix-community.cachix.org";
    extra-trusted-public-keys = "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs=";
  };

  outputs = inputs:
    with inputs; let
      inherit (self) outputs;

      forAllSystems = nixpkgs.lib.genAttrs [
        "aarch64-linux"
        "x86_64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];

      forVMSystems = nixpkgs.lib.genAttrs [
        "aarch64-linux"
        "x86_64-linux"
      ];
    in {
      formatter = forAllSystems (system: alejandra.defaultPackage.${system});

      nixosModules = import ./modules;

      packages = forVMSystems (
        system: let
          pkgs = nixpkgs.legacyPackages.${system};
        in
          import ./pkgs {inherit pkgs inputs outputs;}
      );

      devShells = forAllSystems (
        system: let
          pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              just
              nixos-rebuild
              qemu
              mprocs
              libguestfs-with-appliance
              nix-output-monitor
            ];
          };
        }
      );
    };
}

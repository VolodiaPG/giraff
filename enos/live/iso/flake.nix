{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    srvos.url = "github:numtide/srvos";
    srvos.inputs.nixpkgs.follows = "nixpkgs";
    flake-compat.url = "github:edolstra/flake-compat";
    flake-compat.flake = false;
    impermanence.url = "github:nix-community/impermanence";
    flake-utils.url = "github:numtide/flake-utils";
    gomod2nix.url = "github:nix-community/gomod2nix";
  };

  nixConfig = {
    extra-trusted-substituters = "https://nix-community.cachix.org";
    extra-trusted-public-keys = "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs=";
  };

  outputs = inputs:
    with inputs; let
      inherit (self) outputs;
      proxyFlake = import ./proxy/flake.nix;
      # proxyLock = builtins.fromJSON (builtins.readFile ./proxy/flake.lock);
      proxyFlakeOutputs = proxyFlake.outputs {
        inherit nixpkgs;
        inherit flake-utils;
        # inherit (proxyLock.nodes) gomod2nix;
        inherit gomod2nix;
      };
    in
      nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate {}
      [
        (flake-utils.lib.eachDefaultSystem (
          system: {
            packages.proxy = proxyFlakeOutputs.packages.${system}.default;
          }
        ))
        (flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (
          system: let
            pkgs = nixpkgs.legacyPackages.${system};
            modules = with outputs.nixosModules; [
              base
              configuration
              filesystem
              init
              monitoring
              squid
            ];
          in {
            packages.vm = import ./pkgs {inherit pkgs inputs outputs modules;};
          }
        ))
        (flake-utils.lib.eachDefaultSystem (
          system: let
            pkgs = nixpkgs.legacyPackages.${system};
          in {
            devShells.default = pkgs.mkShell {
              nativeBuildInputs = with pkgs; [
                just
                nixos-rebuild
                qemu
                mprocs
                sshpass
              ];
            };
            formatter = pkgs.alejandra;
          }
        ))
        {
          nixosModules = import ./modules;
        }
      ];
}

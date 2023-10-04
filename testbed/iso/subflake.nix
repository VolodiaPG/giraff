{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
      proxyFlakeOutputs = (import ./proxy/subflake.nix).outputs inputs extra;
    in
      nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate {}
      [
        proxyFlakeOutputs
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
              proxy
            ];
          in {
            packages.vm = import ./pkgs {inherit pkgs inputs outputs modules;};
          }
        ))
        (flake-utils.lib.eachDefaultSystem (
          system: let
            pkgs = nixpkgs.legacyPackages.${system};
          in {
            devShells.iso = pkgs.mkShell {
              nativeBuildInputs = with pkgs; [
                just
                nixos-rebuild
                qemu
                mprocs
                sshpass
              ];
            };
          }
        ))
        {
          nixosModules = import ./modules;
        }
      ];
}

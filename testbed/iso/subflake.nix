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
            packages.openfaas =
              (kubenix.evalModules.${system} {
                module = {kubenix, ...}: {
                  imports = [kubenix.modules.k8s kubenix.modules.helm];
                  kubernetes.helm.releases.openfaas = {
                    namespace = nixpkgs.lib.mkForce "openfaas";
                    overrideNamespace = false;
                    chart = kubenix.lib.helm.fetch {
                      repo = "https://openfaas.github.io/faas-netes/";
                      chart = "openfaas";
                      version = "14.1.9";
                      sha256 = "sha256-KxZhrunv8DbOvFqw7p2t2Zrqm4urvFWCErsutqNUgiM=";
                    };
                  };
                };
              })
              .config
              .kubernetes
              .result;
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

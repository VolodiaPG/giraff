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
            packages.proxy = proxyFlakeOutputs.packages.${system}.proxy;
          }
        ))
        (flake-utils.lib.eachSystem ["x86_64-linux"] (
          system: let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [
                ebpf-netem.overlays.${system}.default
              ];
            };
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
                    chart = pkgs.stdenvNoCC.mkDerivation {
                      name = "openfaas";
                      src = inputs.openfaas;

                      buildCommand = ''
                        ls $src
                        cp -r $src/chart/openfaas/ $out
                      '';
                    };
                  };
                  # kubernetes.resources = {
                  #   services.proxy = {
                  #     metadata.namespace = "openfaas";
                  #     metadata.labels.app = "fn";
                  #     spec = {
                  #       selector.app = "fn";
                  #       type= "ExternalName";
                  #       externalName="10.0.100.100";
                  #       ports = [
                  #         {
                  #           protocol = "TCP";
                  #           port = 3128;
                  #         }
                  #       ];
                  #     };
                  #   };
                  #   # services.proxy = {
                  #   #   metadata.namespace = "openfaas";
                  #   #   metadata.labels.app = "fn";
                  #   #   spec = {
                  #   #     selector.app = "fn";
                  #   #     type= "ClusterIP";
                  #   #     clusterIP = "None";
                  #   #     # selector.name = "proxy";
                  #   #     ports = [
                  #   #       {
                  #   #         name = "http";
                  #   #         protocol = "TCP";
                  #   #         port = 3128;
                  #   #         targetPort = 3128;
                  #   #         # nodePort = 30128;
                  #   #       }
                  #   #     ];
                  #   #   };
                  #   # };
                  #   # endpoints.proxy = {
                  #   #   metadata.namespace = "openfaas";
                  #   #   metadata.labels.app = "fn";
                  #   #   # selector.labels.app = "fn";
                  #   #   # selector.matchLabels.app = "fn";
                  #   #   subsets = [
                  #   #     {
                  #   #       addresses = [
                  #   #         # {ip = "10.0.100.100";}
                  #   #         {ip = "10.0.2.15";}
                  #   #       ];
                  #   #       ports = [
                  #   #         {
                  #   #           port = 3128;
                  #   #         }
                  #   #       ];
                  #   #     }
                  #   #   ];
                  #   # };
                  # };
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
              shellHook = (extra.shellHook system) "iso";
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

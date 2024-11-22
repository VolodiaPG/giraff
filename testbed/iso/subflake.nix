{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
      inherit (nixpkgs) lib;
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
                # ebpf-netem.overlays.${system}.default
              ];
            };
            modules = with outputs.nixosModules; [
              base
              configuration
              filesystem
              init
              monitoring
              proxy
              registry
            ];
          in {
            packages.nixosConfigurations.node_vm = nixpkgs.lib.nixosSystem {
              inherit system;
              specialArgs = {
                inherit inputs outputs;
              };
              modules =
                [
                  impermanence.nixosModules.impermanence
                  disko.nixosModules.disko
                  srvos.nixosModules.server
                  outputs.nixosModules.disk
                  "${nixpkgs}/nixos/modules/profiles/qemu-guest.nix"
                  "${nixpkgs}/nixos/modules/profiles/all-hardware.nix"
                  {
                    disko.devices.disk.sda.imageSize = "15G";

                    networking.hostName = "giraff";
                    system.stateVersion = "22.05"; #config.system.nixos.version;
                  }
                ]
                ++ modules;
            };
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
                  kubernetes.resources.deployments = {
                    gateway.spec.template.spec.containers.gateway.image = lib.mkForce "ghcr.io/volodiapg/openfaas/gateway:0.27.2";
                    gateway.spec.template.spec.containers.faas-netes.image = lib.mkForce "ghcr.io/volodiapg/openfaas/faas-netes:0.17.1";
                    queue-worker.spec.template.spec.containers.queue-worker.image = lib.mkForce "ghcr.io/volodiapg/openfaas/queue-worker:0.14.0";
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

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
            packages.nixosConfigurations.node_vm = nixpkgs.lib.nixosSystem {
              inherit system;
              specialArgs = {
                inherit inputs outputs;
              };
              modules =
                [
                  impermanence.nixosModules.impermanence
                  disko.nixosModules.disko
                  inputs.srvos.nixosModules.server
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

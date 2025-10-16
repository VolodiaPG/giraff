{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
      inherit (nixpkgs) lib;
      proxyFlakeOutputs = (import ./proxy/subflake.nix).outputs inputs extra;
    in
      nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate {} [
        proxyFlakeOutputs
        (flake-utils.lib.eachDefaultSystem (system: {
          packages.proxy = proxyFlakeOutputs.packages.${system}.proxy;
        }))
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
            packages = {
              nixosConfigurations.node_vm = lib.nixosSystem {
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
                      disko.devices.disk.sda.imageSize = "50G";

                      networking.hostName = "giraff";
                      system.stateVersion = "22.05"; # config.system.nixos.version;
                    }
                  ]
                  ++ modules;
              };
              nixosConfigurations.vagrant = lib.nixosSystem {
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
                      environment.etc."rancher/k3s/registries.yaml".text = lib.mkForce ''
                        configs:
                          "ghcr.io":
                          "docker.io":
                          "*":
                            tls:
                              insecure_skip_verify: true

                      '';

                      security.sudo.enable = nixpkgs.lib.mkForce true;
                      environment.systemPackages = with pkgs; [
                        findutils
                        gnumake
                        iputils
                        jq
                        nettools
                        netcat
                        nfs-utils
                        rsync
                      ];
                      services.openssh = {
                        enable = true;
                        extraConfig = ''
                          PubkeyAcceptedKeyTypes +ssh-rsa
                        '';
                        settings.KexAlgorithms = nixpkgs.lib.mkForce [
                          "sntrup761x25519-sha512@openssh.com"
                          "curve25519-sha256"
                          "curve25519-sha256@libssh.org"
                          "diffie-hellman-group-exchange-sha256"
                        ];
                      };
                      users = {
                        groups.vagrant = {
                          name = "vagrant";
                          members = ["vagrant"];
                        };

                        users = {
                          # Creates a "vagrant" group /& user with password-less sudo access
                          vagrant = {
                            description = "Vagrant User";
                            name = "vagrant";
                            group = "vagrant";
                            extraGroups = [
                              "users"
                              "wheel"
                              "root"
                            ];
                            password = "vagrant";
                            home = lib.mkForce "/home/vagrant";
                            createHome = true;
                            # useDefaultShell = true;
                            openssh.authorizedKeys.keys = [
                              "ssh-rsa AAAAB3NzaC1yc2EAAAABIwAAAQEA6NF8iallvQVp22WDkTkyrtvp9eWW6A8YVr+kz4TjGYe7gHzIw+niNltGEFHzD8+v1I2YJ6oXevct1YeS0o9HZyN1Q9qgCgzUFtdOKLv6IedplqoPkcmF0aYet2PkEDo3MlTBckFXPITAMzF8dJSIFo9D8HfdOV0IAdx4O7PtixWKn5y2hMNG0zQPyUecp4pzC6kivAIhyfHilFR61RGL+GPXQ2MWZWFYbAGjyiYJnAmCP3NOTd0jMZEnDkbUvxhMmBYSdETk1rRgm+R4LOzFUGaHqHDLKLX+FIPKcF96hrucXzcWyLbIbEgE98OHlnVYCzRdK8jlqm8tehUc9c9WhQ== vagrant insecure public key"
                            ];
                            isNormalUser = lib.mkForce true;
                            isSystemUser = lib.mkForce false;
                          };
                          root = {
                            password = lib.mkForce "vagrant";
                            openssh.authorizedKeys.keys = [
                              "ssh-rsa AAAAB3NzaC1yc2EAAAABIwAAAQEA6NF8iallvQVp22WDkTkyrtvp9eWW6A8YVr+kz4TjGYe7gHzIw+niNltGEFHzD8+v1I2YJ6oXevct1YeS0o9HZyN1Q9qgCgzUFtdOKLv6IedplqoPkcmF0aYet2PkEDo3MlTBckFXPITAMzF8dJSIFo9D8HfdOV0IAdx4O7PtixWKn5y2hMNG0zQPyUecp4pzC6kivAIhyfHilFR61RGL+GPXQ2MWZWFYbAGjyiYJnAmCP3NOTd0jMZEnDkbUvxhMmBYSdETk1rRgm+R4LOzFUGaHqHDLKLX+FIPKcF96hrucXzcWyLbIbEgE98OHlnVYCzRdK8jlqm8tehUc9c9WhQ== vagrant insecure public key"
                            ];
                          };
                        };
                      };

                      security.sudo.extraConfig = ''
                        Defaults:root,%wheel env_keep+=LOCALE_ARCHIVE
                        Defaults:root,%wheel env_keep+=NIX_PATH
                        Defaults:root,%wheel env_keep+=TERMINFO_DIRS
                        Defaults env_keep+=SSH_AUTH_SOCK
                        Defaults lecture = never
                        root   ALL=(ALL) SETENV: ALL
                        %wheel ALL=(ALL) NOPASSWD: ALL, SETENV: ALL
                      '';

                      disko.devices.disk.sda.imageSize = "50G";

                      networking.hostName = "giraff";
                      system.stateVersion = "22.05"; # config.system.nixos.version;
                    }
                  ]
                  ++ modules;
              };
              openfaas =
                (kubenix.evalModules.${system} {
                  module = {kubenix, ...}: {
                    imports = [
                      kubenix.modules.k8s
                      kubenix.modules.helm
                    ];
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
                      # the paths are obtained looking at the k9s listing: image
                      # and then one click inside the second name
                      gateway.spec.template.spec.containers.gateway.image =
                        lib.mkForce "ghcr.io/volodiapg/openfaas/gateway:0.27.2";
                      gateway.spec.template.spec.containers.faas-netes.image =
                        lib.mkForce "ghcr.io/volodiapg/openfaas/faas-netes:0.17.1";
                      queue-worker.spec.template.spec.containers.queue-worker.image =
                        lib.mkForce "ghcr.io/volodiapg/openfaas/queue-worker:0.14.0";
                      alertmanager.spec.template.spec.containers.alertmanager.image =
                        lib.mkForce "ghcr.io/volodiapg/prom/alertmanager:v0.26.0";
                      prometheus.spec.template.spec.containers.prometheus.image =
                        lib.mkForce "ghcr.io/volodiapg/prom/prometheus:v2.47.2";
                      nats.spec.template.spec.containers.nats.image =
                        lib.mkForce "ghcr.io/volodiapg/nats/nats-streaming:0.25.5";
                      # metrics-server.spec.template.spec.containers.metrics-server.image =
                      #   lib.mkForce
                      #   "ghcr.io/volodiapg/rancher/mirrored-metrics-server:0.7.0";
                    };
                  };
                }).config.kubernetes.result;
            };
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

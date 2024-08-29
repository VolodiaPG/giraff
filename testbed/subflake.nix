{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
      # Load the iso flake
      isoOutputs = (import ./iso/subflake.nix).outputs inputs extra;
      miningOutputs = (import ./mining/subflake.nix).outputs inputs extra;
      expeOutputs = (import ./expe/subflake.nix).outputs inputs extra;
    in
      nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate {}
      [
        isoOutputs
        miningOutputs
        expeOutputs
        (flake-utils.lib.eachDefaultSystem (
          system: let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [overlay];
            };
            overlay = final: prev: {
              pythonPackagesExtensions =
                prev.pythonPackagesExtensions
                ++ [
                  (
                    python-final: python-prev: {
                      randomname = python-prev.buildPythonPackage rec {
                        pname = "randomname";
                        version = "0.2.1";

                        src = prev.fetchPypi rec {
                          inherit pname version;
                          hash = "sha256-t5uYMCukR5FksKT4eZW3vrvR2RASrtpIM0Hj5YrOUg4=";
                        };

                        doCheck = false;
                        doInstallCheck = false;
                      };
                      python-grid5000 = python-prev.buildPythonPackage rec {
                        pname = "python-grid5000";
                        version = "1.2.4";
                        src = prev.fetchFromGitLab {
                          domain = "gitlab.inria.fr";
                          owner = "msimonin";
                          repo = pname;
                          rev = "v${version}";
                          sha256 = "sha256-wfDyoaOn0Dlbz/metxskbN4frsJbkEe8byUeO01upV8=";
                        };
                        doCheck = false;
                        propagatedBuildInputs = with python-prev; [
                          pyyaml
                          requests
                          ipython
                        ];
                      };
                      ansible = python-prev.ansible.overridePythonAttrs (
                        old: rec {
                          version = "8.7.0";
                          src = python-prev.fetchPypi {
                            inherit (old) pname;
                            inherit version;
                            hash = "sha256-OlylFS5FR9WQ5AtULXaxjbvis22k7dAKE6fFGjdP9zc=";
                          };
                        }
                      );
                      ansible-core =
                        (python-prev.ansible-core.overridePythonAttrs (
                          old: rec {
                            version = "2.14.15";
                            src = prev.fetchPypi {
                              inherit (old) pname;
                              inherit version;
                              hash = "sha256-+YciLt86wdGnyhNW6fSBj76SIsE2gUNuvBn9+a+ZTp0=";
                            };
                          }
                        ))
                        .override {inherit (python-final) ansible;};
                    }
                  )
                ];
              experiments = final.python312.withPackages (ps: (with ps; [
                dill
                click
                types-click
                aiohttp
                aiodns
                brotli
                influxdb-client
                marshmallow-dataclass
                randomname
                numpy
                (buildPythonPackage {
                  pname = "enoslib";
                  src = inputs.enoslib;
                  version = "${inputs.enoslib.shortRev}";

                  propagatedBuildInputs = [
                    cryptography
                    sshtunnel
                    ipywidgets
                    rich
                    jsonschema
                    packaging
                    pytz
                    importlib-resources
                    # Pakcaged by me
                    python-grid5000
                    ansible-core
                  ];
                  doCheck = false;
                })
              ]));
              ansible_cfg = prev.writeText "ansible.cfg" ''
                [defaults]
                host_key_checking = False
                gathering = explicit
                verbosity = 3
                [ssh_connection]
                pipelining = True
                transfer_method = scp
                retries = 3
              '';
            };
          in {
            packages = {
              inherit (pkgs) experiments ansible_cfg;
            };
            devShells.testbed = pkgs.mkShell {
              shellHook =
                ((extra.shellHook system) "testbed")
                + ''
                  ln -sfn ${pkgs.ansible_cfg} ansible.cfg
                '';

              PYTHON_KEYRING_BACKEND = "keyring.backends.null.Keyring";

              packages = with pkgs; [
                outputs.packages.${system}.expe
                nerdctl
                just
                jq
                ruff
                black
                isort
                mypy
                experiments
                parallel
                bashInteractive
                bc
                tmux
                mprocs
                rsync
                moreutils # sponge is useful for buffering large cmd outs
                skopeo
              ];
            };
          }
        ))
        (flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (system: let
          pkgs = nixpkgs.legacyPackages.${system};

          basis = pkgs.dockerTools.buildImage {
            name = "enos_deployment_base";
            tag = "latest";
            runAsRoot = ''
              #!${pkgs.runtimeShell}
              ${pkgs.dockerTools.shadowSetup}
              groupadd -g 1000 enos
              useradd -u 1000 -g 1000 enos
              mkdir -p /home/enos
              chown enos:enos -R /home/enos

              mkdir -p /tmp
              mkdir -p /usr/bin
              ln -s ${pkgs.busybox}/bin/env /usr/bin/env
            '';
          };

          dockerImage = pkgs.dockerTools.streamLayeredImage {
            name = "enos_deployment";
            tag = "latest";
            fromImage = basis;
            contents =
              (with pkgs; [
                # Linux toolset
                busybox
                gnused
                bashInteractive
                parallel
                cacert

                # My toolset
                just
                jq
                openssh
                curl
                moreutils
                skopeo
                nerdctl
              ])
              ++ (with outputs.packages.${system}; [
                # Environment to run enos and stuff
                experiments
              ]);
          };
        in {
          packages.docker = dockerImage;
        }))
        {
          nixosModules.enosvm = {
            pkgs,
            lib,
            outputs,
            ...
          }: let
            binPath = with pkgs;
              lib.strings.makeBinPath (
                [
                  podman
                  outputs.packages.${pkgs.system}.experiments
                ]
                ++ outputs.devShells.${pkgs.system}.testbed.buildInputs
                ++ outputs.devShells.${pkgs.system}.testbed.nativeBuildInputs
                ++ outputs.devShells.${pkgs.system}.testbed.propagatedBuildInputs
                ++ stdenv.initialPath
              );
          in {
            virtualisation.podman.enable = true;

            systemd.services.experiment = {
              description = "Start the experiment";
              after = ["mountNfs.service"];
              wantedBy = ["multi-user.target"];
              script = ''
                set -ex
                # Fails if not active
                systemctl is-active mountNfs
                # Load the container for later use
                ${outputs.packages.${pkgs.system}.docker} | ${pkgs.podman}/bin/podman load

                export PATH=${binPath}:$PATH
                mkdir -p /home/enos
                find /nfs/enosvm -exec ln -s "{}" /home/enos/ ';'
                touch /home/enos/env.source
                echo 'export PATH=${binPath}:$PATH' | tee /home/enos/env.source
                cp ${outputs.packages.${pkgs.system}.ansible_cfg} /home/enos/ansible.cfg
              '';
              serviceConfig = {
                Type = "oneshot";
                RemainAfterExit = "yes";
                Restart = "on-failure";
                RestartSec = "3";
              };
            };
          };
        }
        (flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (system: let
          modules = with isoOutputs.nixosModules;
            [
              base
              filesystem
            ]
            ++ [
              outputs.nixosModules.enosvm
              impermanence.nixosModules.impermanence
              disko.nixosModules.disko
              inputs.srvos.nixosModules.server
              outputs.nixosModules.disk
              "${nixpkgs}/nixos/modules/profiles/qemu-guest.nix"
              "${nixpkgs}/nixos/modules/profiles/all-hardware.nix"
              {
                disko.devices.disk.sda.imageSize = "20G";

                networking.hostName = "giraff-master";
                system.stateVersion = "22.05"; #config.system.nixos.version;
              }
            ];
        in {
          packages.nixosConfigurations.enosvm = nixpkgs.lib.nixosSystem {
            inherit system modules;
            specialArgs = {
              inherit inputs outputs;
            };
          };
          devShells.enosvm = isoOutputs.devShells.${system}.iso;
        }))
      ];
}

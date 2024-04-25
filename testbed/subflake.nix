{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
      # Load the iso flake
      isoOutputs = (import ./iso/subflake.nix).outputs inputs extra;
      miningOutputs = (import ./mining/subflake.nix).outputs inputs extra;
    in
      nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate {}
      [
        isoOutputs
        miningOutputs
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
              experiments = final.python311.withPackages (ps: (with ps; [
                dill
                click
                types-click
                aiohttp
                influxdb-client
                marshmallow-dataclass
                randomname
                numpy
                (buildPythonPackage rec {
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
                + (extra.shellHookPython pkgs.experiments.interpreter)
                + ''
                  ln -sfn ${pkgs.ansible_cfg} ansible.cfg
                '';

              PYTHON_KEYRING_BACKEND = "keyring.backends.null.Keyring";

              buildInputs = with pkgs; [
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
              ])
              ++ (with outputs.packages.${system}; [
                # Environment to run enos and stuff
                experiments
              ]);

            # extraCommands = ''
            #   ${pkgs.dockerTools.shadowSetup}
            #   # mkdir -p /home/enos
            #   mkdir -p -m 0777 /tmp
            #   mkdir -p -m 0777 /usr/bin
            #   ln -s ${pkgs.busybox}/bin/env /usr/bin/env
            # '';
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
          nixosModules.make-disk-image-stateless = isoOutputs.nixosModules.make-disk-image-stateless;
        }
        (flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (system: let
          pkgs = nixpkgs.legacyPackages.${system};
          modules = with isoOutputs.nixosModules;
            [
              base
              filesystem
            ]
            ++ [
              outputs.nixosModules.enosvm
            ];
          VMMounts = ''
            #!${pkgs.bash}/bin/bash
            set -ex
            mkdir -p $mountPoint
            sh -c ${outputs.packages.${pkgs.system}.docker} | gzip --fast > $mountPoint/output.gz
          '';
          inVMScript = ''
            set -ex
            gunzip -c output.gz | podman load
          '';
        in {
          packages = {
            enosvm = import ./iso/pkgs {inherit pkgs inputs outputs modules VMMounts inVMScript;};
          };
          devShells.enosvm = isoOutputs.devShells.${system}.iso;
        }))
      ];
}

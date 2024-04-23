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
                      #ansible = python-final.mitogen;
                      ansible = python-prev.ansible.overridePythonAttrs (
                        old: rec {
                          #pname = "ansible-${version}";
                          #version = "${builtins.toString inputs.enoslib-ansible.revCount}-${inputs.enoslib-ansible.shortRev}";
                          #src = inputs.enoslib-ansible;
                          version = "8.7.0";
                          src = python-prev.fetchPypi {
                            inherit (old) pname;
                            inherit version;
                            hash = "sha256-OlylFS5FR9WQ5AtULXaxjbvis22k7dAKE6fFGjdP9zc=";
                            #hash = "sha256-nCBrpRXxOgzJyRnUliGLom31gXVb3Dm+hbB0BmxpmgI=";
                          };
                          #makeWrapperArgs = [
                          #  "--suffix ANSIBLE_STRATEGY_PLUGINS : ${python-final.mitogen}/${final.python3.sitePackages}/ansible_mitogen/plugins/strategy"
                          #  #"--suffix ANSIBLE_STRATEGY_PLUGINS : ${python-prev.mitogen}/${prev.python3.sitePackages}/ansible_mitogen"
                          #  #"--suffix ${python-prev.mitogen}/lib/python${python-prev.pythonVersion}/site-packages/ansible_mitogen/plugins/strategy"
                          #  "--set-default ANSIBLE_STRATEGY mitogen_free"
                          #  #"--set PYTHONPATH $PYTHONPATH"
                          #];
                          #propagatedBuildInputs = old.propagatedBuildInputs ++ [python-final.mitogen];
                        }
                      );
                      #mitogen = python-prev.mitogen.overridePythonAttrs (old: rec {
                      #  version = "0.3.6";
                      #  src = prev.fetchFromGitHub {
                      #    owner = "mitogen-hq";
                      #    repo = "mitogen";
                      #    rev = "v${version}";
                      #    hash = "sha256-zQTto4SGPvQIXPAcTQx8FA+n/5RcpqKKn0UqlFM2yqI=";
                      #  };
                      #});
                      ansible-core =
                        (python-prev.ansible-core.overridePythonAttrs (
                          old: rec {
                            #version = "${builtins.toString inputs.enoslib-ansible.revCount}-${inputs.ansible.shortRev}";
                            #pname = "ansible-core-${version}";
                            #src = inputs.ansible;
                            makeWrapperArgs = [
                              "--suffix ANSIBLE_STRATEGY_PLUGINS : ${python-final.mitogen}/${final.python3.sitePackages}/ansible_mitogen"
                              #"--suffix ANSIBLE_STRATEGY_PLUGINS : ${python-prev.mitogen}/${prev.python3.sitePackages}/ansible_mitogen"
                              #"--suffix ${python-prev.mitogen}/lib/python${python-prev.pythonVersion}/site-packages/ansible_mitogen/plugins/strategy"
                              "--set-default ANSIBLE_STRATEGY mitogen_free"
                              #"--set PYTHONPATH $PYTHONPATH"
                            ];
                            propagatedBuildInputs = old.propagatedBuildInputs ++ [python-final.mitogen];

                            #version = "2.15.10";
                            version = "2.14.15";
                            #version = "2.12.10";
                            src = prev.fetchPypi {
                              inherit (old) pname;
                              inherit version;
                              #hash = "sha256-lU2+jk6AKk3V3wNmGTl1tpKgWAaqjXNYQYp+YXNGsg8=";
                              hash = "sha256-+YciLt86wdGnyhNW6fSBj76SIsE2gUNuvBn9+a+ZTp0=";
                              # hash = "sha256-/rHfYXOM/B9eiTtCouwafeMpd9Z+hnB7Retj0MXDwjY=";
                            };
                            #preInstall = ''
                            #  echo ${prev.python3Minimal.sitePackages}
                            #  exit 1
                            #'';
                            #postPatch = ''
                            #  substituteInPlace lib/ansible/executor/task_executor.py \
                            #    --replace "[python," "["
                            #'';

                            #nativeBuildInputs = with prev; [
                            #  installShellFiles
                            #];
                            #postInstall = ''
                            #  installManPage docs/man/man1/*.1
                            #'';
                          }
                        ))
                        .override {inherit ansible;};
                      #in {
                      #  inherit ansible-core ansible randomname python-grid5000 mitogen;
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
                    #ansible
                    ansible-core
                  ];
                  doCheck = false;
                })
              ]));
              ansible_cfg = prev.writeText "ansible.cfg" ''
                [defaults]
                #strategy_plugins = ${final.experiments}/${final.experiments.sitePackages}/ansible_mitogen/plugins/strategy
                #strategy = mitogen_free
                host_key_checking = False
                gathering = explicit
                #interpreter_python = ${final.experiments}/bin/python3
                verbosity = 3
                [ssh_connection]
                pipelining = True
                transfer_method = scp
                retries = 3
              '';
            };
            #ansible_python_interpreter = ${pkgs.experiments}/bin/python3
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
                #echo 'export ANSIBLE_STRATEGY_PLUGINS="${pkgs.python3Packages.mitogen}/${pkgs.python3Minimal.sitePackages}/ansible_mitogen/plugins/strategy"' >> /home/enos/env.source
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

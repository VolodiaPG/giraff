{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
      # Load the iso flake
      isoOutputs = (import ./iso/subflake.nix).outputs inputs extra;
    in
      nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate {}
      [
        isoOutputs
        (flake-utils.lib.eachDefaultSystem (
          system: let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [overlay];
            };
            overlay = _final: prev: {
              experiments = prev.python311.withPackages (ps: (with ps; [
                dill
                click
                aiohttp
                numpy
                influxdb-client
                marshmallow-dataclass
                (alive-progress.overridePythonAttrs
                  (
                    _old: {
                      postInstall = ''
                        rm $out/LICENSE
                      '';
                    }
                  ))
                (buildPythonPackage rec {
                  pname = "randomname";
                  version = "0.2.1";

                  src = prev.fetchPypi rec {
                    inherit pname version;
                    hash = "sha256-t5uYMCukR5FksKT4eZW3vrvR2RASrtpIM0Hj5YrOUg4=";
                  };

                  doCheck = false;
                  doInstallCheck = false;
                })
                (buildPythonPackage rec {
                  pname = "enoslib";
                  src = inputs.enoslib;
                  version = "${builtins.toString inputs.enoslib.revCount}-${inputs.enoslib.shortRev}";

                  # We do the following because nix cannot yet access the extra builds of poetry
                  patchPhase = ''
                    substituteInPlace setup.cfg --replace "importlib_resources>=5,<6" ""
                    substituteInPlace setup.cfg --replace "importlib_metadata>=6,<7" ""
                    substituteInPlace setup.cfg --replace "rich[jupyter]~=12.0.0" "rich>=12.0.0"
                    substituteInPlace setup.cfg --replace "packaging~=21.3" "packaging>=21.3"
                    substituteInPlace setup.cfg --replace "pytz~=2022.1" "pytz>=2022.1"
                    substituteInPlace setup.cfg --replace "ansible>=2.9,<7.2" "ansible>=2.9"
                  '';
                  propagatedBuildInputs = [
                    cryptography
                    sshtunnel
                    ipywidgets
                    rich
                    jsonschema
                    packaging
                    pytz
                    importlib-resources
                    (nixpkgs-ansible-enoslib.legacyPackages.${system}.python3Packages.ansible-core.override {
                      inherit (ps) callPackage buildPythonPackage fetchPypi cryptography jinja2 junit-xml lxml ncclient packaging paramiko pexpect psutil pycrypto pyyaml requests resolvelib scp xmltodict;
                      ansible = nixpkgs-ansible-enoslib.legacyPackages.${system}.python3Packages.ansible.override {
                        inherit (ps) buildPythonPackage fetchPypi jsonschema jxmlease ncclient netaddr paramiko pynetbox scp textfsm ttp xmltodict;
                      };
                    })
                    (buildPythonPackage rec {
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
                      propagatedBuildInputs = [
                        pyyaml
                        requests
                        ipython
                      ];
                    })
                  ];
                  doCheck = false;
                })
              ]));
            };
          in {
            packages.experiments = pkgs.experiments;
            devShells.testbed = pkgs.mkShell {
              shellHook =
                ((extra.shellHook system) "testbed")
                + (extra.shellHookPython pkgs.experiments.interpreter);

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
                ansible
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
        (flake-utils.lib.eachDefaultSystem (system: let
          pkgs = jupyenv.inputs.nixpkgs.legacyPackages.${system};
          jupyterlab = export:
            jupyenv.lib.${system}.mkJupyterlabNew ({...}: {
              imports = [
                {
                  kernel.r.experiment = {
                    runtimePackages =
                      (with pkgs; [bash toybox])
                      ++ (nixpkgs.lib.optionals export (with pkgs; [
                        texlive.combined.scheme-full
                        pgf3
                      ]));
                    enable = true;
                    name = "giraff";
                    displayName = "giraff";
                    extraRPackages = ps:
                      with ps;
                        [
                          (
                            archive.overrideAttrs (old: {
                              buildInputs =
                                old.buildInputs
                                ++ (with pkgs; [
                                  libarchive
                                ]);
                            })
                          )
                          cowplot
                          reticulate
                          vroom
                          tidyverse
                          igraph
                          r2r
                          formattable
                          stringr
                          viridis
                          geomtextpath
                          scales
                          zoo
                          gghighlight
                          ggdist
                          ggbreak
                          lemon
                          ggprism
                          ggh4x
                          ggExtra
                          tibbletime
                          snakecase
                          reshape2
                          ggside
                          ggbeeswarm
                          ggpubr
                          Hmisc
                          rstatix
                          multcompView

                          doParallel
                          foreach
                          multidplyr

                          magick
                          future_apply
                          (
                            gganimate.overrideAttrs (old: {
                              buildInputs =
                                old.buildInputs
                                ++ (with pkgs; [
                                  future_apply
                                ]);
                              src = pkgs.fetchgit {
                                url = "https://github.com/VolodiaPG/gganimate.git";
                                hash = "sha256-RGtqslMy2hommHJinaHlkamT+hvmD6hOTthc5DbV6xw=";
                              };
                            })
                          )
                          intergraph
                          network
                          ggnetwork
                        ]
                        ++ pkgs.lib.optional export tikzDevice;
                  };
                }
              ];
            });

          # Starts the script cleanly, outside what is currently messed up in the current shell
          jupyterlabScript = executable:
            pkgs.writeShellScriptBin "start" ''
              # Check if the script needs to clear the environment
              if [ -z "$CLEARED_ENV" ]; then
                  # Re-invoke the script with a cleared environment
                  env -i CLEARED_ENV=1 ${pkgs.bash}/bin/bash "$0" "$@"
                  exit $?
              fi

              exec -a "jupyter-giraff" ${executable} --NotebookApp.token="291d6ed55226ce5802cb0a8a6055fa5bffafbffb0d1f3e81"
            '';
        in {
          apps.jupyterlabExport = {
            program = "${nixpkgs.lib.getExe' (jupyerlabScript "${jupyterlab true}/bin/jupyter-lab") "start"}";
            type = "app";
          };
          apps.jupyterlab = {
            program = "${nixpkgs.lib.getExe' (jupyterlabScript "${jupyterlab false}/bin/jupyter-lab") "start"}";
            type = "app";
          };
        }))
      ];
}

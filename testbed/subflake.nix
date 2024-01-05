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
                # importlib-resources
                aiohttp
                influxdb-client
                # simpy
                # scipy
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
                  version = "v8.1.5";
                  src = prev.fetchFromGitLab {
                    domain = "gitlab.inria.fr";
                    owner = "discovery";
                    repo = pname;
                    rev = "${version}";
                    hash = "sha256-3pyPtj8xzbQVVtoFGAaOEuEp98qWAhc35yt3liZkIvM=";
                  };

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
                    # (nixpkgs-ansible-enoslib.legacyPackages.${system}.python3Packages.ansible-core.override {
                    #    inherit (ps) callPackage buildPythonPackage fetchPypi ansible cryptography jinja2 junit-xml lxml ncclient packaging paramiko pexpect psutil pycrypto pyyaml requests resolvelib scp xmltodict;
                    # })
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
                mprocs
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

          dockerImage = pkgs.dockerTools.streamLayeredImage {
            name = "enos_deployment";
            tag = "latest";
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

            config = {
            };
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
            df -h
            mkdir -p $mountPoint
            sh -c ${outputs.packages.${pkgs.system}.docker} | gzip --fast > $mountPoint/output.gz
          '';
          inVMScript = ''
            set -ex
            df -h
            gunzip -c output.gz | podman load
          '';
        in {
          packages.enosvm = import ./iso/pkgs {inherit pkgs inputs outputs modules VMMounts inVMScript;};
          devShells.enosvm = isoOutputs.devShells.${system}.iso;
        }))
        (flake-utils.lib.eachDefaultSystem (system: let
          pkgs = jupyenv.inputs.nixpkgs.legacyPackages.${system};
          jupyterlab = export:
            jupyenv.lib.${system}.mkJupyterlabNew ({...}: {
              imports = [
                {
                  kernel.r.experiment = {
                    runtimePackages = nixpkgs.lib.optionals export (with pkgs; [
                      texlive.combined.scheme-full
                      pgf3
                    ]);
                    enable = true;
                    name = "faas_fog";
                    displayName = "faas_fog";
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

                          # gifski
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
        in {
          apps.jupyterlabExport = {
            program = "${jupyterlab true}/bin/jupyter-lab";
            type = "app";
          };
          apps.jupyterlab = {
            program = "${jupyterlab false}/bin/jupyter-lab";
            type = "app";
          };
        }))
      ];
}

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
            overlay = self: super: {
              experiments = super.python311.withPackages (ps: (with ps; [
                dill
                click
                cryptography
                aiohttp
                influxdb-client
                simpy
                scipy
                marshmallow-dataclass
               ( alive-progress.overridePythonAttrs
                (
                  old: {
                    postInstall = ''
                      rm $out/LICENSE
                    '';
                  }
                ))
                (buildPythonPackage rec {
                  pname = "randomname";
                  version = "0.2.1";

                  src = super.fetchPypi rec {
                    inherit pname version;
                    hash = "sha256-t5uYMCukR5FksKT4eZW3vrvR2RASrtpIM0Hj5YrOUg4=";
                  };

                  doCheck = false;
                  doInstallCheck = false;
                })
              ]) ++ [inputs.nur-kapack.packages.${system}.enoslib]);
            };
          in {
            packages.experiments = pkgs.experiments;
            devShells.testbed = pkgs.experiments.env.overrideAttrs (_oldAttrs: {
              shellHook =
                outputs.checks.${system}.pre-commit-check.shellHook
                + ''
                  ln -sfT ${pkgs.experiments} ./.venv
                '';
              # Fixes https://github.com/python-poetry/poetry/issues/1917 (collection failed to unlock)
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
              ];
            });
          }
        ))
        (flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (system: let
          pkgs = nixpkgs.legacyPackages.${system};

          dockerImage = pkgs.dockerTools.buildImage {
            name = "enos_deployment";
            tag = "latest";
            copyToRoot = pkgs.buildEnv {
              name = "image-root";
              pathsToLink = [
                "/bin"
                "/"
              ];
              paths =
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
            };
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

            config = {
              Env = ["RUN=python" "HOME=/root"];
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
            touch $mountPoint/${outputs.packages.${pkgs.system}.docker}
            mount --bind -o ro ${outputs.packages.${pkgs.system}.docker} $mountPoint/${outputs.packages.${pkgs.system}.docker}
          '';
          inVMScript = ''
            set -ex
            podman load -i ${outputs.packages.${pkgs.system}.docker}
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
                    # ++ [pkgs.toysbox]
                    # ++ outputs.devShells.${system}.testbed.buildInputs
                    # ++ outputs.devShells.${system}.testbed.nativeBuildInputs
                    # ++ outputs.devShells.${system}.testbed.propagatedBuildInputs;
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

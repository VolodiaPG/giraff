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
            pkgs = import poetry2nix.inputs.nixpkgs {
              inherit system;
              overlays = [overlay];
            };

            overlay = self: _super: {
              experiments = self.poetry2nix.mkPoetryEnv {
                projectDir = ./.;
                python = self.python311;
                overrides = self.poetry2nix.overrides.withDefaults (_newattr: oldattr: {
                  # Fixes Enoslib
                  jsonschema-specifications =
                    oldattr.jsonschema-specifications.overridePythonAttrs
                    (
                      old: {
                        postPatch = ''
                          sed -i "/Topic/d" pyproject.toml
                        '';
                        buildInputs = (old.buildInputs or []) ++ [oldattr.hatch-vcs];
                      }
                    );
                  jsonschema =
                    oldattr.jsonschema.overridePythonAttrs
                    (
                      _old: {
                        postPatch = ''
                          sed -i "/Topic/d" pyproject.toml
                        '';
                      }
                    );
                  overrides =
                    oldattr.overrides.overridePythonAttrs
                    (
                      old: {
                        buildInputs = (old.buildInputs or []) ++ [oldattr.setuptools];
                      }
                    );
                  randomname =
                    oldattr.randomname.overridePythonAttrs
                    (
                      old: {
                        buildInputs = (old.buildInputs or []) ++ [oldattr.setuptools];
                      }
                    );
                  pyzmq =
                    oldattr.pyzmq.overridePythonAttrs
                    (
                      old: {
                        buildInputs = (old.buildInputs or []) ++ [oldattr.hatchling];
                      }
                    );
                  referencing =
                    oldattr.referencing.overridePythonAttrs
                    (
                      old: {
                        postPatch = ''
                          sed -i "/Topic/d" pyproject.toml
                        '';
                        buildInputs = (old.buildInputs or []) ++ [oldattr.hatch-vcs];
                      }
                    );
                  rpds-py =
                    oldattr.rpds-py.overridePythonAttrs
                    (
                      old: rec {
                        version = "0.9.2";
                        src = pkgs.fetchgit {
                          url = "https://github.com/crate-py/rpds";
                          rev = "v${version}";
                          hash = "sha256-RV4voOdWGSr4jtvU19Sfo/j0/DjO42FS70cZUwyIZrA=";
                        };

                        cargoDeps = pkgs.rustPlatform.fetchCargoTarball {
                          inherit src;
                          name = "rpds-py-${version}";
                          hash = "sha256-jpeRHKFuzJg2Gngt266cD9SmLnRLMhaX0jDAUaWNX2w=";
                        };

                        buildInputs =
                          (old.buildInputs or [])
                          ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
                            pkgs.libiconv
                          ];

                        nativeBuildInputs =
                          (old.nativeBuildInputs or [])
                          ++ [
                            pkgs.rustPlatform.cargoSetupHook
                            pkgs.rustPlatform.maturinBuildHook
                          ];
                      }
                    );
                  # Fixes alive-progress
                  about-time =
                    oldattr.about-time.overridePythonAttrs
                    (
                      old: {
                        buildInputs = (old.buildInputs or []) ++ [oldattr.setuptools];
                        postInstall = ''
                          rm $out/LICENSE
                        '';
                      }
                    );
                  alive-progress =
                    oldattr.alive-progress.overridePythonAttrs
                    (
                      old: {
                        buildInputs = (old.buildInputs or []) ++ [oldattr.setuptools];
                        postInstall = ''
                          rm $out/LICENSE
                        '';
                      }
                    );
                });
              };
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
          pkgs = import nixpkgs {
            inherit system;
          };
          jupyterlab = export:
            jupyenv.lib.${system}.mkJupyterlabNew ({...}: {
              imports = [
                {
                  kernel.r.experiment = {
                    runtimePackages =
                      nixpkgs.lib.optionals export (with pkgs; [
                        texlive.combined.scheme-full
                        pgf3
                      ])
                      ++ [pkgs.toybox]
                      ++ outputs.devShells.${system}.testbed.buildInputs
                      ++ outputs.devShells.${system}.testbed.nativeBuildInputs
                      ++ outputs.devShells.${system}.testbed.propagatedBuildInputs;
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

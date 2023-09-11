{
  description = "Application packaged using poetry2nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    poetry2nix.url = "github:nix-community/poetry2nix";
    poetry2nix.inputs.nixpkgs.follows = "nixpkgs";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    pre-commit-hooks.inputs.nixpkgs.follows = "nixpkgs";
    pre-commit-hooks.inputs.nixpkgs-stable.follows = "nixpkgs";
    pre-commit-hooks.inputs.flake-utils.follows = "flake-utils";
    # jupyenv.url = "github:tweag/jupyenv";
    jupyenv.url = "github:dialohq/jupyenv";
    jupyenv.inputs.nixpkgs.follows = "nixpkgs";

    impermanence.url = "github:nix-community/impermanence";
  };

  nixConfig = {
    extra-trusted-substituters = "https://nix-community.cachix.org";
    extra-trusted-public-keys = "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs=";
  };

  outputs = inputs:
    with inputs; let
      inherit (self) outputs;
      # Load the iso flake
      isoFlake = import ./iso/flake.nix;
      isoOutputs = isoFlake.outputs {
        inherit nixpkgs;
        inherit flake-utils;
      };
    in
      nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate {}
      [
        (flake-utils.lib.eachDefaultSystem (
          system: let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [overlay];
            };
            # see https://github.com/nix-community/poetry2nix/tree/master#api for more functions and examples.
            inherit (poetry2nix.legacyPackages.${system}) mkPoetryEnv;

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
            checks = {
              pre-commit-check = pre-commit-hooks.lib.${system}.run {
                src = ./.;
                settings.statix.ignore = ["Cargo.nix"];
                settings.mypy.binPath = "${pkgs.mypy}/bin/mypy --no-namespace-packages";
                hooks = {
                  # Nix
                  alejandra.enable = true;
                  statix.enable = true;
                  deadnix = {
                    enable = true;
                    excludes = ["Cargo.nix"];
                  };
                  # Python
                  autoflake.enable = true;
                  isort.enable = true;
                  ruff.enable = true;
                  mypy.enable = true;
                  # Shell scripting
                  shfmt.enable = true;
                  shellcheck.enable = true;
                  bats.enable = true;
                  # Git (conventional commits)
                  commitizen.enable = true;
                };
              };
            };
            devShells.default = pkgs.mkShell {
              shellHook =
                self.checks.${system}.pre-commit-check.shellHook
                + ''
                  ln -sfT ${pkgs.experiments} ./.venv
                '';
              # Fixes https://github.com/python-poetry/poetry/issues/1917 (collection failed to unlock)
              PYTHON_KEYRING_BACKEND = "keyring.backends.null.Keyring";
              packages = with pkgs; [
                just
                jq
                experiments
                ruff
                black
                isort
                mypy
                mprocs
                parallel
                bashInteractive
                bc
                tmux
              ];
            };
            formatter = pkgs.alejandra;
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

                  # openvpn # to connect to the inside of g5k
                  # openresolv
                  # update-resolv-conf
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

              mkdir /tmp
              mkdir -p /usr/bin
              ln -s ${pkgs.busybox}/bin/env /usr/bin/env

              mkdir -p /etc/openvpn
              ln -s ${pkgs.update-resolv-conf}/libexec/openvpn/* /etc/openvpn
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
                outputs.packages.${pkgs.system}.enosDeployment.buildInputs
                ++ stdenv.initialPath
              );
          in {
            virtualisation.podman.enable = true;

            systemd.services.experiment = {
              description = "Start the experiment";
              after = ["network-online.target"];
              wantedBy = ["multi-user.target"];
              script = ''
                #!${pkgs.bash}/bin/bash
                set -ex
                export PATH=${binPath}:$PATH
                mkdir -p /home/enos
                ln -s ${outputs.packages.${pkgs.system}.enosDeployment}/* /home/enos
                ln -s ${outputs.packages.${pkgs.system}.enosDeployment}/.env /home/enos
                ln -s ${outputs.packages.${pkgs.system}.enosDeployment}/.experiments.env /home/enos
                touch /home/enos/env.source
                echo 'export PATH=${binPath}:$PATH' | tee /home/enos/env.source
              '';
              serviceConfig = {
                Type = "oneshot";
                RemainAfterExit = "yes";
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
          devShells.enosvm = isoOutputs.devShells.${system}.default;
          packages.enosDeployment = pkgs.stdenv.mkDerivation {
            src = ./.;
            name = "enosDeployment";
            inherit (outputs.devShells.${pkgs.system}.default) nativeBuildInputs propagatedBuildInputs;
            buildInputs =
              [
                outputs.packages.${pkgs.system}.experiments
              ]
              ++ outputs.devShells.${pkgs.system}.default.buildInputs
              ++ outputs.devShells.${pkgs.system}.default.nativeBuildInputs
              ++ outputs.devShells.${pkgs.system}.default.propagatedBuildInputs;
            unpackPhase = ''
              mkdir -p $out
              cp $src/*.py $out
              cp $src/.env $out
              cp $src/.experiments.env $out
              cp $src/justfile $out
            '';
          };
        }))
        (flake-utils.lib.eachDefaultSystem (system: let
          pkgs = nixpkgs.legacyPackages.${system};
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
                      ++ outputs.devShells.${system}.default.buildInputs
                      ++ outputs.devShells.${system}.default.nativeBuildInputs
                      ++ outputs.devShells.${system}.default.propagatedBuildInputs;
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

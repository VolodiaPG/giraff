{
  description = "Application packaged using poetry2nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    # nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    poetry2nix.url = "github:nix-community/poetry2nix";
    poetry2nix.inputs.nixpkgs.follows = "nixpkgs";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    pre-commit-hooks.inputs.nixpkgs.follows = "nixpkgs";
    pre-commit-hooks.inputs.nixpkgs-stable.follows = "nixpkgs";
    pre-commit-hooks.inputs.flake-utils.follows = "flake-utils";
    alejandra.url = "github:kamadorueda/alejandra/3.0.0";
    alejandra.inputs.nixpkgs.follows = "nixpkgs";
    jupyenv.url = "github:tweag/jupyenv";
    jupyenv.inputs.nixpkgs.follows = "nixpkgs";
  };

  nixConfig = {
    extra-trusted-substituters = "https://nix-community.cachix.org";
    extra-trusted-public-keys = "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs=";
  };

  outputs = inputs:
    with inputs; let
      inherit (self) outputs;
    in
      nixpkgs.lib.recursiveUpdate
      (flake-utils.lib.eachDefaultSystem (system: let
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
              beautifulsoup4 =
                oldattr.beautifulsoup4.overridePythonAttrs
                (
                  old: {
                    buildInputs = (old.buildInputs or []) ++ [oldattr.hatchling];
                  }
                );
              urllib3 =
                oldattr.urllib3.overridePythonAttrs
                (
                  old: {
                    buildInputs = (old.buildInputs or []) ++ [oldattr.hatchling];
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
                    version = "0.8.10";
                    src = pkgs.fetchgit {
                      url = "https://github.com/crate-py/rpds";
                      rev = "v${version}";
                      hash = "sha256-DJPYxJ1gJFmRy+a8KmR1H6tFHKTyd0D5PDD30iH7z1g=";
                    };

                    cargoDeps = pkgs.rustPlatform.fetchCargoTarball {
                      inherit src;
                      name = "rpds-py-${version}";
                      hash = "sha256-NhvajV9s8w233XP//KZosjEzar8YOQmKZR/zV5GVU0k=";
                    };
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

        pkgs = import nixpkgs {
          inherit system;
          overlays = [overlay];
        };
      in {
        packages.experiments = pkgs.experiments;
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            settings.statix.ignore = ["Cargo.nix"];
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
              # Shell scripting
              shfmt.enable = true;
              shellcheck.enable = true;
              bats.enable = true;
              # Git (conventional commits)
              commitizen.enable = true;
            };
          };
        };
        devShells.default = let 
        venv = outputs.packages.${system}.experiments;
        in pkgs.mkShell {
          shellHook = self.checks.${system}.pre-commit-check.shellHook + ''
            ln -sfT ${venv} ./.venv
          '';
          # Fixes https://github.com/python-poetry/poetry/issues/1917 (collection failed to unlock)
          PYTHON_KEYRING_BACKEND = "keyring.backends.null.Keyring";
          packages =
            [poetry2nix.packages.${system}.poetry]
            ++ (with pkgs; [
              just
              jq
              experiments
              poetry
              ruff
              black
              mypy
              mprocs
              parallel
              bashInteractive
            ]);
        };
      }))
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

                # My toolset
                just
                jq
                openssh
                curl

                openvpn # to connect to the inside of g5k
                openresolv
                update-resolv-conf
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
      // flake-utils.lib.eachDefaultSystem (system: let
        pkgs = nixpkgs.legacyPackages.${system};

        inherit (pkgs) lib;

        inherit (jupyenv.lib.${system}) mkJupyterlabNew;
        jupyterlab = export:
          mkJupyterlabNew ({...}: {
            inherit (inputs) nixpkgs;

            imports = [
              {
                kernel.r.experiment = {
                  runtimePackages =
                    nixpkgs.lib.optionals export (with pkgs; [
                      texlive.combined.scheme-full
                      pgf3
                    ])
                    ++ [pkgs.busybox]
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

                        doParallel
                        foreach

                        gifski
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
                      ++ nixpkgs.lib.optional export tikzDevice;
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
      })
      // {
        formatter = alejandra.defaultPackage;
      };
}

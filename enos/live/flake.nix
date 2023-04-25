{
  description = "Application packaged using poetry2nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    poetry2nix = {
      url = "github:nix-community/poetry2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-stable.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    alejandra = {
      url = "github:kamadorueda/alejandra/3.0.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    jupyenv = {
      url = "github:tweag/jupyenv";
      inputs.nixpkgs.follows = "nixpkgs-unstable";
    };
  };

  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem (system: let
        # see https://github.com/nix-community/poetry2nix/tree/master#api for more functions and examples.
        inherit (poetry2nix.legacyPackages.${system}) mkPoetryEnv;

        overlay = self: super: {
          experiments = self.poetry2nix.mkPoetryEnv {
            projectDir = ./.;
            python = self.python311;
            overrides = self.poetry2nix.overrides.withDefaults (_newattr: oldattr: {
              # Fixes Enoslib
              cryptography = oldattr.cryptography.overridePythonAttrs (
                old: {
                  cargoDeps = super.rustPlatform.fetchCargoTarball {
                    src = old.src;
                    sourceRoot = "${old.pname}-${old.version}/src/rust";
                    name = "${old.pname}-${old.version}";
                    sha256 = "sha256-Admz48/GS2t8diz611Ciin1HKQEyMDEwHxTpJ5tZ1ZA=";
                  };
                }
              );
              rfc3986-validator =
                oldattr.rfc3986-validator.overridePythonAttrs
                (
                  old: {
                    buildInputs = (old.buildInputs or []) ++ [oldattr.setuptools oldattr.setuptools-scm oldattr.pytest-runner];
                  }
                );
              pathspec =
                oldattr.pathspec.overridePythonAttrs
                (
                  old: {
                    buildInputs = (old.buildInputs or []) ++ [oldattr.flit-scm oldattr.pytest-runner];
                  }
                );
              ncclient =
                oldattr.ncclient.overridePythonAttrs
                (
                  old: {
                    buildInputs = (old.buildInputs or []) ++ [oldattr.six];
                  }
                );
              jupyter-server-terminals =
                oldattr.jupyter-server-terminals.overridePythonAttrs
                (
                  old: {
                    buildInputs = (old.buildInputs or []) ++ [oldattr.hatchling];
                  }
                );
              jupyter-events =
                oldattr.jupyter-events.overridePythonAttrs
                (
                  old: {
                    buildInputs = (old.buildInputs or []) ++ [oldattr.hatchling];
                  }
                );
              jupyter-server =
                oldattr.jupyter-server.overridePythonAttrs
                (
                  old: {
                    buildInputs = (old.buildInputs or []) ++ [oldattr.hatch-jupyter-builder oldattr.hatchling];
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
        lib = nixpkgs.lib;

        dockerImage = pkgs.dockerTools.buildImage {
          name = "enos_deployment";
          tag = "latest";
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            pathsToLink = [
              "/bin"
              "/"
            ];
            paths = with pkgs; [
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
              update-resolv-conf

              # Environment to run enos and stuff
              experiments
            ];
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
            ln -s ${pkgs.update-resolv-conf}/libexec/openvpn/update-resolv-conf /etc/openvpn/update-resolv-conf
          '';
          config = {
            Env = ["RUN=python" "HOME=/root"];
          };
        };

        # superchargedNixpkgs = nixpkgs-unstable.lib.extendDerivation nixpkgs-unstable.legacyPackages.${nixpkgs-unstable.system} (final: prev: {
        #   texlive.combined.scheme-medium = prev.texlive.combined.scheme-medium.overrideAttrs (oldAttrs: {
        #     buildInputs = (oldAttrs.buildInputs or []) ++ [prev.pgf3];
        #   });
        # });
        inherit (jupyenv.lib.${system}) mkJupyterlabNew;
        jupyterlab = mkJupyterlabNew ({...}: {
          nixpkgs = inputs.nixpkgs;
          # nixpkgs = superchargedNixpkgs;
          imports = [
            {
              kernel.r.experiment = {
                runtimePackages = with pkgs; [
                  texlive.combined.scheme-full
                  pgf3
                ];
                enable = true;
                name = "faas_fog";
                displayName = "faas_fog";
                extraRPackages = ps:
                  with ps; [
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
                    tikzDevice
                  ];
              };
            }
          ];
        });
      in {
        packages = {
          docker = dockerImage;
          default = pkgs.experiments;
        };
        apps.jupyterlab = {
          program = "${jupyterlab}/bin/jupyter-lab";
          type = "app";
        };
        formatter = alejandra.defaultPackage.${system};
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

        devShells.default = pkgs.mkShell {
          inherit (self.checks.${system}.pre-commit-check) shellHook;
          packages =
            [poetry2nix.packages.${system}.poetry]
            ++ (with pkgs; [
              just
              jq
              experiments
              poetry
              ruff
            ]);
        };
      });
}

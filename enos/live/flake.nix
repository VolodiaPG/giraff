{
  description = "Application packaged using poetry2nix";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.poetry2nix = {
    url = "github:nix-community/poetry2nix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, poetry2nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # see https://github.com/nix-community/poetry2nix/tree/master#api for more functions and examples.
        inherit (poetry2nix.legacyPackages.${system}) mkPoetryEnv mkPoetryApplication;
        pkgs = nixpkgs.legacyPackages.${system};
        lib = nixpkgs.lib;

        overlay = self: super:{
          experiments = self.poetry2nix.mkPoetryEnv {
            projectDir = ./.;
            python = self.python3;
            overrides = pkgs.poetry2nix.overrides.withDefaults (self: super: {
                cryptography = super.cryptography.overridePythonAttrs (
                    old: {
                      cargoDeps =
                        pkgs.rustPlatform.fetchCargoTarball {
                          src = old.src;
                          sourceRoot = "${old.pname}-${old.version}/src/rust";
                          name = "${old.pname}-${old.version}";
                          sha256 = "sha256-clorC0NtGukpE3DnZ84MSdGhJN+qC89DZPITZFuL01Q=";
                        };
                    }
                  );
                rfc3986-validator = super.rfc3986-validator.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or [ ]) ++ [ super.setuptools self.setuptools-scm self.pytest-runner ];
                    }
                  );
                pathspec = super.pathspec.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or [ ]) ++ [ self.flit-scm self.pytest-runner ];
                    }
                  );
                ncclient = super.ncclient.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or [ ]) ++ [ self.six ];
                    }
                  );
                jupyter-server-terminals = super.jupyter-server-terminals.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or [ ]) ++ [ self.hatchling ];
                    }
                  );
                jupyter-events = super.jupyter-events.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or [ ]) ++ [ self.hatchling ];
                    }
                  );
                jupyter-server = super.jupyter-server.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or [ ]) ++ [ self.hatch-jupyter-builder self.hatchling ];
                    }
                  );
              });
          };
        };

        linuxPackages = import nixpkgs {
          inherit system;
          overlays = [overlay];
        };

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
                    coreutils
                    gnused
                    bashInteractive

                    # My toolset
                    just
                    jq
                    openssh
                    curl
                    frp

                    # Environment to run enos and stuff
                    linuxPackages.experiments
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
              ln -s ${pkgs.coreutils}/bin/env /usr/bin/env
            '';
          config = {
            Env = [ "RUN=python" "HOME=/root"]; 
          };
        };
      in
      {
        packages = {
          docker = dockerImage;
          default = self.packages.${system}.myapp;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [ 
            just
            jq
            poetry2nix.packages.${system}.poetry
          ];
        };
      });
}

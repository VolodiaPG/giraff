{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    srvos = {
      url = "github:nix-community/srvos";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        nixpkgs-stable.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    nur-kapack = {
      url = "github:oar-team/nur-kapack";
      inputs = {
        flake-utils.follows = "flake-utils";
      };
    };
    jupyenv = {
      url = "github:dialohq/jupyenv"; #"github:tweag/jupyenv";
    };
    gomod2nix = {
      url = "github:nix-community/gomod2nix";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
    impermanence.url = "github:nix-community/impermanence";
    kubenix = {
      url = "github:hall/kubenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fwatchdog = {
      url = "github:openfaas/of-watchdog";
      flake = false;
    };
  };

  nixConfig = {
    extra-substituters = ["https://giraff.cachix.org"];
    extra-trusted-public-keys = ["giraff.cachix.org-1:3sol29PSsWCh/7bAiRze+5Zq6OML02FDRH13K5i3qF4="];
  };

  outputs = inputs:
    with inputs; let
      inherit (self) outputs;
      inherit (nixpkgs) lib;

      extra = {
        buildRustEnv = import ./rust.nix {inherit inputs;};
      };

      subflake = path:
        (import path).outputs inputs extra;
    in
      lib.foldl lib.recursiveUpdate {}
      [
        (subflake ./testbed/subflake.nix)
        (subflake ./manager/subflake.nix)
        (subflake ./iot_emulation/subflake.nix)
        (subflake ./openfaas-functions/subflake.nix)
        (flake-utils.lib.eachDefaultSystem (
          system: let
            pkgs = import nixpkgs {
              inherit system;
            };
          in {
            apps.cachix = let
              binPath = with pkgs;
                lib.strings.makeBinPath (
                  [
                    nix
                    parallel
                    cachix
                    toybox
                  ]
                  ++ stdenv.initialPath
                );

              trace_pkgs = builtins.concatStringsSep " " (
                pkgs.lib.mapAttrsToList
                (name: output:
                  if (lib.strings.hasSuffix "vm" name) || (lib.strings.hasPrefix "fog_node" name) || (lib.strings.hasPrefix "market" name)
                  then ""
                  else "${output}")
                outputs.packages.${system}
              );
              trace_apps = builtins.concatStringsSep " " (
                pkgs.lib.mapAttrsToList
                (name: output:
                  if (lib.strings.hasInfix "cachix" name) || (lib.strings.hasSuffix "Export" name)
                  then ""
                  else "${output.program}")
                outputs.apps.${system}
              );

              script = pkgs.writeShellScript "cachix" ''
                set -e
                export PATH=${binPath}:$PATH
                tmp=$(mktemp)
                parallel nix path-info --derivation {} '>>' $tmp ::: ${trace_apps} ${trace_pkgs}
                parallel nix-store --query --requisites --include-outputs '$(' nix path-info --derivation {} ') >>' $tmp ::: ${trace_apps} ${trace_pkgs}
                cachix push -j $(nproc --all) -m xz -c 9 giraff $(cat $tmp | tr "\n" " ")
                rm $tmp
              '';
            in {
              type = "app";
              program = "${script}";
            };
            devShells.default = pkgs.mkShell {
              inherit
                (outputs.checks.${system}.pre-commit-check)
                shellHook
                ;
              packages = with pkgs; [just cachix];
            };
            formatter = pkgs.alejandra;
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
                  # Git (conventional commits)
                  commitizen.enable = true;
                  # Python
                  autoflake.enable = true;
                  isort.enable = true;
                  ruff.enable = true;
                  mypy.enable = true;
                  zCachix = {
                    enable = true;
                    name = "push cachix";
                    entry = "sh -c 'nix run .#cachix'";
                    language = "system";
                    pass_filenames = false;
                  };
                  # manager
                  # manager = {
                  #   enable = true;
                  #   name = "rust (manager)";
                  #   entry = "sh -c 'cd `git rev-parse --show-toplevel`/manager; nix develop -c .#manager just pre_commit'";
                  #   language = "system";
                  #   pass_filenames = false;
                  # };
                  # iot_emulation = {
                  #   enable = true;
                  #   name = "rust (iot_emulation)";
                  #   entry = "sh -c 'cd iot_emulation; nix develop -c .#iot_emulation just pre_commit'";
                  #   language = "system";
                  #   pass_filenames = false;
                  # };
                  # # functions
                  # rustEcho = {
                  #   enable = true;
                  #   name = "rust (OpenFaaS functions)";
                  #   entry = "sh -c 'cd openfaas-functions && nix develop .#openfaas_functions just pre_commit'";
                  #   language = "system";
                  #   pass_filenames = false;
                  # };
                  # Go
                  gofmt.enable = true;
                  revive.enable = true;
                  # staticcheck.enable = true;
                };
              };
            };
          }
        ))
      ];
}

{
  inputs = {
    # Al packages
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # Rust
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Rust tooling
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Linux server general optimizations
    srvos = {
      url = "github:nix-community/srvos";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Made to run at each commit and check
    pre-commit-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        nixpkgs-stable.follows = "nixpkgs";
      };
    };
    disko = {
      #url = "github:jmbaur/disko/self-contained-deps";
      url = "github:nix-community/disko"; # for disk generation
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Go
    gomod2nix = {
      url = "github:nix-community/gomod2nix";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
    # Provides linux on temporary filesystems (wiped at reboot)
    impermanence.url = "github:nix-community/impermanence";
    # Kubernetes config file definitions strictly defined
    kubenix = {
      url = "github:hall/kubenix?ref=refs/tags/0.2.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # OpenFaaS function software running in the container (like a proxy)
    fwatchdog = {
      url = "github:openfaas/of-watchdog";
      flake = false;
    };
    enoslib = {
      url = "git+https://gitlab.inria.fr/discovery/enoslib";
      flake = false;
    };
    openfaas = {
      # Since > v0.17.2 does a check at startup on checkip.amazonaws.com to start
      # It seems OFaas sort of just not works without internet, more than 15 functions, more that 60 days... so for now let's stay on 17x
      url = "github:openfaas/faas-netes?ref=refs/tags/0.17.2";
      flake = false;
    };
    ebpf-netem.url = "github:volodiapg/ebpf-netem/loss";
  };

  # Enable caching
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
        shellHook = system: shellName: let
          pkgs = nixpkgs.legacyPackages.${system};
          # packages is used as bash list in the following script
          # deadnix: skip
          packages =
            outputs.devShells.${system}.${shellName}.buildInputs
            ++ outputs.devShells.${system}.${shellName}.nativeBuildInputs
            ++ outputs.devShells.${system}.${shellName}.propagatedBuildInputs;
        in ''
          # Add additional folders to to XDG_DATA_DIRS if they exists, which will get sourced by bash-completion
          for p in ''${packages}; do
            if [ -d "$p/share/bash-completion" ]; then
              XDG_DATA_DIRS="$XDG_DATA_DIRS:$p/share"
            fi
          done

          source ${pkgs.bash-completion}/etc/profile.d/bash_completion.sh
          ${outputs.checks.${system}.pre-commit-check.shellHook}
        '';
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
                tmp2=$(mktemp)
                parallel "(nix  --extra-experimental-features 'nix-command flakes' path-info --json --derivation {} | jq -r 'map(.path)' | jq -r '.[]' | grep '/nix/store') >> $tmp" ::: ${trace_apps} ${trace_pkgs}
                parallel -N1000 -a $tmp "(nix-store --query --requisites --include-outputs {} | grep '/nix/store') >> $tmp2"
                cat $tmp2 >> $tmp
                parallel -N1000 -a $tmp cachix push -j $(nproc --all) -m xz -c 9 giraff {}
                rm $tmp
                rm $tmp2
              '';
            in {
              type = "app";
              program = "${script}";
            };
            devShells.default = pkgs.mkShell {
              shellHook = (extra.shellHook system) "default";
              packages = with pkgs; [just cachix go];
            };
            formatter = pkgs.alejandra;
            checks = {
              pre-commit-check = pre-commit-hooks.lib.${system}.run {
                src = ./.;
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
                  pyright.enable = true;
                  # Update the cache at each commit
                  zCachix = {
                    enable = false;
                    name = "push cachix";
                    entry = ''
                      sh -c 'nix run --extra-experimental-features "nix-command flakes" .#cachix'
                    '';
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
                  staticcheck.enable = true;
                };
              };
            };
          }
        ))
      ];
}

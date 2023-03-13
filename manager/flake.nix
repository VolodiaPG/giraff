{
  inputs = {
    nixpkgs.url = "github:Nixos/nixpkgs/nixos-22.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix/release-0.11.0";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
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
  };

  outputs = inputs:
    with inputs;
      flake-utils.lib.eachSystem ["x86_64-linux"] (
        # flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            config.allowUnfree = true;
            overlays = [cargo2nix.overlays.default];
          };

          # Define Rust environment to use
          rustChannel = "nightly";
          rustProfile = "minimal";
          rustVersion = "2023-02-26";
          target = "x86_64-unknown-linux-gnu";
          extraRustComponents = ["clippy" "rustfmt"];

          #Packages
          rustPkgs = pkgs.rustBuilder.makePackageSet {
            inherit rustChannel rustProfile target rustVersion extraRustComponents;
            packageFun = import ./Cargo.nix;
            rootFeatures = [];
          };

          # Generators
          pkgsGenerator = {rootFeatures ? []}:
            pkgs.rustBuilder.makePackageSet {
              inherit rustChannel rustProfile target rustVersion rootFeatures extraRustComponents;
              packageFun = import ./Cargo.nix;
            };

          dockerImageFogNodeGenerator = {
            tag,
            rootFeatures,
          }:
            pkgs.dockerTools.buildLayeredImage {
              inherit tag;
              name = "fog_node";
              config = {
                Cmd = ["${((pkgsGenerator {inherit rootFeatures;}).workspace.fog_node {}).bin}/bin/fog_node"];
              };
            };

          dockerImageMarket = pkgs.dockerTools.buildLayeredImage {
            name = "market";
            tag = "latest";
            config = {
              Env = ["SERVER_PORT=3003"];
              Cmd = ["${(rustPkgs.workspace.market {}).bin}/bin/market"];
            };
          };
        in rec {
          packages =
            {market = dockerImageMarket;}
            // builtins.listToAttrs
            (
              builtins.map
              (
                settings: let
                  tag = "${settings.strategy}_${settings.telemetry}";
                in {
                  name = "fog_node_${tag}";
                  value = dockerImageFogNodeGenerator {
                    inherit tag;
                    rootFeatures =
                      []
                      ++ nixpkgs.lib.optional (settings.strategy != "auction") settings.strategy
                      ++ nixpkgs.lib.optional (settings.telemetry != "no-telemetry") settings.telemetry;
                  };
                }
              )
              (
                nixpkgs.lib.attrsets.cartesianProductOfSets
                {
                  strategy = ["auction" "edge_first" "edge_ward" "cloud_only"];
                  telemetry = ["no-telemetry" "jaeger"];
                }
              )
            );
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
                # Rust
                rust = {
                  enable = true;
                  name = "rust (justfile pre_commit)";
                  entry = "sh -c 'cd manager && just pre_commit'";
                  language = "system";
                  pass_filenames = false;
                };
                # Git (conventional commits)
                commitizen.enable = true;
              };
            };
          };
          devShell = pkgs.mkShell {
            default = rustPkgs.workspaceShell {
              inherit (self.checks.${system}.pre-commit-check) shellHook;
              packages = with pkgs; [
                docker
                just
                pkg-config
                jq
                openssl
                rust-analyzer
                cargo-outdated
                cargo-udeps
                # cargo-watch
                lldb
                kubectl
                (rustfmt.override {asNightly = true;})
                cargo2nix.packages.${system}.cargo2nix
              ];
            };
          };
        }
      );
}

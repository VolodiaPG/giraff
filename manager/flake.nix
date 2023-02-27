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
      flake-utils.lib.eachDefaultSystem (
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

          dockerImageFogNodeGenerator = {feature ? null}: let
            tag =
              if feature != null
              then feature
              else "default";
            rootFeatures =
              if feature != null
              then [feature]
              else [];
          in
            pkgs.dockerTools.buildImage {
              inherit tag;
              name = "fog_node";
              config = {
                Cmd = ["${((pkgsGenerator {inherit rootFeatures;}).workspace.fog_node {}).bin}/bin/fog_node"];
              };
            };

          dockerImageFogNodeAuction = dockerImageFogNodeGenerator {};
          dockerImageFogNodeEdgeFirst = dockerImageFogNodeGenerator {feature = "edge_first";};
          dockerImageFogNodeEdgeWard = dockerImageFogNodeGenerator {feature = "edge_ward";};

          dockerImageMarket = pkgs.dockerTools.buildImage {
            name = "market";
            tag = "latest";
            config = {
              Env = ["SERVER_PORT=3003"];
              Cmd = ["${(rustPkgs.workspace.market {}).bin}/bin/market"];
            };
          };
        in rec {
          packages = {
            market = dockerImageMarket;

            fog_node.auction = dockerImageFogNodeAuction;
            fog_node.edge_first = dockerImageFogNodeEdgeFirst;
            fog_node.edge_ward = dockerImageFogNodeEdgeWard;
          };
          formatter = alejandra.defaultPackage.${system};
          checks = {
            pre-commit-check = pre-commit-hooks.lib.${system}.run {
              src = ./.;
              settings.rust.cargoManifestPath = "./manager/Cargo.toml";
              settings.statix.ignore = ["Cargo.nix"];
              hooks = {
                # Nix
                alejandra.enable = true;
                statix = {
                  enable = true;
                  # ignore = ["Cargo.nix"];
                };
                deadnix = {
                  enable = true;
                  excludes = ["Cargo.nix"];
                };
                # Rust
                rust = {
                  enable = true;

                  # The name of the hook (appears on the report table):
                  name = "Rust checks (using justfile)";

                  # The command to execute (mandatory):
                  entry = "sh -c 'cd manager && just pre_commit'";

                  # # The pattern of files to run on (default: "" (all))
                  # # see also https://pre-commit.com/#hooks-files
                  # files = "\\.(c|h)$";

                  # # List of file types to run on (default: [ "file" ] (all files))
                  # # see also https://pre-commit.com/#filtering-files-with-types
                  # # You probably only need to specify one of `files` or `types`:
                  # types = ["text" "c"];

                  # # Exclude files that were matched by these patterns (default: [ ] (none)):
                  # excludes = ["irrelevant\\.c"];

                  # The language of the hook - tells pre-commit
                  # how to install the hook (default: "system")
                  # see also https://pre-commit.com/#supported-languages
                  language = "system";

                  # Set this to false to not pass the changed files
                  # to the command (default: true):
                  pass_filenames = false;
                };
                # Rust
                # rustfmt = {
                #   enable = true;
                #   # entry = ${pkgs.rustfmt.override {asNightly = true;}}
                # };
                # clippy.enable = true;
                # # cargo-check.enable = true;
              };
            };
          };
          devShells = pkgs.mkShell {
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
                lldb
                kubectl
                # clippy
                (rustfmt.override {asNightly = true;})
                cargo2nix.packages.${system}.cargo2nix
              ];
            };
          };
        }
      );
}

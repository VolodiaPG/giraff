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
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
          overlays = [ cargo2nix.overlays.default ];
        };

        # Define Rust environment to use
        rustChannel = "nightly";
        rustProfile = "minimal";
        rustVersion = "2023-02-26";
        target = "x86_64-unknown-linux-gnu";

        #Packages
        rustPkgs = pkgs.rustBuilder.makePackageSet {
          inherit rustChannel rustProfile target rustVersion;
          packageFun = import ./Cargo.nix;
          rootFeatures = [ ];
        };

        # Generators
        pkgsGenerator = { rootFeatures ? [ ] }: pkgs.rustBuilder.makePackageSet {
          inherit rustChannel rustProfile target rustVersion rootFeatures;
          packageFun = import ./Cargo.nix;
        };

        dockerImageFogNodeGenerator = { feature ? null }:
          let
            tag = if feature != null then feature else "default";
            rootFeatures = if feature != null then [ feature ] else [ ];
          in
          pkgs.dockerTools.buildImage {
            inherit tag;
            name = "fog_node";
            config = {
              Cmd = [ "${((pkgsGenerator {inherit rootFeatures;}).workspace.fog_node { }).bin}/bin/fog_node" ];
            };
          };

        dockerImageFogNodeAuction = dockerImageFogNodeGenerator { };
        dockerImageFogNodeEdgeFirst = dockerImageFogNodeGenerator { feature = "edge_first"; };
        dockerImageFogNodeEdgeWard = dockerImageFogNodeGenerator { feature = "edge_ward"; };

        dockerImageMarket = pkgs.dockerTools.buildImage {
          name = "market";
          tag = "latest";
          config = {
            Env = [ "SERVER_PORT=3003" ];
            Cmd = [ "${(rustPkgs.workspace.market { }).bin}/bin/market" ];
          };
        };
      in
      rec {
        packages = {
          market = dockerImageMarket;

          fog_node.auction = dockerImageFogNodeAuction;
          fog_node.edge_first = dockerImageFogNodeEdgeFirst;
          fog_node.edge_ward = dockerImageFogNodeEdgeWard;
        };
        formatter = nixpkgs.legacyPackages.${system}.nixpkgs-fmt;
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              # Nix
              nixpkgs-fmt.enable = true;
              statix.enable = true;
              deadnix = {
                enable = true;
                excludes = [ "Cargo.nix" ];
              };
              # Rust
              rustfmt.enable = true;
              clippy.enable = true;
              cargo-check.enable = true;
            };
          };
        };
        devShells = pkgs.mkShell {
          default = (rustPkgs.workspaceShell {
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
              (rustfmt.override { asNightly = true; })
              cargo2nix.packages.${system}.cargo2nix
            ];
          });
        };
      }
    );
}
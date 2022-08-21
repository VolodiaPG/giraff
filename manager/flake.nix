{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [cargo2nix.overlays.default];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustChannel = "nightly";
          rustProfile = "minimal";
          target = "x86_64-unknown-linux-musl";
          packageFun = import ./Cargo.nix;
        };
        fog_node_bin = (rustPkgs.workspace.fog_node {}).bin;
        market_bin = (rustPkgs.workspace.market {}).bin;

        dockerImageFogNode = pkgs.dockerTools.buildImage {
          name = "nix_fog_node";
          tag = "latest";
          config.Cmd = [ "${fog_node_bin}/bin/fog_node" ];

        };

        dockerImageMarket = pkgs.dockerTools.buildImage {
          name = "nix_market";
          tag = "latest";
          config.Cmd = [ "${market_bin}/bin/market" ];
        };

      in rec {
        packages = {
          # replace hello-world with your package name
          fog_node = fog_node_bin;
          market = market_bin;
          docker_fog_node = dockerImageFogNode;
          docker_market = dockerImageMarket;

          default = packages.fog_node;
        };
        #   devShell = pkgs.mkShell {
        #   buildInputs =
        #     [ (rustVersion.override { extensions = [ "rust-src" ]; }) ];
        # };
      }
    );
}

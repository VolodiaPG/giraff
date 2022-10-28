{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
          overlays = [ cargo2nix.overlays.default ];
        };

        rustChannel = "nightly";
        rustProfile = "minimal";
        target = "x86_64-unknown-linux-musl";

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          inherit rustChannel rustProfile target;
          packageFun = import ./Cargo.nix;
          rootFeatures = [ ];
        };

        rustPkgs_naive = pkgs.rustBuilder.makePackageSet {
          inherit rustChannel rustProfile target;
          packageFun = import ./Cargo.nix;
          rootFeatures = [ "fog_node/bottom_up_placement" ];
        };

        workspaceShell = (rustPkgs.workspaceShell {
          packages = with pkgs; [
            docker
            just
            rust-analyzer
            (rustfmt.override { asNightly = true; })
            cargo2nix.packages.${system}.cargo2nix
          ];
        });


        fog_node_bin = (rustPkgs.workspace.fog_node { }).bin;
        market_bin = (rustPkgs.workspace.market { }).bin;

        fog_node_naive_bin = (rustPkgs_naive.workspace.fog_node { }).bin;

        log4rs = pkgs.writeTextDir "/log4rs.yaml" (builtins.readFile ./log4rs.yaml);

        dockerImageFogNode = pkgs.dockerTools.buildImage
          {
            name = "nix_fog_node";
            tag = "latest";
            config = {
              Cmd = [ "${fog_node_bin}/bin/fog_node" ];
              Env = [ "LOG_CONFIG_PATH=/log4rs.yaml" ];
            };
            # Now renamed to copyToRoot
            contents = pkgs.buildEnv {
              name = "log4rs.yaml";
              paths = [ log4rs ];
              pathsToLink = [ "/" ];
            };
          };

        dockerImageMarket = pkgs.dockerTools.buildImage {
          name = "nix_market";
          tag = "latest";
          runAsRoot = ''
            #!${pkgs.runtimeShell}
            mkdir -p /var/log/
            ln -sf /proc/self/fd/1 /var/log/stdout.log
          '';
          config.Cmd = [ "${market_bin}/bin/market" ];
        };

        dockerImageFogNodeNaive = pkgs.dockerTools.buildImage {
          name = "nix_fog_node";
          tag = "latest";
          runAsRoot = ''
            #!${pkgs.runtimeShell}
            mkdir -p /var/log/
            ln -sf /proc/self/fd/1 /var/log/stdout.log
          '';
          config.Cmd = [ "${fog_node_naive_bin}/bin/fog_node" ];
        };
      in
      rec {
        packages = {
          # replace hello-world with your package name
          fog_node = fog_node_bin;
          fog_node_naive = fog_node_naive_bin;
          market = market_bin;
          docker_fog_node = dockerImageFogNode;
          docker_fog_node_naive = dockerImageFogNodeNaive;
          docker_market = dockerImageMarket;
        };
        devShells = pkgs.mkShell {
          default = workspaceShell;
        };
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              nixpkgs-fmt.enable = true;
            };
          };
        };
      }
    );
}

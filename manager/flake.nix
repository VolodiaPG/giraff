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
        rustProfile = "default";
        target = "x86_64-unknown-linux-musl";

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          # inherit rustChannel rustProfile target;
          inherit rustChannel rustProfile target;
          packageFun = import ./Cargo.nix;
          rootFeatures = [ ];
          rustcBuildFlags = [
            "-Cforce-frame-pointers=yes"
          ];
          rustcLinkFlags = [
            "-Cforce-frame-pointers=yes"
          ];
        };

        rustPkgs_naive = pkgs.rustBuilder.makePackageSet {
          inherit rustChannel rustProfile target;
          packageFun = import ./Cargo.nix;
          rootFeatures = [ "fog_node/bottom_up_placement" ];
        };

        workspaceShell = (rustPkgs_naive.workspaceShell {
          packages = with pkgs; [
            docker
            just
            rust-analyzer
            (rustfmt.override { asNightly = true; })
            cargo2nix.packages.${system}.cargo2nix
          ];
        });

        dockerImageFogNodeBuilder = { fog_node_bin }: {
          name = "fog_node";
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

        fog_node_bin = (rustPkgs.workspace.fog_node { }).bin;
        market_bin = (rustPkgs.workspace.market { }).bin;

        fog_node_naive_bin = (rustPkgs_naive.workspace.fog_node { }).bin;

        log4rs = pkgs.writeTextDir "/log4rs.yaml" (builtins.readFile ./log4rs.yaml);

        dockerImageFogNode = dockerImageFogNodeBuilder { fog_node_bin = fog_node_bin; };
        dockerImageFogNodeNaive = dockerImageFogNodeBuilder { fog_node_bin = fog_node_naive_bin; };

        dockerImageFogNodePerfTools = pkgs.dockerTools.buildImage
          {
            name = "fog_node";
            tag = "latest";
            config = {
              Cmd = [ "${pkgs.linuxPackages_latest.perf}/bin/perf" "record" "-F99" "--call-graph" "dwarf" "-o" "/var/log/perf.data" "${fog_node_bin}/bin/fog_node" ];
              Env = [ "LOG_CONFIG_PATH=/log4rs.yaml" ];
            };
            runAsRoot = ''
              #!${pkgs.runtimeShell}
              mkdir -p /var/log/
            '';
            # Now renamed to copyToRoot
            contents = pkgs.buildEnv {
              name = "log4rs.yaml";
              pathsToLink = [ "/" "/bin" ];
              paths = [
                log4rs
                pkgs.coreutils
                pkgs.bashInteractive
                pkgs.micro
                pkgs.ps
                pkgs.linuxPackages_latest.perf
              ];
            };
          };

        perftoolsflame = pkgs.fetchFromGitHub {
          owner = "brendangregg";
          repo = "FlameGraph";
          rev = "d9fcc272b6a08c3e3e5b7919040f0ab5f8952d65";
          sha256 = "sha256-1Mk+DJKD21YImpWNTBJL2jWaU2LoCLbGa7+FnJc9ZSY=";
        };

        dockerImagePerfTools = pkgs.dockerTools.buildImage
          {
            name = "perftools";
            tag = "latest";
            config = {
              Cmd = [ pkgs.bashInteractive ];
              Env = [ "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt" ];
            };
            runAsRoot = ''
              #!${pkgs.runtimeShell}
              mkdir -p /var/log/
              mkdir -p /FlameGraph
              mkdir -p /usr/bin
              cp -r ${perftoolsflame}/* /FlameGraph
              ln -s ${pkgs.perl}/bin/* /usr/bin
            '';
            contents = pkgs.buildEnv {
              name = "image-root";
              pathsToLink = [ "/" "/bin" ];
              paths = [
                pkgs.coreutils
                pkgs.bashInteractive
                pkgs.micro
                pkgs.ps
                pkgs.linuxPackages_latest.perf
                fog_node_bin
                pkgs.curl
                pkgs.perl
                pkgs.cacert
              ];
            };
          };

        dockerImageMarket = pkgs.dockerTools.buildImage {
          name = "market";
          tag = "latest";
          config.Cmd = [ "${market_bin}/bin/market" ];
        };
      in
      rec {
        packages = {
          # replace hello-world with your package name
          fog_node = fog_node_bin;
          fog_node_naive = fog_node_naive_bin;
          market = market_bin;

          docker_market = dockerImageMarket;
          docker_fog_node = dockerImageFogNode;
          docker_fog_node_naive = dockerImageFogNodeNaive;
          docker_fog_node_perftools = dockerImageFogNodePerfTools;

          docker_perftools = dockerImagePerfTools;
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

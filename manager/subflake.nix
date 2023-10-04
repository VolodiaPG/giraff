{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
    in
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            config.allowUnfree = true;
            overlays = [cargo2nix.overlays.default];
          };

          #Packages
          rustPkgs = pkgs.rustBuilder.makePackageSet {
            inherit (extra.rustToolchain) rustChannel rustProfile rustVersion extraRustComponents;
            packageFun = import ./Cargo.nix;
            rootFeatures = [];
          };

          # Generators
          pkgsGenerator = {rootFeatures}:
            pkgs.rustBuilder.makePackageSet {
              inherit (extra.rustToolchain) rustChannel rustProfile rustVersion extraRustComponents;
              inherit rootFeatures;
              packageFun = import ./Cargo.nix;
            };

          dockerImageFogNodeGenerator = {
            tag,
            rootFeatures, # crate_name/feature
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
            {
              market = dockerImageMarket;
            }
            // builtins.listToAttrs
            (
              builtins.map
              (
                settings: let
                  tag = "${settings.strategy}_${settings.valuation}_${settings.telemetry}";
                in {
                  name = "fog_node_${tag}";
                  value = dockerImageFogNodeGenerator {
                    inherit tag;
                    rootFeatures =
                      ["fog_node/${settings.strategy}"]
                      ++ nixpkgs.lib.optional (settings.valuation != "valuation_resources") "fog_node/${settings.valuation}"
                      ++ nixpkgs.lib.optional (settings.telemetry != "no-telemetry") "fog_node/${settings.telemetry}"
                      ++ nixpkgs.lib.optional (settings.telemetry != "no-telemetry") "openfaas/${settings.telemetry}";
                  };
                }
              )
              (
                nixpkgs.lib.attrsets.cartesianProductOfSets
                {
                  # Do not forget to run cargo2nix at each new features added
                  strategy = ["auction" "edge_first" "edge_first_v2" "edge_ward" "edge_ward_v2" "edge_ward_v3" "cloud_only" "cloud_only_v2"];
                  valuation = ["valuation_resources" "valuation_rates"];
                  telemetry = ["no-telemetry" "jaeger"];
                }
              )
            );
          devShells.manager = rustPkgs.workspaceShell {
            inherit (outputs.checks.${system}.pre-commit-check) shellHook;
            packages = with pkgs; [
              docker
              just
              pkg-config
              jq
              mprocs
              openssl
              rust-analyzer
              cargo-outdated
              cargo-udeps
              cargo-expand
              # cargo-watch
              lldb
              kubectl
              (rustfmt.override {asNightly = true;})
              cargo2nix.packages.${system}.cargo2nix
              parallel
            ];
          };
        }
      );
}

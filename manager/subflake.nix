{
  outputs = inputs: _extra:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [fenix.overlays.default];
          };

          inherit (pkgs) lib;

          craneLib = crane.lib.${system}.overrideToolchain (fenix.packages.${system}.latest.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ]);

          buildRustPackage = pname: features: let
            features_cmd = builtins.concatStringsSep " " features;
            src = craneLib.cleanCargoSource (craneLib.path ./.);
            commonArgs = {
              inherit src;
              inherit pname;
              version = "0.1";
              strictDeps = true;

              nativeBuildInputs = with pkgs; [
                pkg-config
              ];

              buildInputs = with pkgs;
                [
                  openssl
                ]
                ++ lib.optionals pkgs.stdenv.isDarwin [
                  pkgs.libiconv
                ];
            };
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          in
            craneLib.buildPackage (
              commonArgs
              // {
                inherit cargoArtifacts;
                cargoExtraArgs = "--bin ${pname} --features '${features_cmd}'";
              }
            );

          dockerImageFogNodeGenerator = {
            tag,
            features, # crate_name/feature
          }: let
            fog_node = buildRustPackage "fog_node" features;
          in
            pkgs.dockerTools.buildLayeredImage {
              inherit tag;
              name = "fog_node";
              config = {
                Cmd = ["${fog_node}/bin/fog_node"];
              };
            };

          dockerImageMarket = pkgs.dockerTools.buildLayeredImage {
            name = "market";
            tag = "latest";
            config = {
              Env = ["SERVER_PORT=3003"];
              Cmd = ["${buildRustPackage "market" []}/bin/market"];
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
                    features =
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
          devShells.manager = craneLib.devShell {
            checks = self.checks.${system};

            packages = with pkgs; [
              docker
              just
              pkg-config
              jq
              mprocs
              openssl
              # rust-analyzer
              rust-analyzer-nightly
              cargo-outdated
              cargo-udeps
              cargo-expand
              # cargo-watch
              lldb
              kubectl
              (rustfmt.override {asNightly = true;})
              parallel
            ];
          };
        }
      );
}

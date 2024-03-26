{
  outputs = inputs: extra:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [fenix.overlays.default];
          };

          inherit (pkgs) lib;
          rust = let
            src = ./.;
            symlinks = [./helper ./helper_derive ./model ./openfaas ./kube_metrics];
          in
            extra.buildRustEnv {inherit pkgs src symlinks;};

          dockerImageGenerator = {
            features, # crate_name/feature
            name,
            execName,
            config,
          }: let
            exec = rust.buildRustPackage execName features;
          in
            pkgs.dockerTools.streamLayeredImage {
              inherit name;
              config =
                config
                // {
                  Cmd = ["${nixpkgs.lib.getBin exec}/bin/${execName}"];
                };
            };
        in rec {
          packages =
            (builtins.listToAttrs
              (
                builtins.map
                (
                  settings: let
                    name = "market_${settings.strategy}_${settings.telemetry}";
                  in {
                    inherit name;
                    value = dockerImageGenerator {
                      inherit name;
                      execName = "market";
                      config = {
                        Env = ["SERVER_PORT=3003"];
                      };
                      features =
                        nixpkgs.lib.optional (settings.strategy != "default-strategy") "market/${settings.strategy}"
                        ++ nixpkgs.lib.optional (settings.telemetry != "no-telemetry") "market/${settings.telemetry}";
                    };
                  }
                ) (
                  nixpkgs.lib.attrsets.cartesianProductOfSets
                  {
                    # Do not forget to run cargo2nix at each new features added
                    strategy = ["default-strategy" "powerrandom"];
                    telemetry = ["no-telemetry" "jaeger"];
                  }
                )
              ))
            // (
              builtins.listToAttrs
              (
                builtins.map
                (
                  settings: let
                    name = "fog_node_${settings.strategy}_${settings.valuation}_${settings.telemetry}";
                  in {
                    inherit name;
                    value = dockerImageGenerator {
                      inherit name;
                      execName = "fog_node";
                      config = {
                        Env = ["FUNCTION_LIVE_TIMEOUT_MSECS=120000"];
                      };
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
                    strategy = ["auction" "edge_first" "edge_furthest" "edge_ward" "edge_ward_v3" "powerrandom"];
                    valuation = ["valuation_rates" "quadratic_rates" "powerrandom_rates"];
                    telemetry = ["no-telemetry" "jaeger"];
                  }
                )
              )
            );
          devShells.manager = rust.craneLib.devShell {
            shellHook =
              ((extra.shellHook system) "manager")
              + (extra.shellHookPython pkgs.python3.interpreter);

            packages = with pkgs;
              [
                docker
                just
                pkg-config
                jq
                mprocs
                openssl
                rust-analyzer-nightly
                cargo-outdated
                cargo-udeps
                cargo-expand
                lldb
                kubectl
                (rustfmt.override {asNightly = true;})
                parallel
                skopeo
              ]
              ++ lib.optionals pkgs.stdenv.isDarwin [
                pkgs.libiconv
                darwin.apple_sdk.frameworks.SystemConfiguration
              ];
          };
        }
      );
}

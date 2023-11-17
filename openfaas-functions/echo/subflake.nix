{
  outputs = inputs: extra:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [fenix.overlays.default];
          };

          fwatchdog = pkgs.buildGoModule {
            pname = "of-watchdog";
            version = "giraff-0.1";
            src = inputs.fwatchdog;
            vendorHash = null;
          };

          rust = let
            src = ./.;
            symlinks = [./helper ./model ./helper_derive];
          in
            extra.buildRustEnv {inherit pkgs src symlinks;};

          echoGenerator = {
            tag,
            features, # crate_name/feature
          }: let
            echo = rust.buildRustPackage "echo" features;
          in
            pkgs.dockerTools.streamLayeredImage {
              name = "echo";
              tag = "latest";
              config = {
                Env = [
                  "RUST_LOG=warn,echo=trace"
                  "fprocess=${echo}/bin/echo"
                  "mode=http"
                  "http_upstream_url=http://127.0.0.1:3000"
                ];
                ExposedPorts = {
                  "8080/tcp" = {};
                };
                Cmd = ["${fwatchdog}/bin/of-watchdog"];
              };
            };
        in {
          packages =
            {
              inherit fwatchdog;
            }
            // builtins.listToAttrs (
              builtins.map
              (
                settings: let
                  tag = "${settings.telemetry}";
                in {
                  name = "echo_${tag}";
                  value = echoGenerator {
                    inherit tag;
                    features = nixpkgs.lib.optional (settings.telemetry != "no-telemetry") "${settings.telemetry}";
                  };
                }
              )
              (
                nixpkgs.lib.attrsets.cartesianProductOfSets
                {
                  # Do not forget to run cargo2nix at each new features added
                  telemetry = ["no-telemetry" "jaeger"];
                }
              )
            );
          devShells.openfaas_functions = rust.craneLib.devShell {
            checks = self.checks.${system};

            packages = with pkgs; [
              docker
              faas-cli
              just
              pkg-config
              openssl
              rust-analyzer
              lldb
              skopeo
              (rustfmt.override {asNightly = true;})
            ];
          };
        }
      );
}

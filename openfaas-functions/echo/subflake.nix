{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
    in
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [fenix.overlays.default];
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
              name = "fn_echo";
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
                Cmd = ["${outputs.packages.${system}.fwatchdog}/bin/of-watchdog"];
              };
            };
        in {
          packages = builtins.listToAttrs (
            builtins.map
            (
              settings: let
                tag = "${settings.telemetry}";
              in {
                name = "fn_echo_${tag}";
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
          devShells.fn_echo = rust.craneLib.devShell {
            checks = self.checks.${system};

            packages = with pkgs; [
              just
              pkg-config
              openssl
              rust-analyzer
              skopeo
              (rustfmt.override {asNightly = true;})
            ];
          };
        }
      );
}

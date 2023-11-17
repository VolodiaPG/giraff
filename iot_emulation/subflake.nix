{
  outputs = inputs: extra:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [fenix.overlays.default];
          };

          rust = let
            src = ./.;
            symlinks = [./helper ./helper_derive];
          in
            extra.buildRustEnv {inherit pkgs src symlinks;};

          dockerIOTEmulation = features:
            pkgs.dockerTools.streamLayeredImage {
              name = "iot_emulation";
              tag = "latest";
              config = {
                Env = ["SERVER_PORT=3003"];
                Cmd = ["${rust.buildRustPackage "iot_emulation" features}/bin/iot_emulation"];
              };
            };
        in {
          packages = {
            iot_emulation_no-telemetry = dockerIOTEmulation [];
            iot_emulation_jaeger = dockerIOTEmulation ["jaeger"];
          };
          devShells.iot_emulation = rust.craneLib.devShell {
            checks = self.checks.${system};

            packages = with pkgs; [
              just
              # pkg-config
              jq
              # openssl
              # rust-analyzer
              cargo-outdated
              cargo-udeps
              lldb
              (rustfmt.override {asNightly = true;})
              skopeo
            ];
          };
        }
      );
}

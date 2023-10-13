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

          src = craneLib.cleanCargoSource (craneLib.path ./.);
          helper = craneLib.cleanCargoSource (craneLib.path ./helper);
          helper_derive = craneLib.cleanCargoSource (craneLib.path ./helper_derive);

          # Common arguments can be set here to avoid repeating them later
          commonArgs = {
            inherit src;
            strictDeps = true;

            preConfigurePhases = [
              "link_local_deps"
            ];

            link_local_deps = ''
              ln -s ${helper} ./helper
              ln -s ${helper_derive} ./helper_derive
            '';

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
          iot_emulation = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          dockerIOTEmulation = pkgs.dockerTools.buildLayeredImage {
            name = "iot_emulation";
            tag = "latest";
            config = {
              Env = ["SERVER_PORT=3003"];
              Cmd = ["${iot_emulation}/bin/iot_emulation"];
            };
          };
        in {
          packages = {
            iot_emulation = dockerIOTEmulation;
            iot_emulation_raw = iot_emulation;
          };
          devShells.iot_emulation = craneLib.devShell {
            checks = self.checks.${system};

            # Automatically inherit any build inputs from `my-crate`
            # inputsFrom = [iot_emulation];

            packages = with pkgs;
              [
                just
                # pkg-config
                jq
                # openssl
                # rust-analyzer
                cargo-outdated
                cargo-udeps
                lldb
                (rustfmt.override {asNightly = true;})
              ]
              ++ commonArgs.buildInputs;
          };
        }
      );
}

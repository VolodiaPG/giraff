{
  outputs = inputs: _extra:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [fenix.overlays.default];
          };
          craneLib = crane.lib.${system}.overrideToolchain (fenix.packages.${system}.latest.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ]);
        in {
          devShells.openfaas_functions = craneLib.devShell {
            checks = self.checks.${system};

            packages = with pkgs; [
              docker
              faas-cli
              just
              pkg-config
              openssl
              rust-analyzer
              lldb
              (rustfmt.override {asNightly = true;})
            ];
          };
        }
      );
}

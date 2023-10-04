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

          rustPkgsEcho = pkgs.rustBuilder.makePackageSet {
            inherit (extra.rustToolchain) rustChannel rustProfile rustVersion extraRustComponents;
            packageFun = import ./echo/Cargo.nix;
            rootFeatures = [];
          };
        in rec {
          devShells.openfaas_functions = pkgs.mkShell {
            inherit (outputs.checks.${system}.pre-commit-check) shellHook;
            openfaas_functions = rustPkgsEcho.workspaceShell {
              packages = with pkgs; [
                docker
                faas-cli
                just
                pkg-config
                openssl
                rust-analyzer
                lldb
                (rustfmt.override {asNightly = true;})
                cargo2nix.packages.${system}.cargo2nix
              ];
            };
          };
        }
      );
}

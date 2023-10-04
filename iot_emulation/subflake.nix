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

          dockerIOTEmulation = pkgs.dockerTools.buildLayeredImage {
            name = "iot_emulation";
            tag = "latest";
            config = {
              Env = ["SERVER_PORT=3003"];
              Cmd = ["${(rustPkgs.workspace.iot_emulation {}).bin}/bin/iot_emulation"];
            };
          };
        in {
          packages = {
            iot_emulation = dockerIOTEmulation;
          };
          devShells.iot_emulation = rustPkgs.workspaceShell {
            inherit (outputs.checks.${system}.pre-commit-check) shellHook;
            packages = with pkgs; [
              docker
              just
              pkg-config
              jq
              openssl
              rust-analyzer
              cargo-outdated
              cargo-udeps
              lldb
              (rustfmt.override {asNightly = true;})
              cargo2nix.packages.${system}.cargo2nix
            ];
          };
        }
      );
}

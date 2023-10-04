{
  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            config.allowUnfree = true;
            overlays = [cargo2nix.overlays.default];
          };

          # Define Rust environment to use
          rustChannel = "nightly";
          rustProfile = "minimal";
          rustVersion = "2023-08-16";
          target = "x86_64-unknown-linux-gnu";
          extraRustComponents = ["clippy" "rustfmt"];

          #Packages
          rustPkgs = pkgs.rustBuilder.makePackageSet {
            inherit rustChannel rustProfile target rustVersion extraRustComponents;
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
          formatter = pkgs.alejandra;
          checks = {
            pre-commit-check = pre-commit-hooks.lib.${system}.run {
              src = ./.;
              settings.statix.ignore = ["Cargo.nix"];
              hooks = {
                # Nix
                alejandra.enable = false;
                statix.enable = true;
                deadnix = {
                  enable = true;
                  excludes = ["Cargo.nix"];
                };
                # Git (conventional commits)
                commitizen.enable = true;
              };
            };
          };
          devShells.iot_emulation = rustPkgs.workspaceShell {
            inherit (self.checks.${system}.pre-commit-check) shellHook;
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

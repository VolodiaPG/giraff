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

          rustChannel = "nightly";
          rustProfile = "minimal";
          rustVersion = "2023-06-15";
          target = "x86_64-unknown-linux-gnu";
          extraRustComponents = ["clippy" "rustfmt"];

          rustPkgsEcho = pkgs.rustBuilder.makePackageSet {
            inherit rustChannel rustProfile target rustVersion extraRustComponents;
            packageFun = import ./echo/Cargo.nix;
            rootFeatures = [];
          };
        in rec {
          formatter = alejandra.defaultPackage.${system};
          checks = {
            pre-commit-check = pre-commit-hooks.lib.${system}.run {
              src = ./.;
              settings.statix.ignore = ["Cargo.nix"];
              hooks = {
                # Nix
                alejandra.enable = true;
                statix.enable = true;
                deadnix = {
                  enable = true;
                  excludes = ["Cargo.nix"];
                };
                # Rust
                rust = {
                  enable = true;
                  name = "rust (justfile pre_commit)";
                  entry = "sh -c 'cd openfaas-functions && just pre_commit'";
                  language = "system";
                  pass_filenames = false;
                };
                # Git (conventional commits)
                commitizen.enable = true;
              };
            };
          };
          devShells = pkgs.mkShell {
            inherit (self.checks.${system}.pre-commit-check) shellHook;
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

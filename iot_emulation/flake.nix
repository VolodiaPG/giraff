{
  inputs = {
    nixpkgs.url = "github:Nixos/nixpkgs/nixos-22.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix/release-0.11.0";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-stable.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
          overlays = [ cargo2nix.overlays.default ];
        };

        # Define Rust environment to use
        rustChannel = "nightly";
        rustProfile = "minimal";
        rustVersion = "2023-02-26";
        target = "x86_64-unknown-linux-gnu";

        #Packages
        rustPkgs = pkgs.rustBuilder.makePackageSet {
          inherit rustChannel rustProfile target rustVersion;
          packageFun = import ./Cargo.nix;
          rootFeatures = [ ];
        };

        dockerIOTEmulation = pkgs.dockerTools.buildImage {
          name = "iot_emulation";
          tag = "latest";
          config = {
            Env = [ "SERVER_PORT=3003" ];
            Cmd = [ "${(rustPkgs.workspace.iot_emulation { }).bin}/bin/iot_emulation" ];
          };
        };
      in
      rec {
        packages = {
          iot_emulation = dockerIOTEmulation;
        };
        formatter = nixpkgs.legacyPackages.${system}.nixpkgs-fmt;
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              # Nix
              nixpkgs-fmt.enable = true;
              statix.enable = false;
              deadnix = {
                enable = true;
                excludes = [ "Cargo.nix" ];
              };
              # Rust
              rustfmt.enable = true;
              clippy.enable = true;
              cargo-check.enable = true;
            };
          };
        };
        devShells = nixpkgs.legacyPackages.${system}.mkShell {
          default = (rustPkgs.workspaceShell {
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
              (rustfmt.override { asNightly = true; })
              cargo2nix.packages.${system}.cargo2nix
            ];
          });
        };
      }
    );
}

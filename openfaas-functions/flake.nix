{
  inputs = {
    nixpkgs.url = "github:Nixos/nixpkgs/nixos-unstable";
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix/release-0.11.0";
      inputs.rust-overlay.follows = "rust-overlay";
    };
    cargo2nix.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.follows = "cargo2nix/flake-utils";
    # nixpkgs.follows = "github:Nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
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

        rustChannel = "nightly";
        rustProfile = "minimal";
        rustVersion = "2022-11-05";
        target = "x86_64-unknown-linux-gnu";

        rustPkgsEcho = pkgs.rustBuilder.makePackageSet {
          inherit rustChannel rustProfile target rustVersion;
          packageFun = import ./echo/Cargo.nix;
          rootFeatures = [ ];
        };

        workspaceShell = (rustPkgsEcho.workspaceShell {
          packages = with pkgs; [
            docker
            faas-cli
            just
            pkg-config
            openssl
            rust-analyzer
            lldb
            (rustfmt.override { asNightly = true; })
            cargo2nix.packages.${system}.cargo2nix
          ];
        });
      in
      rec {
        devShells = pkgs.mkShell {
          default = workspaceShell;
        };
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              nixpkgs-fmt.enable = true;
            };
          };
        };
      }
    );
}

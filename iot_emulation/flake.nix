{
  inputs = {
    nixpkgs.url = "github:Nixos/nixpkgs/nixos-unstable";
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix/release-0.11.0";
      inputs.rust-overlay.follows = "rust-overlay";
    };
    # cargo2nix.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.follows = "cargo2nix/flake-utils";
    # nixpkgs.follows = "github:Nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ cargo2nix.overlays.default ];
        };

        rustChannel = "nightly";
        rustProfile = "minimal";
        rustVersion = "2022-11-05";
        target = "x86_64-unknown-linux-musl";

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          inherit rustChannel rustProfile target rustVersion;
          packageFun = import ./Cargo.nix;
        };

        workspaceShell = (rustPkgs.workspaceShell {
          packages = with pkgs; [
            just
            docker
            rust-analyzer
            (rustfmt.override { asNightly = true; })
            cargo2nix.packages.${system}.cargo2nix
          ];
        });


        iot_emulation_bin = (rustPkgs.workspace.iot_emulation { }).bin;

        dockerImageIotEmulation = pkgs.dockerTools.buildImage
          {
            name = "iot_emulation";
            tag = "latest";
            config = {
              Cmd = [ "${iot_emulation_bin}/bin/iot_emulation" ];
              Env = [
                "ROCKET_PORT=3030"
                "ROCKET_ADDRESS=0.0.0.0"
              ];
              Expose = [ 3030 ];
            };
          };
      in
      rec {
        packages = {
          # replace hello-world with your package name
          iot_emulation = iot_emulation_bin;
          docker_iot_emulation = dockerImageIotEmulation;

          default = dockerImageIotEmulation;
        };
        devShells = pkgs.mkShell {
          default = workspaceShell;
        };
      }
    );
}

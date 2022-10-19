{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
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
        target = "x86_64-unknown-linux-musl";

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          inherit rustChannel rustProfile target;
          packageFun = import ./Cargo.nix;
        };

        workspaceShell = (rustPkgs.workspaceShell {
          packages = with pkgs; [
            just
            docker
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

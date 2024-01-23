{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
    in
      flake-utils.lib.eachDefaultSystem (system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [gomod2nix.overlays.default];
        };
        iot_emulation = pkgs.buildGoApplication {
          pname = "iot_emulation";
          version = "0.1";
          pwd = ./.;
          src = ./.;
          modules = ./gomod2nix.toml;
        };
        dockerIOTEmulation = pkgs.dockerTools.streamLayeredImage {
          name = "iot_emulation";
          tag = "latest";
          config = {
            Env = [
              "PORT=30080"
              "PATH_AUDIO=${outputs.packages.${system}.dataset_audio}"
              "PATH_IMAGE=${outputs.packages.${system}.dataset_image}"
            ];
            Cmd = ["${iot_emulation}/bin/iot_emulation"];
          };
        };
      in {
        packages = {
          iot_emulation = dockerIOTEmulation;
          dataset_image = import ./dataset_image.nix {inherit pkgs;};
          dataset_audio = import ./dataset_audio.nix {inherit pkgs;};
        };
        devShells.iot_emulation = pkgs.mkShell {
          shellHook = (extra.shellHook system) "iot_emulation";
          packages = with pkgs; [
            git
            gnumake
            gomod2nix
            gopls
            gotools
            just
            go-tools
            skopeo
            (pkgs.mkGoEnv {pwd = ./.;})
          ];
        };
      });
}

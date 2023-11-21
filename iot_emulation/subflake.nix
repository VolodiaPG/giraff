{
  outputs = inputs: _extra:
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
            Env = ["SERVER_PORT=30080"];
            Cmd = ["${iot_emulation}/bin/iot_emulation"];
          };
        };
      in {
        packages.iot_emulation = dockerIOTEmulation;
        devShells.iot_emulation = pkgs.mkShell {
          inherit (outputs.checks.${system}.pre-commit-check) shellHook;
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

{
  outputs = inputs: extra:
    with inputs;
      flake-utils.lib.eachDefaultSystem (system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [gomod2nix.overlays.default];
        };
      in {
        packages.expe = pkgs.buildGoApplication {
          pname = "expe";
          version = "0.1";
          pwd = ./.;
          src = ./.;
          modules = ./gomod2nix.toml;
        };
        devShells.expe = pkgs.mkShell {
          shellHook = (extra.shellHook system) "expe";
          packages = with pkgs; [
            git
            gnumake
            gomod2nix
            gopls
            gotools
            just
            go-tools
            (pkgs.mkGoEnv {pwd = ./.;})
          ];
        };
      });
}

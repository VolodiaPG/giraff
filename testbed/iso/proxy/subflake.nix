{
  description = "A basic gomod2nix flake";

  outputs = inputs:
    with inputs; let
      inherit (self) outputs;
    in
      flake-utils.lib.eachDefaultSystem (system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [gomod2nix.overlays.default];
        };
      in {
        packages.default = pkgs.buildGoApplication {
          pname = "proxy";
          version = "0.1";
          pwd = ./.;
          src = ./.;
          modules = ./gomod2nix.toml;
        };
        packages.goEnv = pkgs.mkGoEnv {pwd = ./.;};
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              alejandra.enable = true;
              govet.enable = true;
              revive.enable = true;
              # staticcheck.enable = true;
            };
          };
        };
        devShells.proxy = pkgs.mkShell {
          inherit (self.checks.${system}.pre-commit-check) shellHook;
          packages = with pkgs; [
            git
            gnumake
            gomod2nix
            gopls
            gotools
            just
            go-tools
            outputs.packages.${system}.goEnv
          ];
        };
        formatter = pkgs.alejandra;
      });
}

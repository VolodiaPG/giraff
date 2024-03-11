{
  outputs = inputs: extra:
    with inputs; let
      extra' =
        extra
        // {
          openfaas_env = [
            "inject_cgi_headers=true"
          ];
        };
      currentDir = builtins.readDir ./.;
      dirs = builtins.filter (name: currentDir.${name} == "directory") (builtins.attrNames currentDir);
      subflakes = builtins.map (dir: (import ./${dir}/subflake.nix).outputs inputs extra') dirs;
    in
      nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate {}
      ([
          (flake-utils.lib.eachDefaultSystem (
            system: let
              pkgs = nixpkgs.legacyPackages.${system};
              fwatchdog = pkgs.buildGoModule {
                pname = "of-watchdog";
                version = "giraff-0.1";
                src = inputs.fwatchdog;
                vendorHash = null;
                patches = [
                  ./of-watchdog-giraff-headers.patch
                ];
              };
            in {
              packages = {
                inherit fwatchdog;
              };
            }
          ))
          (flake-utils.lib.eachDefaultSystem (
            system: let
              pkgs = nixpkgs.legacyPackages.${system};
            in {
              devShells."openfaas-functions" = pkgs.mkShell {
                shellHook = (extra.shellHook system) "openfaas-functions";
                packages = with pkgs; [
                  parallel
                  skopeo
                  just
                ];
              };
            }
          ))
        ]
        ++ subflakes);
}

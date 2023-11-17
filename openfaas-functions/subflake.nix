{
  outputs = inputs: extra:
    with inputs; let
      currentDir = builtins.readDir ./.;
      dirs = builtins.filter (name: currentDir.${name} == "directory") (builtins.attrNames currentDir);
      subflakes = builtins.map (dir: (import ./${dir}/subflake.nix).outputs inputs extra) dirs;
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
              };
            in {
              packages = {
                inherit fwatchdog;
              };
            }
          ))
        ]
        ++ subflakes);
}

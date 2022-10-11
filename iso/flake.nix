{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    enoslib.url = "github:volodiapg/enoslib-flake";
  };

  outputs = { self, nixpkgs, ... }@inputs:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system; config = { allowUnfree = true; };
        overlays = [ inputs.enoslib.overlay.${system} ];
      };
      lib = nixpkgs.lib;
      r_pkgs = with pkgs.rPackages; [
        # rmarkdown-related packages.
        knitr
        rmarkdown
        tidyverse
        viridis

        # others
        ggplot2
        reticulate
        tidyverse
        igraph
        r2r
        formattable
      ];
    in
    {
      formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
      nixosConfigurations.isoimage = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          ./iso.nix
          "${nixpkgs}/nixos/modules/installer/cd-dvd/installation-cd-base.nix"
        ];
      };

      devShells.${system} = {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            ((rstudioWrapper.override {
              packages = r_pkgs;
            }))
            pandoc # for rstudio
            enoslib
            just
            jq
          ];
        };
      };
    };
}

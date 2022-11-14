{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    enoslib.url = "github:volodiapg/enoslib-flake";
  };

  outputs = { self, nixpkgs, nixpkgs-unstable, ... }@inputs:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system; config = { allowUnfree = true; };
        # overlays = [ inputs.enoslib.overlay.${system} ];
      };
      pkgs-unstable = import nixpkgs-unstable {
        inherit system; config = { allowUnfree = true; };
      };
      lib = nixpkgs.lib;

      r_pkgs = with pkgs.rPackages; [
        # rmarkdown-related packages.
        knitr
        knitLatex
        rmarkdown
        tidyverse
        viridis
        languageserver

        # others
        ggplot2
        reticulate
        tidyverse
        igraph
        r2r
        formattable
        zoo
      ];
    in
    {
      formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
      nixosConfigurations.vm = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          "${nixpkgs}/nixos/modules/profiles/qemu-guest.nix"
          "${nixpkgs}/nixos/modules/profiles/all-hardware.nix"
          ./machine-config.nix
        ];
      };

      qcow2 = import "${nixpkgs}/nixos/lib/make-disk-image.nix" {
        inherit lib pkgs;
        config = (nixpkgs.lib.nixosSystem {
          inherit lib pkgs system;
          modules = [
            "${nixpkgs}/nixos/modules/profiles/qemu-guest.nix"
            "${nixpkgs}/nixos/modules/profiles/all-hardware.nix"
            ./machine-config.nix
          ];
        }).config;
        diskSize = "auto";
        additionalSpace = "2048M"; # Space added after all the necessary 
        format = "qcow2-compressed";
      };


      devShells.${system} = {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (
              (rstudioWrapper.override {
                packages = r_pkgs;
              })
            )
            pandoc # for rstudio
            inputs.enoslib.defaultPackage
            just
            jq
            pkgs-unstable.mprocs
            black
            qemu
          ];
        };
      };
    };
}

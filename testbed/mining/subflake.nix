{
  outputs = inputs: extra:
    with inputs;
      nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate {}
      [
        (flake-utils.lib.eachDefaultSystem (system: let
          pkgs = inputs.nixpkgs.legacyPackages.${system};
          R-pkgs = with pkgs.rPackages; [
            languageserver
            networkD3
            plotly
            htmlwidgets
            htmltools
            #treemapify
            (
              archive.overrideAttrs (old: {
                buildInputs =
                  old.buildInputs
                  ++ (with pkgs; [
                    libarchive
                  ]);
              })
            )
            #cowplot
            #reticulate
            vroom
            tidyverse
            igraph
            #r2r
            formattable
            stringr
            viridis
            # geomtextpath
            scales
            zoo
            #gghighlight
            #ggdist
            #ggbreak
            #lemon
            ggprism
            #ggh4x
            #ggExtra
            tibbletime
            snakecase
            reshape2
            #ggside
            ggbeeswarm
            #ggpubr
            Hmisc
            #rstatix
            #multcompView

            doParallel
            foreach
            multidplyr

            magick
            future_apply
            (
              gganimate.overrideAttrs (old: {
                buildInputs =
                  old.buildInputs
                  ++ (with pkgs; [
                    future_apply
                  ]);
                src = pkgs.fetchgit {
                  url = "https://github.com/VolodiaPG/gganimate.git";
                  hash = "sha256-RGtqslMy2hommHJinaHlkamT+hvmD6hOTthc5DbV6xw=";
                };
              })
            )
            intergraph
            network
            ggnetwork
            memoise
            cachem
          ];
          my-R = pkgs.rWrapper.override {packages = R-pkgs;};
        in {
          devShells.mining = pkgs.mkShell {
            shellHook =
              (extra.shellHook system) "mining";

            packages = with pkgs; [
              just
              my-R
              pandoc
              nodePackages_latest.browser-sync
              entr
              rPackages.styler
              binserve
              parallel
            ];
          };
          devShells.mining-export = pkgs.mkShell {
            shellHook =
              (extra.shellHook system) "mining-export";

            packages = with pkgs; [
              just
              my-R
              python3
              pandoc

              texliveMinimal
              pgf3
              rPackages.tikzDevice
            ];
          };
        }))
      ];
}

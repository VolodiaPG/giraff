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
            cowplot
            patchwork
            dlookr
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
            ggrepel
            #ggbreak
            #lemon
            ggforce
            multcompView
            ggprism
            #ggh4x
            #ggExtra
            tibbletime
            snakecase
            reshape2
            #ggside
            ggbeeswarm
            purrr
            #ggpubr
            Hmisc
            #rstatix
            #multcompView
            car

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
            gifski
          ];
          my-R = pkgs.rWrapper.override {packages = R-pkgs;};
          commonPackages = with pkgs; [
            just
            my-R
            pandoc
            nodePackages_latest.browser-sync
            entr
            rPackages.styler
            binserve
            parallel
          ];
          FONTCONFIG_FILE = pkgs.makeFontsConf {
            fontDirectories = [pkgs.freefont_ttf];
          };
        in {
          devShells.mining = pkgs.mkShell {
            inherit FONTCONFIG_FILE;
            shellHook =
              (extra.shellHook system) "mining";

            packages = commonPackages;
          };
          devShells.mining-export = pkgs.mkShell {
            inherit FONTCONFIG_FILE;
            shellHook =
              (extra.shellHook system) "mining-export";

            packages =
              commonPackages
              ++ (with pkgs; [
                python3
                (texlive.combine {
                  inherit (texlive) scheme-basic xetex pgf preview fontspec xunicode latex-tools-dev ms graphics ec;
                })
                rPackages.tikzDevice
              ]);
          };
        }))
      ];
}

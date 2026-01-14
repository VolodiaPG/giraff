{
  outputs = inputs: extra:
    with inputs;
      nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate {} [
        (flake-utils.lib.eachDefaultSystem (
          system: let
            pkgs = inputs.nixpkgs.legacyPackages.${system};
            R-pkgs = with pkgs.rPackages; [
              languageserver
              networkD3
              plotly
              htmlwidgets
              htmltools
              #treemapify
              (archive.overrideAttrs (old: {
                buildInputs =
                  old.buildInputs
                  ++ (
                    with pkgs;
                      [
                        libarchive
                      ]
                      ++ lib.optionals stdenv.hostPlatform.isDarwin [
                        gfortran
                        libiconv
                      ]
                  );
                # (with pkgs.darwin.apple_sdk.frameworks; pkgs.lib.optionals pkgs.stdenv.isDarwin [Cocoa Foundation pkgs.gfortran pkgs.libiconv]);
              }))
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
              ggpubr
              Hmisc
              #rstatix
              #multcompView
              car
              jsonlite

              doParallel
              foreach
              multidplyr
              targets
              usethis
              crew
              tarchetypes
              qs2
              rstatix

              magick
              future_apply
              (gganimate.overrideAttrs (old: {
                buildInputs =
                  old.buildInputs
                  ++ (with pkgs; [
                    future_apply
                  ]);
                src = pkgs.fetchgit {
                  url = "https://github.com/VolodiaPG/gganimate.git";
                  hash = "sha256-RGtqslMy2hommHJinaHlkamT+hvmD6hOTthc5DbV6xw=";
                };
              }))
              intergraph
              network
              ggnetwork
              memoise
              cachem
              gifski
              hexbin
              akima
            ];
            my-R = pkgs.rWrapper.override {packages = R-pkgs;};
            commonPackages = with pkgs; [
              just
              my-R
              pandoc
              nodePackages_latest.browser-sync
              entr
              rPackages.styler
              rPackages.languageserver
              binserve
              parallel
              # nmap
            ];
            FONTCONFIG_FILE = pkgs.makeFontsConf {
              fontDirectories = [pkgs.freefont_ttf];
            };
          in {
            devShells.mining = pkgs.mkShell {
              inherit FONTCONFIG_FILE;
              shellHook = (extra.shellHook system) "mining";

              packages = commonPackages;
            };
            devShells.mining-export = pkgs.mkShell {
              inherit FONTCONFIG_FILE;
              shellHook = (extra.shellHook system) "mining-export";

              packages =
                commonPackages
                ++ (with pkgs; [
                  python3
                  (texlive.combine {
                    inherit
                      (texlive)
                      scheme-basic
                      xetex
                      pgf
                      preview
                      fontspec
                      xunicode
                      latex-tools-dev
                      graphics
                      ec
                      ;
                  })
                  (rPackages.tikzDevice.overrideAttrs (_: {
                    src = pkgs.fetchFromGitHub {
                      owner = "volodiapg";
                      repo = "tikzDevice";
                      rev = "e3efd4f71ad54ffe7d1b9e28072eb0e79febd5cf";
                      sha256 = "sha256-hBe8s3mfAY91gsKWETyDERhwd8an2tUOB3/75YRZGLY=";
                    };
                  }))
                ]);
            };
          }
        ))
      ];
}

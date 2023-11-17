{
  outputs = inputs: _extra:
    with inputs; let
      inherit (self) outputs;
      fn_name = "image-classification-squeezenet-cpu";
    in
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import poetry2nix.inputs.nixpkgs {
            inherit system;
            overlays = [overlay];
          };

          overlay = self: _super: {
            myFunction = self.python310.withPackages (ps: with ps; [torch torchvision waitress flask pillow]);
          };

          squeezenetModel = pkgs.stdenv.mkDerivation {
            name = "squeezenet";
            version = "1.1";
            src = builtins.fetchurl {
              url = "https://download.pytorch.org/models/squeezenet1_1-b8a52dc0.pth";
              sha256 = "sha256:0yvy9nmms2k5q6yzxch4cf5spbv2fd2xzl4anrm4n3mn9702v9dq";
            };
            unpackPhase = ":";
            installPhase = ''
              cp $src $out
            '';
          };

          image = pkgs.dockerTools.streamLayeredImage {
            name = "fn_${fn_name}";
            tag = "latest";
            extraCommands = ''
              ln -s ${./imagenet_classes.txt} imagenet_classes.txt
            '';
            config = {
              Env = [
                "fprocess=${pkgs.myFunction}/bin/python ${./main.py}"
                "mode=http"
                "http_upstream_url=http://127.0.0.1:5000"
                "SQUEEZENET_MODEL=${squeezenetModel}"
              ];
              ExposedPorts = {
                "8080/tcp" = {};
              };
              Cmd = ["${outputs.packages.${system}.fwatchdog}/bin/of-watchdog"];
            };
          };
        in {
          packages."fn_${fn_name}" = image;
          devShells."fn_${fn_name}" = pkgs.mkShell {
            shellHook =
              outputs.checks.${system}.pre-commit-check.shellHook
              + ''
                mkdir -p ./.venv/bin
                ln -sfT ${pkgs.myFunction}/bin/python ./.venv/bin/python
              '';
            SQUEEZENET_MODEL = squeezenetModel;
            # Fixes https://github.com/python-poetry/poetry/issues/1917 (collection failed to unlock)
            PYTHON_KEYRING_BACKEND = "keyring.backends.null.Keyring";
            packages = with pkgs; [
              just
              skopeo
              myFunction
            ];
          };
        }
      );
}

{
  outputs = inputs: _extra:
    with inputs; let
      inherit (self) outputs;
      fn_name = "sentiment-analysis";
    in
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import poetry2nix.inputs.nixpkgs {
            inherit system;
            overlays = [overlay];
          };

          overlay = self: _super: {
            myFunction = self.poetry2nix.mkPoetryEnv {
              projectDir = ./.;
              python = self.python311;
              overrides = self.poetry2nix.overrides.withDefaults (_newattr: oldattr: {
                urllib3 =
                  oldattr.urllib3.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or []) ++ [oldattr.hatchling];
                    }
                  );
                blinker =
                  oldattr.blinker.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or []) ++ [oldattr.flit-core];
                    }
                  );
                werkzeug =
                  oldattr.werkzeug.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or []) ++ [oldattr.flit-core];
                    }
                  );
                flask =
                  oldattr.flask.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or []) ++ [oldattr.flit-core];
                    }
                  );
                textblob =
                  oldattr.textblob.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or []) ++ [oldattr.setuptools];
                    }
                  );
              });
            };
          };

          punktModel = pkgs.stdenv.mkDerivation rec {
            pname = "punkt";
            version = "1.0";
            src = builtins.fetchurl {
              url = "https://raw.githubusercontent.com/nltk/nltk_data/gh-pages/packages/tokenizers/punkt.zip";
              sha256 = "sha256:1v306rjpjfcqd8mh276lfz8s1d22zgj8n0lfzh5nbbxfjj4hghsi";
            };
            nativeBuildInputs = [pkgs.unzip];
            unpackPhase = "unzip $src -d folder";
            installPhase = ''
              mkdir -p $out/tokenizers
              cp -r folder/* $out/tokenizers
            '';
          };

          image = pkgs.dockerTools.streamLayeredImage {
            name = "fn_${fn_name}";
            tag = "latest";
            config = {
              Env = [
                "fprocess=${pkgs.myFunction}/bin/python ${./main.py}"
                "mode=http"
                "http_upstream_url=http://127.0.0.1:5000"
                "NLTK_DATA=${punktModel}"
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
                ln -sfT ${pkgs.myFunction} ./.venv
              '';
            NLTK_DATA = "${punktModel}";
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

{
  outputs = inputs: _extra:
    with inputs; let
      inherit (self) outputs;
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
                speechrecognition =
                  oldattr.speechrecognition.overridePythonAttrs
                  (
                    old: {
                      buildInputs = (old.buildInputs or []) ++ [oldattr.setuptools];
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
              });
            };
          };

          voskModel = pkgs.stdenv.mkDerivation rec {
            pname = "vosk-model-small-en-us";
            version = "0.15";
            src = builtins.fetchurl {
              url = "https://alphacephei.com/vosk/models/${pname}-${version}.zip";
              sha256 = "sha256:1614jj01gx4zz5kq6fj2lclwp1m6swnk1js2isa9yi7bqi165wih";
            };
            nativeBuildInputs = [pkgs.unzip];
            unpackPhase = "unzip $src -d folder";

            installPhase = ''
              cp -r folder/${pname}-${version}/ $out
            '';
          };

          image = pkgs.dockerTools.streamLayeredImage {
            name = "fn_speech_recognition";
            tag = "latest";
            extraCommands = ''
              ln -s ${voskModel} model
            '';

            config = {
              Env = [
                "fprocess=${pkgs.myFunction}/bin/python ${./main.py}"
                "mode=http"
                "http_upstream_url=http://127.0.0.1:5000"
              ];
              ExposedPorts = {
                "8080/tcp" = {};
              };
              Cmd = ["${outputs.packages.${system}.fwatchdog}/bin/of-watchdog"];
            };
          };
        in {
          packages.fn_speech_recognition = image;
          devShells.fn_speech_recognition = pkgs.mkShell {
            shellHook =
              outputs.checks.${system}.pre-commit-check.shellHook
              + ''
                ln -sfT ${pkgs.myFunction} ./.venv
                ln -sfT ${voskModel} ./model
              '';
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

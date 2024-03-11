{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
      fn_name = "speech_recognition";
    in
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [overlay];
          };

          overlay = self: super: {
            myFunction = self.python311.withPackages (
              ps: (with ps; [
                waitress
                flask
                pillow
                (pyttsx3.overrideAttrs {
                  meta.broken = false; # For darwin
                })
                opentelemetry-exporter-otlp
                opentelemetry-exporter-otlp-proto-grpc
                opentelemetry-api
                opentelemetry-sdk
                opentelemetry-instrumentation-flask
                (buildPythonPackage rec {
                  pname = "vosk";
                  version = "0.3.45";
                  format = "wheel";

                  src = super.fetchPypi rec {
                    inherit pname version format;
                    hash = "sha256-JeAlCTxDmdcnj1Q1aO2MxUYKw6S/SMI2c6zh4l0mYZ8=";
                    dist = python;
                    python = "py3";
                    abi = "none";
                    platform = "manylinux_2_12_x86_64.manylinux2010_x86_64";
                  };

                  # https://pypi.org/pypi/vosk/json
                  propagatedBuildInputs = with pkgs.python3Packages; [
                    cffi
                    requests
                    srt
                    tqdm
                    websockets
                  ];
                })
                (buildPythonPackage rec {
                  pname = "SpeechRecognition";
                  version = "3.10.0";
                  format = "wheel";

                  src = super.fetchPypi rec {
                    inherit pname version format;
                    hash = "sha256-eumWaIfZkJzj5aDCfsw+rPyhb9DAgp939VKRlBjoYwY=";
                  };

                  # https://pypi.org/pypi/SpeechRecognition/json
                  propagatedBuildInputs = with pkgs.python3Packages; [
                    requests
                  ];
                })
              ])
            );
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
            name = "fn_${fn_name}";
            tag = "latest";
            extraCommands = ''
              ln -s ${voskModel} model
            '';

            config = {
              Env =
                [
                  "fprocess=${pkgs.myFunction}/bin/python ${./main.py}"
                  "mode=http"
                  "http_upstream_url=http://127.0.0.1:5000"
                  "OTEL_PYTHON_LOGGING_AUTO_INSTRUMENTATION_ENABLED=true"
                  "ready_path=http://127.0.0.1:3000/health"
                ]
                ++ extra.openfaas_env;
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
              ((extra.shellHook system) "fn_${fn_name}")
              + (extra.shellHookPython pkgs.myFunction.interpreter);
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

{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
      fn_name = "sentiment_analysis";
    in
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [overlay];
          };

          overlay = self: super: {
            myFunction = self.python312.withPackages (
              ps: (with ps; [
                waitress
                flask
                pillow
                nltk
                opentelemetry-exporter-otlp
                opentelemetry-exporter-otlp-proto-grpc
                opentelemetry-api
                opentelemetry-sdk
                opentelemetry-instrumentation-flask
                (buildPythonPackage rec {
                  pname = "textblob";
                  version = "0.17.1";
                  format = "wheel";

                  src = super.fetchPypi rec {
                    inherit pname version format;
                    hash = "sha256-FVRtfzCelqP1Qr7kJ1HI5c5NUZ09J07nnfIxgUHwt4g=";
                  };

                  # https://pypi.org/pypi/textblob/json
                  propagatedBuildInputs = with pkgs.python3Packages; [
                    nltk
                  ];
                })
              ])
            );
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
              Env =
                [
                  "fprocess=${pkgs.myFunction}/bin/python ${./main.py}"
                  "mode=http"
                  "http_upstream_url=http://127.0.0.1:5000"
                  "NLTK_DATA=${punktModel}"
                  "OTEL_PYTHON_LOGGING_AUTO_INSTRUMENTATION_ENABLED=true"
                  "ready_path=http://127.0.0.1:5000/health"
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
              (extra.shellHook system) "fn_${fn_name}";
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

{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
      fn_name = "image_classification_squeezenet_cpu";
    in
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [overlay];
          };

          overlay = self: _super: {
            myFunction = self.python312.withPackages (
              ps: (with ps; [
                torch
                torchvision
                waitress
                flask
                pillow
                opentelemetry-exporter-otlp
                opentelemetry-exporter-otlp-proto-grpc
                opentelemetry-api
                opentelemetry-sdk
                opentelemetry-instrumentation-flask
              ])
            );
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
              Env =
                [
                  "fprocess=${pkgs.myFunction}/bin/python ${./main.py}"
                  "mode=http"
                  "http_upstream_url=http://127.0.0.1:5000"
                  "SQUEEZENET_MODEL=${squeezenetModel}"
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
            SQUEEZENET_MODEL = squeezenetModel;
            # Fixes https://github.com/python-poetry/poetry/issues/1917 (collection failed to unlock)
            PYTHON_KEYRING_BACKEND = "keyring.backends.null.Keyring";
            PATH_IMAGE = outputs.packages.${system}.dataset_image;
            packages = with pkgs; [
              just
              skopeo
              myFunction
            ];
          };
        }
      );
}

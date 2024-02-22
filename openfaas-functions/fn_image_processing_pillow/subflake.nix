{
  outputs = inputs: extra:
    with inputs; let
      inherit (self) outputs;
      fn_name = "image_processing_pillow";
    in
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [overlay];
          };

          overlay = self: _super: {
            myFunction = self.python311.withPackages (ps:
              (with ps; [
                waitress
                flask
                pillow
                opentelemetry-exporter-otlp
                opentelemetry-exporter-otlp-proto-grpc
                opentelemetry-api
                opentelemetry-sdk
              ])
              ++ (with outputs.packages.${system}; [
                otelFlask
              ]));
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
            PATH_IMAGE = "${outputs.packages.${system}.dataset_image}";
            packages = with pkgs; [
              just
              skopeo
              myFunction
            ];
          };
        }
      );
}

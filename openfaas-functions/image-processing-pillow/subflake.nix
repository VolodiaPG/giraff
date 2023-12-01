{
  outputs = inputs: _extra:
    with inputs; let
      inherit (self) outputs;
      fn_name = "image-processing-pillow";
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
                # requests
                opentelemetry-exporter-otlp
                opentelemetry-exporter-otlp-proto-grpc
                opentelemetry-api
                opentelemetry-sdk
              ])
              ++ (with outputs.packages.${system}; [
                otelFlask
                # otelRequests
              ]));
          };

          image = pkgs.dockerTools.streamLayeredImage {
            name = "fn_${fn_name}";
            tag = "latest";
            config = {
              Env = [
                "fprocess=${pkgs.myFunction}/bin/python ${./main.py}"
                "mode=http"
                "http_upstream_url=http://127.0.0.1:5000"
                "OTEL_PYTHON_LOGGING_AUTO_INSTRUMENTATION_ENABLED=true"
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

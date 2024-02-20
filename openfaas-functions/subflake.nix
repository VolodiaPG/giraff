{
  outputs = inputs: extra:
    with inputs; let
      extra' =
        extra
        // {
          openfaas_env = [
            "inject_cgi_headers=true"
          ];
        };
      currentDir = builtins.readDir ./.;
      dirs = builtins.filter (name: currentDir.${name} == "directory") (builtins.attrNames currentDir);
      subflakes = builtins.map (dir: (import ./${dir}/subflake.nix).outputs inputs extra') dirs;
    in
      nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate {}
      ([
          (flake-utils.lib.eachDefaultSystem (
            system: let
              pkgs = nixpkgs.legacyPackages.${system};
              fwatchdog = pkgs.buildGoModule {
                pname = "of-watchdog";
                version = "giraff-0.1";
                src = inputs.fwatchdog;
                vendorHash = null;
                patches = [
                  ./of-watchdog-giraff-headers.patch
                ];
              };
            in {
              packages = {
                inherit fwatchdog;
              };
            }
          ))
          (flake-utils.lib.eachDefaultSystem (
            system: let
              pkgs = nixpkgs.legacyPackages.${system};
              otelFlask = with pkgs.python3Packages;
                buildPythonPackage rec {
                  pname = "opentelemetry-instrumentation-flask";
                  inherit (opentelemetry-instrumentation) version src;
                  disabled = pythonOlder "3.7";

                  sourceRoot = "${opentelemetry-instrumentation.src.name}/instrumentation/${pname}";

                  format = "pyproject";

                  nativeBuildInputs = [
                    hatchling
                  ];

                  propagatedBuildInputs = with pkgs.python3Packages; [
                    opentelemetry-api
                    opentelemetry-instrumentation
                    opentelemetry-instrumentation-asgi
                    opentelemetry-instrumentation-wsgi
                    opentelemetry-semantic-conventions
                    opentelemetry-util-http
                    flask
                  ];

                  nativeCheckInputs = [
                    opentelemetry-test-utils
                    pytestCheckHook
                  ];

                  pythonImportsCheck = ["opentelemetry.instrumentation.flask"];
                };
              otelRequests = with pkgs.python3Packages;
                buildPythonPackage rec {
                  pname = "opentelemetry-instrumentation-requests";
                  inherit (opentelemetry-instrumentation) version src;
                  disabled = pythonOlder "3.7";

                  sourceRoot = "${opentelemetry-instrumentation.src.name}/instrumentation/${pname}";

                  format = "pyproject";

                  nativeBuildInputs = [
                    hatchling
                  ];

                  propagatedBuildInputs = with pkgs.python3Packages; [
                    opentelemetry-api
                    opentelemetry-instrumentation
                    opentelemetry-instrumentation-asgi
                    opentelemetry-instrumentation-wsgi
                    opentelemetry-semantic-conventions
                    opentelemetry-util-http
                    requests
                  ];

                  nativeCheckInputs =
                    [
                      opentelemetry-test-utils
                      pytestCheckHook
                    ]
                    ++ (with pkgs.python3Packages; [
                      httpretty
                    ]);

                  pythonImportsCheck = ["opentelemetry.instrumentation.requests"];
                };
            in {
              packages = {
                inherit otelFlask otelRequests;
              };
              devShells."openfaas-functions" = pkgs.mkShell {
                shellHook = (extra.shellHook system) "openfaas-functions";
                packages = with pkgs; [
                  parallel
                  skopeo
                  just
                ];
              };
            }
          ))
        ]
        ++ subflakes);
}

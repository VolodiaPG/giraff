{inputs}:
with inputs; let
  buildRustEnv = {
    pkgs,
    src,
    symlinks ? [],
  }: let
    craneLib = (crane.mkLib pkgs).overrideToolchain (fenix.packages.${pkgs.system}.latest.withComponents [
      "cargo"
      "clippy"
      "rust-src"
      "rustc"
      "rustfmt"
    ]);

    buildRustPackage = pname: features: let
      features_cmd = builtins.concatStringsSep " " features;

      commonArgs = {
        inherit pname;

        src = craneLib.cleanCargoSource (craneLib.path src);
        version = "0.1";
        strictDeps = true;
        CARGO_BUILD_JOBS = -1;

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        preConfigurePhases = [
          "link_local_deps"
        ];

        link_local_deps = builtins.concatStringsSep "; " (builtins.map (path: "ln -s " + (craneLib.cleanCargoSource (craneLib.path path)) + " ./" + builtins.baseNameOf path) symlinks);

        buildInputs = with pkgs;
          [
            openssl
          ]
          ++ lib.optionals stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            libiconv
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];
      };
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in
      craneLib.buildPackage (
        commonArgs
        // {
          inherit cargoArtifacts;
          cargoExtraArgs = "--bin ${pname} --features '${features_cmd}'";
        }
      );
  in {
    inherit buildRustPackage craneLib;
  };
in
  buildRustEnv

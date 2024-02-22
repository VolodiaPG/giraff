{pkgs, ...}:
pkgs.stdenv.mkDerivation rec {
  pname = "imagette-10";
  version = "1.0";
  src = builtins.fetchurl {
    url = "s3://fast-ai-imageclas/imagenette2-320.tgz";
    sha256 = "sha256:130miqcg0iyh228gdv9w0lky2prirw4z3l9myclxvdldr6bl96sn"; # Replace with the actual hash
  };

  nativeBuildInputs = [pkgs.unzip];
  unpackPhase = "tar -xzf $src";

  installPhase = ''
    mkdir -p $out
    find -name '*.JPEG' -type f -print0 | xargs -0 -r -- cp -t "$out/" --
  '';

  meta = with pkgs.lib; {
    description = "Imagenette dataset";
    homepage = "https://github.com/fastai/imagenette";
    license = licenses.bsd3; # or the appropriate license
  };
}

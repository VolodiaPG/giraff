{pkgs, ...}:
pkgs.stdenv.mkDerivation rec {
  pname = "iamgenet-10";
  version = "1.0";

  src = pkgs.fetchurl {
    url = "https://github.com/ultralytics/yolov5/releases/download/v1.0/imagenet10.zip";
    hash = "sha256-060EaXUQxej3Vzheobp+KFwQ5LyNi4MS7oNJUj5FSJA="; # Replace with the actual hash
  };

  nativeBuildInputs = [pkgs.unzip];
  unpackPhase = "unzip $src -d folder";

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

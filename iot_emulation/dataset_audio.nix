{pkgs, ...}:
pkgs.stdenv.mkDerivation rec {
  pname = "librispeech-mini";
  version = "2.0";

  src = pkgs.fetchurl {
    url = "https://www.openslr.org/resources/31/dev-clean-2.tar.gz";
    hash = "sha256-F27FAUkOztLWwfifTw3cff55nmSeUyL4ukn+P/UMgBI="; # Replace with the actual hash
  };

  nativeBuildInputs = with pkgs; [
    flac
  ];

  unpackPhase = "tar -xzf $src";

  # find . -type f -name '*.flac' -print0 | xargs -0 -I {} sh -c 'f="{}"; flac --decode "$f" "''${f%.flac}.wav"'
  buildPhase = ''
    echo "Converting from FLAC to WAV"
    find . -type f -name '*.flac' -print0 | xargs -0 -I {} sh -c 'f="{}"; flac --no-keep-foreign-metadata --decode "$f"'
  '';

  installPhase = ''
    mkdir -p $out
    find -name '*.wav' -type f -print0 | xargs -0 -r -- cp -t "$out/" --
  '';

  meta = with pkgs.lib; {
    description = "LibriSpeech (mini) dataset";
    homepage = "https://www.openslr.org/31/";
    license = licenses.cc-by-40; # or the appropriate license
  };
}

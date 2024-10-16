{pkgs, ...}: let
  listenAddress = "0.0.0.0";
  port = 5555;

  tls-cert = {alt ? []}: (pkgs.runCommand "selfSignedCert" {buildInputs = [pkgs.openssl];} ''
    mkdir -p $out
    openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:secp384r1 -days 365 -nodes \
      -keyout $out/cert.key -out $out/cert.crt \
      -subj "/CN=localhost" -addext "subjectAltName=DNS:localhost,${builtins.concatStringsSep "," (["IP:127.0.0.1"] ++ alt)}"
  '');
  cert = tls-cert {};
  certificate = "${cert}/cert.crt";
  key = "${cert}/cert.key";
in {
  services.dockerRegistry = {
    enable = true;
    inherit port;
    inherit listenAddress;
    extraConfig = {
      http.tls = {
        inherit certificate key;
      };
      storage = {
        filesystem = {
          rootdirectory = "/var/lib/docker-registry";
        };
      };
    };
  };
  networking.firewall.allowedTCPPorts = [port];
  environment.etc."containers/policy.json".text = ''
    {
        "default": [
            {
                "type": "insecureAcceptAnything"
            }
        ]
    }
  '';
}
